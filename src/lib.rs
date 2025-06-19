use std::{
    env::args,
    ffi::{CStr, CString, c_char, c_int, c_void},
    num::NonZeroIsize,
    ops::Deref,
    ptr::{NonNull, null},
    sync::Arc,
    thread,
};

use raw_window_handle::{AppKitWindowHandle, RawWindowHandle, Win32WindowHandle};

pub use self::sys::{
    Modifiers, MouseButtons, PageState as WebViewState, Rect, TouchEventType, TouchPointerType,
};

#[allow(
    dead_code,
    unused_imports,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]
mod sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

/// A pointer type that is assumed to be thread-safe.
///
/// The creator of this type must ensure that the pointer implementation is thread-safe.
struct ThreadSafePointer<T>(*mut T);

unsafe impl<T> Send for ThreadSafePointer<T> {}
unsafe impl<T> Sync for ThreadSafePointer<T> {}

impl<T> ThreadSafePointer<T> {
    fn as_ptr(&self) -> *mut T {
        self.0
    }
}

trait CStringExt {
    fn as_raw(&self) -> *const c_char;
}

impl CStringExt for Option<CString> {
    fn as_raw(&self) -> *const c_char {
        self.as_ref()
            .map(|it| it.as_c_str().as_ptr() as _)
            .unwrap_or_else(|| null())
    }
}

struct Args {
    #[allow(unused)]
    inner: Vec<CString>,
    raw: Vec<*const c_char>,
}

unsafe impl Send for Args {}
unsafe impl Sync for Args {}

impl Default for Args {
    fn default() -> Self {
        let inner = args()
            .map(|it| CString::new(it).unwrap())
            .collect::<Vec<_>>();

        let raw = inner
            .iter()
            .map(|it| it.as_c_str().as_ptr())
            .collect::<Vec<_>>();

        Self { inner, raw }
    }
}

impl Args {
    fn size(&self) -> usize {
        self.raw.len()
    }

    fn as_ptr(&self) -> *const *const c_char {
        self.raw.as_ptr() as _
    }
}

#[derive(Debug)]
pub enum Error {
    FailedToCreateRuntime,
    FailedToCreateWebView,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Runtime configuration attributes
#[derive(Default)]
pub struct RuntimeAttributes {
    /// Whether to enable windowless rendering mode
    ///
    /// Do not enable this value if the application does not use windowless rendering as it may reduce
    /// rendering performance on some systems.
    windowless_rendering_enabled: bool,

    /// The directory where data for the global browser cache will be stored on disk
    cache_dir_path: Option<CString>,

    /// The path to a separate executable that will be launched for sub-processes
    ///
    /// This executable will be launched to handle sub-processes.
    browser_subprocess_path: Option<CString>,

    /// The directory path for custom protocol handlers
    scheme_dir_path: Option<CString>,

    /// The path to the CEF framework directory on macOS
    ///
    /// If this value is empty, the framework must exist at
    /// "Contents/Frameworks/Chromium Embedded Framework.framework" in the top-level app bundle.
    /// If this value is non-empty, it must be an absolute path. Also configurable using the
    /// "framework-dir-path" command-line switch.
    framework_dir_path: Option<CString>,

    /// The path to the main bundle on macOS
    ///
    /// If this value is empty, the main bundle must exist at "Contents/MacOS/main" in the top-level app bundle.
    /// If this value is non-empty, it must be an absolute path. Also configurable using the
    /// "main-bundle-path" command-line switch.
    main_bundle_path: Option<CString>,
}

impl RuntimeAttributes {
    pub fn create_runtime<T>(&self, handler: T) -> Result<Runtime, Error>
    where
        T: RuntimeHandler + 'static,
    {
        Runtime::new(&self, handler)
    }
}

#[derive(Default)]
pub struct RuntimeAttributesBuilder(RuntimeAttributes);

impl RuntimeAttributesBuilder {
    /// Enable windowless rendering mode
    ///
    /// Do not enable this value if the application does not use windowless rendering as it may reduce
    /// rendering performance on some systems.
    pub fn with_windowless_rendering_enabled(mut self, value: bool) -> Self {
        self.0.windowless_rendering_enabled = value;
        self
    }

    /// Set the directory where data for the global browser cache will be stored on disk
    pub fn with_cache_dir_path(mut self, value: &str) -> Self {
        self.0.cache_dir_path = Some(CString::new(value).unwrap());
        self
    }

    /// Set the path to a separate executable that will be launched for sub-processes
    ///
    /// This executable will be launched to handle sub-processes.
    pub fn with_browser_subprocess_path(mut self, value: &str) -> Self {
        self.0.browser_subprocess_path = Some(CString::new(value).unwrap());
        self
    }

    /// Set the directory path for custom protocol handlers
    pub fn with_scheme_dir_path(mut self, value: &str) -> Self {
        self.0.scheme_dir_path = Some(CString::new(value).unwrap());
        self
    }

    /// Set the path to the CEF framework directory on macOS
    ///
    /// If this value is empty, the framework must exist at
    /// "Contents/Frameworks/Chromium Embedded Framework.framework" in the top-level app bundle.
    /// If this value is non-empty, it must be an absolute path. Also configurable using the
    /// "framework-dir-path" command-line switch.
    pub fn with_framework_dir_path(mut self, value: &str) -> Self {
        self.0.framework_dir_path = Some(CString::new(value).unwrap());
        self
    }

    /// Set the path to the main bundle on macOS
    ///
    /// If this value is empty, the main bundle must exist at "Contents/MacOS/main" in the top-level app bundle.
    /// If this value is non-empty, it must be an absolute path. Also configurable using the
    /// "main-bundle-path" command-line switch.
    pub fn with_main_bundle_path(mut self, value: &str) -> Self {
        self.0.main_bundle_path = Some(CString::new(value).unwrap());
        self
    }

    pub fn build(self) -> RuntimeAttributes {
        self.0
    }
}

impl Deref for RuntimeAttributesBuilder {
    type Target = RuntimeAttributes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(unused_variables)]
pub trait RuntimeHandler: Send + Sync {
    /// Called when the context is initialized
    ///
    /// This callback is called when the application's context is initialized.
    fn on_context_initialized(&self) {}

    /// Called when scheduling message pump work
    ///
    /// This callback is called when scheduling message pump work.
    ///
    /// The `delay` parameter indicates how long to wait before calling `poll`.
    fn on_schedule_message_pump_work(&self, delay: u64) {}
}

#[allow(unused)]
pub struct Runtime {
    handler: ThreadSafePointer<Box<dyn RuntimeHandler>>,
    raw: Arc<ThreadSafePointer<c_void>>,
}

impl Runtime {
    /// Execute subprocess
    ///
    /// This function is used to execute subprocesses.
    /// 
    /// ### Please be careful! 
    /// 
    /// Do not call this function in an asynchronous runtime, such as tokio, 
    /// which can lead to unexpected crashes!
    pub fn execute_subprocess() -> bool {
        let args = Args::default();

        (unsafe { sys::execute_subprocess(args.size() as _, args.as_ptr() as _) }) == 0
    }

    /// Check if current process is a subprocess
    ///
    /// This function is used to check if the current process is a subprocess.
    ///
    /// Note that if the current process is a subprocess, it will block until the subprocess exits.
    pub fn is_subprocess() -> bool {
        args().find(|it| it.contains("--type")).is_some()
    }

    fn new<T>(attr: &RuntimeAttributes, handler: T) -> Result<Self, Error>
    where
        T: RuntimeHandler + 'static,
    {
        let mut options = sys::AppOptions {
            cache_dir_path: attr.cache_dir_path.as_raw(),
            scheme_dir_path: attr.scheme_dir_path.as_raw(),
            browser_subprocess_path: attr.browser_subprocess_path.as_raw(),
            windowless_rendering_enabled: attr.windowless_rendering_enabled,
            main_bundle_path: attr.main_bundle_path.as_raw(),
            framework_dir_path: attr.framework_dir_path.as_raw(),
            // Only macOS doesn't support multi-threaded message loops.
            // To shield users from these details, we specifically enable the
            // message pump driver approach for macOS.
            external_message_pump: cfg!(target_os = "macos"),
            multi_threaded_message_loop: !cfg!(target_os = "macos"),
        };

        let handler: *mut Box<dyn RuntimeHandler> = Box::into_raw(Box::new(Box::new(handler)));
        let ptr = unsafe {
            sys::create_app(
                &mut options,
                sys::AppObserver {
                    on_context_initialized: Some(Self::on_context_initialized),
                    on_schedule_message_pump_work: Some(Self::on_schedule_message_pump_work),
                },
                handler as _,
            )
        };

        let raw = if ptr.is_null() {
            return Err(Error::FailedToCreateRuntime);
        } else {
            Arc::new(ThreadSafePointer(ptr))
        };

        {
            let args = Args::default();

            if cfg!(target_os = "macos") {
                // On macOS, multi-threaded message loop is not supported, so we execute directly.
                // This won't block the current thread, it just does some initialization work.
                unsafe {
                    sys::execute_app(raw.as_ptr(), args.size() as _, args.as_ptr() as _);
                }
            } else {
                // On other platforms, multi-threaded message loop is supported, so we need to create a new thread.
                //
                // The execution is blocking, so we create a separate thread to run the message loop,
                // no external handling is needed.
                let raw = raw.clone();
                thread::spawn(move || unsafe {
                    sys::execute_app(raw.as_ptr(), args.size() as _, args.as_ptr() as _);
                });
            }
        }

        Ok(Self {
            handler: ThreadSafePointer(handler),
            raw,
        })
    }

    pub fn create_webview<T>(
        &self,
        url: &str,
        attr: &WebViewAttributes,
        handler: T,
    ) -> Result<WebView, Error>
    where
        T: WebViewHandler + 'static,
    {
        WebView::new(&self, url, attr, handler)
    }

    extern "C" fn on_context_initialized(ctx: *mut c_void) {
        unsafe { &*(ctx as *mut Box<dyn RuntimeHandler>) }.on_context_initialized();
    }

    extern "C" fn on_schedule_message_pump_work(delay: i64, ctx: *mut c_void) {
        unsafe { &*(ctx as *mut Box<dyn RuntimeHandler>) }
            .on_schedule_message_pump_work(delay as u64);
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // On macOS, the multi-threaded message loop is not supported, so we
        // don't need to quit it.
        if !cfg!(target_os = "macos") {
            unsafe {
                sys::quit_message_loop();
            }
        }

        unsafe {
            sys::close_app(self.raw.as_ptr());
        }

        drop(unsafe { Box::from_raw(self.handler.as_ptr()) });
    }
}

pub trait RuntimeExtMacos {
    /// Run the message loop on macOS
    ///
    /// This function is used to run the message loop on macOS.
    ///
    /// Note that this function will block the current thread until the message loop ends.
    fn block_run();

    /// Drive the message loop pump on macOS
    ///
    /// This function is used to poll the message loop on macOS.
    ///
    /// Note that this function won't block the current thread, external code needs to drive the message loop pump.
    fn poll();

    /// Quit the message loop on macOS
    ///
    /// This function is used to quit the message loop on macOS.
    ///
    /// Calling this function will cause `block_run` to exit and return.
    fn quit();
}

impl RuntimeExtMacos for Runtime {
    fn block_run() {
        unsafe { sys::run_message_loop() }
    }

    fn poll() {
        unsafe { sys::poll_message_loop() }
    }

    fn quit() {
        unsafe {
            sys::quit_message_loop();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ActionState {
    Down,
    Up,
}

impl ActionState {
    pub fn is_pressed(self) -> bool {
        self == Self::Down
    }
}

#[derive(Debug, Clone)]
pub enum MouseAction {
    Click(MouseButtons, ActionState, Option<Position>),
    Move(Position),
    Wheel(Position),
}

#[derive(Debug)]
pub enum IMEAction<'a> {
    Composition(&'a str),
    Pre(&'a str, i32, i32),
}

#[allow(unused)]
pub trait WebViewHandler: Send + Sync {
    /// Called when the web page state changes
    ///
    /// This callback is called when the web page state changes.
    fn on_state_change(&self, state: WebViewState) {}

    /// Called when the IME composition rectangle changes
    ///
    /// This callback is called when the IME composition rectangle changes.
    fn on_ime_rect(&self, rect: Rect) {}

    /// Push a new frame when rendering changes
    ///
    /// This only works in windowless rendering mode.
    fn on_frame(&self, texture: &[u8], width: u32, height: u32) {}

    /// Called when the title changes
    ///
    /// This callback is called when the title changes.
    fn on_title_change(&self, title: &str) {}

    /// Called when the fullscreen state changes
    ///
    /// This callback is called when the fullscreen state changes.
    fn on_fullscreen_change(&self, fullscreen: bool) {}

    /// Called when a message is received
    ///
    /// This callback is called when a message is received from the web page.
    fn on_message(&self, message: &str) {}
}

#[derive(Debug)]
pub struct WebViewAttributes {
    /// External native window handle.
    pub window_handle: Option<RawWindowHandle>,
    /// The maximum rate in frames per second (fps).
    pub windowless_frame_rate: u32,
    /// window size width.
    pub width: u32,
    /// window size height.
    pub height: u32,
    /// window device scale factor.
    pub device_scale_factor: f32,
    /// page defalt fixed font size.
    pub default_font_size: u32,
    /// page defalt fixed font size.
    pub default_fixed_font_size: u32,
    /// Controls whether JavaScript can be executed.
    pub javascript_enable: bool,
    /// Controls whether JavaScript can access the clipboard.
    pub javascript_access_clipboard: bool,
    /// Controls whether local storage can be used.
    pub local_storage: bool,
}

unsafe impl Send for WebViewAttributes {}
unsafe impl Sync for WebViewAttributes {}

impl Default for WebViewAttributes {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            window_handle: None,
            device_scale_factor: 1.0,
            windowless_frame_rate: 30,
            default_font_size: 12,
            default_fixed_font_size: 12,
            javascript_enable: true,
            local_storage: true,
            javascript_access_clipboard: false,
        }
    }
}

#[derive(Default)]
pub struct WebViewAttributesBuilder(WebViewAttributes);

impl WebViewAttributesBuilder {
    /// Set the window handle
    ///
    /// In windowed mode, setting the window handle will set the browser as a child view.
    ///
    /// In windowless mode, setting the window handle is used to identify monitor information and as a parent view
    /// for dialog boxes, context menus, and other elements. If not provided, the main screen monitor will be used,
    /// and some features that require a parent view may not work properly.
    pub fn with_window_handle(mut self, value: RawWindowHandle) -> Self {
        self.0.window_handle = Some(value);
        self
    }

    /// Set the frame rate in windowless rendering mode
    ///
    /// This function is used to set the frame rate in windowless rendering mode.
    ///
    /// Note that this parameter only works in windowless rendering mode.
    pub fn with_windowless_frame_rate(mut self, value: u32) -> Self {
        self.0.windowless_frame_rate = value;
        self
    }

    /// Set the window width
    ///
    /// This function is used to set the window width.
    ///
    /// Note that this parameter only works in windowless rendering mode.
    pub fn with_width(mut self, value: u32) -> Self {
        self.0.width = value;
        self
    }

    /// Set the window height
    ///
    /// This function is used to set the window height.
    ///
    /// Note that this parameter only works in windowless rendering mode.
    pub fn with_height(mut self, value: u32) -> Self {
        self.0.height = value;
        self
    }

    /// Set the device scale factor
    ///
    /// This function is used to set the device scale factor.
    pub fn with_device_scale_factor(mut self, value: f32) -> Self {
        self.0.device_scale_factor = value;
        self
    }

    /// Set the default font size
    ///
    /// This function is used to set the default font size.
    pub fn with_default_font_size(mut self, value: u32) -> Self {
        self.0.default_font_size = value;
        self
    }

    /// Set the default fixed font size
    ///
    /// This function is used to set the default fixed font size.
    pub fn with_default_fixed_font_size(mut self, value: u32) -> Self {
        self.0.default_fixed_font_size = value;
        self
    }

    /// Set whether JavaScript is enabled
    ///
    /// This function is used to set whether JavaScript is enabled.
    pub fn with_javascript_enable(mut self, value: bool) -> Self {
        self.0.javascript_enable = value;
        self
    }

    /// Set whether JavaScript can access the clipboard
    ///
    /// This function is used to set whether JavaScript can access the clipboard.
    pub fn with_javascript_access_clipboard(mut self, value: bool) -> Self {
        self.0.javascript_access_clipboard = value;
        self
    }

    /// Set whether local storage is enabled
    ///
    /// This function is used to set whether local storage is enabled.
    pub fn with_local_storage(mut self, value: bool) -> Self {
        self.0.local_storage = value;
        self
    }

    pub fn build(self) -> WebViewAttributes {
        self.0
    }
}

impl Deref for WebViewAttributesBuilder {
    type Target = WebViewAttributes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct WebView {
    handler: ThreadSafePointer<Box<dyn WebViewHandler>>,
    raw: ThreadSafePointer<c_void>,
}

impl WebView {
    fn new<T>(
        runtime: &Runtime,
        url: &str,
        attr: &WebViewAttributes,
        handler: T,
    ) -> Result<Self, Error>
    where
        T: WebViewHandler + 'static,
    {
        let options = sys::PageOptions {
            width: attr.width,
            height: attr.height,
            device_scale_factor: attr.device_scale_factor,
            windowless_frame_rate: attr.windowless_frame_rate,
            default_fixed_font_size: attr.default_fixed_font_size as _,
            default_font_size: attr.default_font_size as _,
            javascript: attr.javascript_enable,
            javascript_access_clipboard: attr.javascript_access_clipboard,
            local_storage: attr.local_storage,
            window_handle: if let Some(it) = attr.window_handle {
                match it {
                    RawWindowHandle::Win32(it) => it.hwnd.get() as _,
                    RawWindowHandle::AppKit(it) => it.ns_view.as_ptr() as _,
                    _ => unimplemented!("{:?}", it),
                }
            } else {
                null()
            },
        };

        let url = CString::new(url).unwrap();
        let handler: *mut Box<dyn WebViewHandler> = Box::into_raw(Box::new(Box::new(handler)));
        let ptr = unsafe {
            sys::create_page(
                runtime.raw.as_ptr(),
                url.as_c_str().as_ptr(),
                &options,
                sys::PageObserver {
                    on_state_change: Some(Self::on_state_change_callback),
                    on_ime_rect: Some(Self::on_ime_rect_callback),
                    on_frame: Some(Self::on_frame_callback),
                    on_title_change: Some(Self::on_title_change_callback),
                    on_fullscreen_change: Some(Self::on_fullscreen_change_callback),
                    on_message: Some(Self::on_message_callback),
                },
                handler as _,
            )
        };

        let raw = if ptr.is_null() {
            return Err(Error::FailedToCreateWebView);
        } else {
            ThreadSafePointer(ptr)
        };

        Ok(Self {
            handler: ThreadSafePointer(handler),
            raw,
        })
    }

    /// Send a message
    ///
    /// This function is used to send a message to the web page.
    ///
    /// Messages sent from the web page are received through the `WebViewHandler::on_message` callback.
    pub fn send_message(&self, message: &str) {
        let message = CString::new(message).unwrap();

        unsafe {
            sys::page_send_message(self.raw.as_ptr(), message.as_c_str().as_ptr());
        }
    }

    /// Send a mouse event
    ///
    /// This function is used to send mouse events.
    ///
    /// Note that this function only works in windowless rendering mode.
    pub fn mouse(&self, action: MouseAction) {
        match action {
            MouseAction::Move(pos) => unsafe {
                sys::page_send_mouse_move(self.raw.as_ptr(), pos.x, pos.y)
            },
            MouseAction::Wheel(pos) => unsafe {
                sys::page_send_mouse_wheel(self.raw.as_ptr(), pos.x, pos.y)
            },
            MouseAction::Click(button, state, pos) => {
                if let Some(pos) = pos {
                    unsafe {
                        sys::page_send_mouse_click_with_pos(
                            self.raw.as_ptr(),
                            button,
                            state.is_pressed(),
                            pos.x,
                            pos.y,
                        )
                    }
                } else {
                    unsafe {
                        sys::page_send_mouse_click(self.raw.as_ptr(), button, state.is_pressed())
                    }
                }
            }
        }
    }

    /// Send a keyboard event
    ///
    /// This function is used to send keyboard events.
    ///
    /// Note that this function only works in windowless rendering mode.
    pub fn keyboard(&self, scan_code: u32, state: ActionState, modifiers: Modifiers) {
        unsafe {
            sys::page_send_keyboard(
                self.raw.as_ptr(),
                scan_code as c_int,
                state.is_pressed(),
                modifiers,
            )
        }
    }

    /// Send a touch event
    ///
    /// This function is used to send touch events.
    ///
    /// Note that this function only works in windowless rendering mode.
    pub fn touch(
        &self,
        id: i32,
        x: i32,
        y: i32,
        ty: TouchEventType,
        pointer_type: TouchPointerType,
    ) {
        unsafe { sys::page_send_touch(self.raw.as_ptr(), id, x, y, ty, pointer_type) }
    }

    /// Send an IME event
    ///
    /// This function is used to send IME events.
    ///
    /// Note that this function only works in windowless rendering mode.
    pub fn ime(&self, action: IMEAction) {
        let input = match action {
            IMEAction::Composition(it) | IMEAction::Pre(it, _, _) => CString::new(it).unwrap(),
        };

        match action {
            IMEAction::Composition(_) => unsafe {
                sys::page_send_ime_composition(self.raw.as_ptr(), input.as_c_str().as_ptr())
            },
            IMEAction::Pre(_, x, y) => unsafe {
                sys::page_send_ime_set_composition(
                    self.raw.as_ptr(),
                    input.as_c_str().as_ptr(),
                    x,
                    y,
                )
            },
        }
    }

    /// Resize the window
    ///
    /// This function is used to resize the window.
    ///
    /// Note that this function only works in windowless rendering mode.
    pub fn resize(&self, width: u32, height: u32) {
        unsafe { sys::page_resize(self.raw.as_ptr(), width as c_int, height as c_int) }
    }

    /// Get the window handle
    ///
    /// This function is used to get the window handle.
    pub fn window_handle(&self) -> RawWindowHandle {
        let handle = unsafe { sys::page_get_hwnd(self.raw.as_ptr()) };
        if handle.is_null() {
            panic!("window handle pointer is null!");
        }

        if cfg!(target_os = "windows") {
            RawWindowHandle::Win32(Win32WindowHandle::new(
                NonZeroIsize::new(handle as _).unwrap(),
            ))
        } else if cfg!(target_os = "macos") {
            RawWindowHandle::AppKit(AppKitWindowHandle::new(NonNull::new(handle as _).unwrap()))
        } else {
            unimplemented!()
        }
    }

    /// Set whether developer tools are enabled
    ///
    /// This function is used to set whether developer tools are enabled.
    pub fn devtools_enabled(&self, enable: bool) {
        unsafe { sys::page_set_devtools_state(self.raw.as_ptr(), enable) }
    }

    extern "C" fn on_state_change_callback(state: WebViewState, ctx: *mut c_void) {
        unsafe { &*(ctx as *mut Box<dyn WebViewHandler>) }.on_state_change(state);
    }

    extern "C" fn on_ime_rect_callback(rect: Rect, ctx: *mut c_void) {
        (unsafe { &*(ctx as *mut Box<dyn WebViewHandler>) }).on_ime_rect(rect);
    }

    extern "C" fn on_frame_callback(
        texture: *const c_void,
        width: c_int,
        height: c_int,
        ctx: *mut c_void,
    ) {
        (unsafe { &*(ctx as *mut Box<dyn WebViewHandler>) }).on_frame(
            unsafe {
                std::slice::from_raw_parts(texture as _, width as usize * height as usize * 4)
            },
            width as u32,
            height as u32,
        );
    }

    extern "C" fn on_title_change_callback(title: *const c_char, ctx: *mut c_void) {
        if !title.is_null() {
            if let Ok(title) = unsafe { CStr::from_ptr(title) }.to_str() {
                (unsafe { &*(ctx as *mut Box<dyn WebViewHandler>) }).on_title_change(title);
            }
        }
    }

    extern "C" fn on_fullscreen_change_callback(fullscreen: bool, ctx: *mut c_void) {
        (unsafe { &*(ctx as *mut Box<dyn WebViewHandler>) }).on_fullscreen_change(fullscreen);
    }

    extern "C" fn on_message_callback(message: *const c_char, ctx: *mut c_void) {
        if !message.is_null() {
            if let Ok(message) = unsafe { CStr::from_ptr(message) }.to_str() {
                (unsafe { &*(ctx as *mut Box<dyn WebViewHandler>) }).on_message(message);
            }
        }
    }
}

impl Drop for WebView {
    fn drop(&mut self) {
        unsafe {
            sys::close_page(self.raw.as_ptr());
        }

        drop(unsafe { Box::from_raw(self.handler.as_ptr()) });
    }
}
