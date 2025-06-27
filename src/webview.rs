//!
//! ## Communication with Web Pages
//!
//! This library's runtime will inject a global object into web pages for
//! communication between Rust and web pages.
//!
//! ```typescript
//! declare global {
//!     interface Window {
//!         MessageTransport: {
//!             on: (handle: (message: string) => void) => void;
//!             send: (message: string) => void;
//!         };
//!     }
//! }
//! ```
//!
//! Usage example:
//!
//! ```typescript
//! window.MessageTransport.on((message: string) => {
//!     console.log("Received message from Rust:", message);
//! });
//!
//! window.MessageTransport.send("Send message to Rust");
//! ```
//!
//! `WebViewHandler::on_message` is used to receive messages sent by
//! `MessageTransport.send`, while `MessageTransport.on` is used to receive
//! messages sent by `WebView::send_message`. Sending and receiving messages are
//! full-duplex and asynchronous.

use std::{
    ffi::{CStr, CString, c_char, c_int, c_void},
    marker::PhantomData,
    ops::Deref,
    ptr::null,
    sync::Arc,
};

use parking_lot::Mutex;

use crate::{
    Error, Rect, WindowlessRenderWebView,
    events::{
        IMEAction, KeyboardEvent, KeyboardEventType, KeyboardModifiers, MouseButton, MouseEvent,
    },
    request::{CustomRequestHandlerFactory, ICustomRequestHandlerFactory},
    runtime::{IRuntime, Runtime},
    sys,
    utils::{AnySrtingCastRaw, GetSharedRef, ThreadSafePointer},
};

/// Represents the state of a web page
///
/// The order of events is as follows:
///
/// ```text
/// BeforeLoad -> Loaded -> RequestClose -> Close
///          \ -> LoadError -> Loaded -> RequestClose -> Close
/// ```
///
/// Regardless of whether the loading exists an error, the `Loaded` event is
/// triggered, the difference is that if the loading error occurs, the
/// `LoadError` event is triggered first.
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum WebViewState {
    /// The web page is before loading
    BeforeLoad = 1,
    /// The web page is loaded
    Loaded = 2,
    /// The web page is loading error
    LoadError = 3,
    /// The web page is requesting to close
    RequestClose = 4,
    /// The web page is closed
    Close = 5,
}

/// Represents a window handle
pub struct WindowHandle(ThreadSafePointer<c_void>);

impl WindowHandle {
    /// Create a new window handle
    pub fn new(value: *const c_void) -> Self {
        WindowHandle(ThreadSafePointer::new(value as _))
    }

    /// Get the raw pointer of the window handle
    pub fn as_ptr(&self) -> *const c_void {
        self.0.as_ptr()
    }
}

/// WebView handler
///
/// This trait is used to handle web view events.
#[allow(unused)]
pub trait WebViewHandler: Send + Sync {
    /// Called when the web page state changes
    ///
    /// You need to pay attention to status changes, determine whether loading
    /// was successful, and monitor events related to the page closing.
    fn on_state_change(&self, state: WebViewState) {}

    /// Called when the title changes
    fn on_title_change(&self, title: &str) {}

    /// Called when the fullscreen state changes
    fn on_fullscreen_change(&self, fullscreen: bool) {}

    /// Called when a message is received
    ///
    /// This callback is called when a message is received from the web page.
    fn on_message(&self, message: &str) {}
}

/// Windowless render web view handler
///
/// A specific event handler for windowless rendering WebView.
#[allow(unused)]
pub trait WindowlessRenderWebViewHandler: WebViewHandler {
    /// Called when the IME composition rectangle changes
    ///
    /// When the IME region changes, you should notify the external window.
    fn on_ime_rect(&self, rect: Rect) {}

    /// Push a new frame when rendering changes
    ///
    /// This only works in windowless rendering mode.
    ///
    /// #### Note:
    ///
    /// Fixed as BGRA texture buffer, not padded and not aligned.
    ///
    /// It should be noted that if the webview is resized, the width and height
    /// of the texture will also change.
    fn on_frame(&self, texture: &[u8], rect: Rect) {}
}

/// WebView configuration attributes
pub struct WebViewAttributes {
    /// Request handler factory.
    pub request_handler_factory: Option<CustomRequestHandlerFactory>,
    /// External native window handle.
    pub window_handle: Option<WindowHandle>,
    /// The maximum rate in frames per second (fps).
    pub windowless_frame_rate: u32,
    /// window size width.
    pub width: u32,
    /// window size height.
    pub height: u32,
    /// window device scale factor.
    pub device_scale_factor: f32,
    /// page defalt font size.
    pub default_font_size: u32,
    /// page defalt fixed font size.
    pub default_fixed_font_size: u32,
    /// The minimum font size.
    pub minimum_font_size: u32,
    /// The minimum logical font size.
    pub minimum_logical_font_size: u32,
    /// Controls whether WebGL is enabled.
    pub webgl: bool,
    /// Controls whether databases are enabled.
    pub databases: bool,
    /// Controls whether JavaScript can be executed.
    pub javascript: bool,
    /// Controls whether JavaScript can access the clipboard.
    pub javascript_access_clipboard: bool,
    /// Controls whether JavaScript can be used to close windows that were not
    /// opened via JavaScript.
    pub javascript_close_windows: bool,
    /// Controls whether DOM pasting is supported in the editor via
    /// execCommand("paste").
    pub javascript_dom_paste: bool,
    /// Controls whether local storage can be used.
    pub local_storage: bool,
    /// END values that map to WebPreferences settings.
    pub background_color: u32,
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
            javascript: true,
            local_storage: true,
            javascript_access_clipboard: false,
            request_handler_factory: None,
            webgl: false,
            databases: false,
            javascript_close_windows: false,
            javascript_dom_paste: false,
            background_color: 0xFFFFFFFF,
            minimum_font_size: 12,
            minimum_logical_font_size: 12,
        }
    }
}

/// WebView configuration attributes builder
#[derive(Default)]
pub struct WebViewAttributesBuilder(WebViewAttributes);

impl WebViewAttributesBuilder {
    /// Set the request handler factory
    ///
    /// This function is used to set the request handler factory.
    pub fn with_request_handler_factory(mut self, value: CustomRequestHandlerFactory) -> Self {
        self.0.request_handler_factory = Some(value);
        self
    }

    /// Set the window handle
    ///
    /// In windowed mode, setting the window handle will set the browser as a
    /// child view.
    ///
    /// In windowless mode, setting the window handle is used to identify
    /// monitor information and as a parent view for dialog boxes, context
    /// menus, and other elements. If not provided, the main screen monitor will
    /// be used, and some features that require a parent view may not work
    /// properly.
    pub fn with_window_handle(mut self, value: WindowHandle) -> Self {
        self.0.window_handle = Some(value);
        self
    }

    /// Set the frame rate in windowless rendering mode
    ///
    /// This function is used to set the frame rate in windowless rendering
    /// mode.
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

    /// Set the minimum font size
    ///
    /// This function is used to set the minimum font size.
    pub fn with_minimum_font_size(mut self, value: u32) -> Self {
        self.0.minimum_font_size = value;
        self
    }

    /// Set the minimum logical font size
    ///
    /// This function is used to set the minimum logical font size.
    pub fn with_minimum_logical_font_size(mut self, value: u32) -> Self {
        self.0.minimum_logical_font_size = value;
        self
    }

    /// Set whether local storage is enabled
    ///
    /// This function is used to set whether local storage is enabled.
    pub fn with_local_storage(mut self, value: bool) -> Self {
        self.0.local_storage = value;
        self
    }

    /// Set whether WebGL is enabled
    ///
    /// This function is used to set whether WebGL is enabled.
    pub fn with_webgl(mut self, value: bool) -> Self {
        self.0.webgl = value;
        self
    }

    /// Set whether databases are enabled
    ///
    /// This function is used to set whether databases are enabled.
    pub fn with_databases(mut self, value: bool) -> Self {
        self.0.databases = value;
        self
    }

    /// Set whether JavaScript is enabled
    ///
    /// This function is used to set whether JavaScript is enabled.
    pub fn with_javascript(mut self, value: bool) -> Self {
        self.0.javascript = value;
        self
    }

    /// Set whether JavaScript can access the clipboard
    ///
    /// This function is used to set whether JavaScript can access the
    /// clipboard.
    pub fn with_javascript_access_clipboard(mut self, value: bool) -> Self {
        self.0.javascript_access_clipboard = value;
        self
    }

    /// Set whether JavaScript can be used to close windows that were not opened
    /// via JavaScript.
    ///
    /// This function is used to set whether JavaScript can be used to close
    /// windows that were not opened via JavaScript.
    pub fn with_javascript_close_windows(mut self, value: bool) -> Self {
        self.0.javascript_close_windows = value;
        self
    }

    /// Set whether DOM pasting is supported in the editor via
    /// execCommand("paste").
    ///
    /// This function is used to set whether DOM pasting is supported in the
    /// editor via execCommand("paste").
    pub fn with_javascript_dom_paste(mut self, value: bool) -> Self {
        self.0.javascript_dom_paste = value;
        self
    }

    /// Set the background color
    ///
    /// This function is used to set the background color.
    pub fn with_background_color(mut self, value: u32) -> Self {
        self.0.background_color = value;
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

pub(crate) struct IWebView {
    mouse_event: Mutex<sys::MouseEvent>,
    // The runtime may use a custom request interceptor; a reference is kept here to ensure correct
    // lifetime management.
    #[allow(unused)]
    request_handler_factory: Option<Arc<ICustomRequestHandlerFactory>>,
    context: ThreadSafePointer<WebViewContext>,
    raw: Mutex<ThreadSafePointer<c_void>>,
}

impl IWebView {
    fn new<R, W>(
        runtime: &Runtime<R, W>,
        url: &str,
        attr: WebViewAttributes,
        handler: MixWebviewHnadler,
    ) -> Result<Self, Error> {
        let runtime = runtime.get_shared_ref();
        let raw_runtime = runtime.get_raw();

        let options = sys::WebViewSettings {
            width: attr.width,
            height: attr.height,
            webgl: attr.webgl,
            databases: attr.databases,
            local_storage: attr.local_storage,
            background_color: attr.background_color,
            javascript: attr.javascript,
            javascript_access_clipboard: attr.javascript_access_clipboard,
            javascript_close_windows: attr.javascript_close_windows,
            javascript_dom_paste: attr.javascript_dom_paste,
            minimum_font_size: attr.minimum_font_size as _,
            minimum_logical_font_size: attr.minimum_logical_font_size as _,
            device_scale_factor: attr.device_scale_factor,
            windowless_frame_rate: attr.windowless_frame_rate,
            default_fixed_font_size: attr.default_fixed_font_size as _,
            default_font_size: attr.default_font_size as _,
            window_handle: if let Some(it) = &attr.window_handle {
                it.as_ptr()
            } else {
                null()
            },
            request_handler_factory: if let Some(it) = &attr.request_handler_factory {
                it.as_raw().as_ptr() as _
            } else {
                null()
            },
        };

        let context: *mut WebViewContext = Box::into_raw(Box::new(WebViewContext {
            runtime: Some(runtime),
            handler,
        }));

        let url = CString::new(url).unwrap();
        let ptr = unsafe {
            sys::create_webview(
                raw_runtime.as_ptr(),
                url.as_raw(),
                &options,
                sys::WebViewHandler {
                    on_state_change: Some(on_state_change_callback),
                    on_ime_rect: Some(on_ime_rect_callback),
                    on_frame: Some(on_frame_callback),
                    on_title_change: Some(on_title_change_callback),
                    on_fullscreen_change: Some(on_fullscreen_change_callback),
                    on_message: Some(on_message_callback),
                    context: context as _,
                },
            )
        };

        let raw = if ptr.is_null() {
            return Err(Error::FailedToCreateWebView);
        } else {
            ThreadSafePointer::new(ptr)
        };

        Ok(Self {
            raw: Mutex::new(raw),
            context: ThreadSafePointer::new(context),
            mouse_event: Mutex::new(unsafe { std::mem::zeroed() }),
            request_handler_factory: if let Some(it) = &attr.request_handler_factory {
                Some(it.get_shared_ref())
            } else {
                None
            },
        })
    }
}

impl Drop for IWebView {
    fn drop(&mut self) {
        unsafe {
            sys::close_webview(self.raw.lock().as_ptr());
        }

        drop(unsafe { Box::from_raw(self.context.as_ptr()) });
    }
}

/// Represents an opened web page
#[allow(unused)]
pub struct WebView<W> {
    _w: PhantomData<W>,
    inner: Arc<IWebView>,
}

impl<W> GetSharedRef for WebView<W> {
    type Ref = Arc<IWebView>;

    fn get_shared_ref(&self) -> Self::Ref {
        self.inner.clone()
    }
}

impl<W> WebView<W> {
    pub(crate) fn new<R>(
        runtime: &Runtime<R, W>,
        url: &str,
        attr: WebViewAttributes,
        handler: MixWebviewHnadler,
    ) -> Result<Self, Error> {
        Ok(Self {
            _w: PhantomData,
            inner: Arc::new(IWebView::new(runtime, url, attr, handler)?),
        })
    }

    /// Send a message
    ///
    /// This function is used to send a message to the web page.
    ///
    /// Messages sent from the web page are received through the
    /// `WebViewHandler::on_message` callback.
    pub fn send_message(&self, message: &str) {
        let message = CString::new(message).unwrap();

        unsafe {
            sys::webview_send_message(self.inner.raw.lock().as_ptr(), message.as_raw());
        }
    }

    /// Get the window handle
    ///
    /// This function is used to get the window handle.
    pub fn window_handle(&self) -> Option<WindowHandle> {
        let handle = unsafe { sys::webview_get_window_handle(self.inner.raw.lock().as_ptr()) };
        if !handle.is_null() {
            Some(WindowHandle::new(handle))
        } else {
            None
        }
    }

    /// Set whether developer tools are enabled
    ///
    /// This function is used to set whether developer tools are enabled.
    pub fn devtools_enabled(&self, enable: bool) {
        unsafe { sys::webview_set_devtools_state(self.inner.raw.lock().as_ptr(), enable) }
    }
}

impl WebView<WindowlessRenderWebView> {
    /// Send a mouse event
    ///
    /// This function is used to send mouse events.
    ///
    /// Note that this function only works in windowless rendering mode.
    pub fn mouse(&self, action: &MouseEvent) {
        let mut event = self.inner.mouse_event.lock();

        match action {
            MouseEvent::Move(pos) => unsafe {
                event.x = pos.x;
                event.y = pos.y;

                sys::webview_mouse_move(self.inner.raw.lock().as_ptr(), *event)
            },
            MouseEvent::Wheel(pos) => unsafe {
                sys::webview_mouse_wheel(self.inner.raw.lock().as_ptr(), *event, pos.x, pos.y)
            },
            MouseEvent::Click(button, is_pressed, pos) => {
                if let Some(pos) = pos {
                    event.x = pos.x;
                    event.y = pos.y;
                }

                unsafe {
                    sys::webview_mouse_click(
                        self.inner.raw.lock().as_ptr(),
                        *event,
                        (*button).into(),
                        *is_pressed,
                    )
                }
            }
        }
    }

    /// Send a keyboard event
    ///
    /// This function is used to send keyboard events.
    ///
    /// Note that this function only works in windowless rendering mode.
    pub fn keyboard(&self, event: &KeyboardEvent) {
        let mut modifiers = sys::EventFlags::WEBVIEW_EVENTFLAG_NONE as u32;
        for it in KeyboardModifiers::all() {
            if event.modifiers.contains(it) {
                let flag: sys::EventFlags = it.into();
                modifiers |= flag as u32;
            }
        }

        unsafe {
            sys::webview_keyboard(
                self.inner.raw.lock().as_ptr(),
                sys::KeyEvent {
                    modifiers,
                    character: event.character,
                    unmodified_character: event.unmodified_character,
                    windows_key_code: event.windows_key_code as i32,
                    native_key_code: event.native_key_code as i32,
                    is_system_key: event.is_system_key as i32,
                    focus_on_editable_field: event.focus_on_editable_field as i32,
                    type_: event.ty.into(),
                },
            )
        }
    }

    /// Send an IME event
    ///
    /// This function is used to send IME events.
    ///
    /// Note that this function only works in windowless rendering mode.
    pub fn ime(&self, action: &IMEAction) {
        let input = match action {
            IMEAction::Composition(it) | IMEAction::Pre(it, _, _) => CString::new(*it).unwrap(),
        };

        match action {
            IMEAction::Composition(_) => unsafe {
                sys::webview_ime_composition(self.inner.raw.lock().as_ptr(), input.as_raw())
            },
            IMEAction::Pre(_, x, y) => unsafe {
                sys::webview_ime_set_composition(
                    self.inner.raw.lock().as_ptr(),
                    input.as_raw(),
                    *x,
                    *y,
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
        unsafe {
            sys::webview_resize(
                self.inner.raw.lock().as_ptr(),
                width as c_int,
                height as c_int,
            )
        }
    }

    /// Set the focus state
    ///
    /// This function is used to set the focus state.
    ///
    /// Note that this function only works in windowless rendering mode.
    pub fn focus(&self, state: bool) {
        unsafe { sys::webview_set_focus(self.inner.raw.lock().as_ptr(), state) }
    }
}

impl From<sys::WebViewState> for WebViewState {
    fn from(value: sys::WebViewState) -> Self {
        use sys::WebViewState;

        match value {
            WebViewState::WEBVIEW_BEFORE_LOAD => Self::BeforeLoad,
            WebViewState::WEBVIEW_LOADED => Self::Loaded,
            WebViewState::WEBVIEW_LOAD_ERROR => Self::LoadError,
            WebViewState::WEBVIEW_REQUEST_CLOSE => Self::RequestClose,
            WebViewState::WEBVIEW_CLOSE => Self::Close,
        }
    }
}

impl Into<sys::KeyEventType> for KeyboardEventType {
    fn into(self) -> sys::KeyEventType {
        use sys::KeyEventType;

        match self {
            Self::KeyDown => KeyEventType::WEBVIEW_KEYEVENT_KEYDOWN,
            Self::KeyUp => KeyEventType::WEBVIEW_KEYEVENT_KEYUP,
            Self::Char => KeyEventType::WEBVIEW_KEYEVENT_CHAR,
        }
    }
}

impl Into<sys::EventFlags> for KeyboardModifiers {
    fn into(self) -> sys::EventFlags {
        use sys::EventFlags;

        match self {
            Self::None => EventFlags::WEBVIEW_EVENTFLAG_NONE,
            Self::Win => EventFlags::WEBVIEW_EVENTFLAG_COMMAND_DOWN,
            Self::Shift => EventFlags::WEBVIEW_EVENTFLAG_SHIFT_DOWN,
            Self::Ctrl => EventFlags::WEBVIEW_EVENTFLAG_CONTROL_DOWN,
            Self::Alt => EventFlags::WEBVIEW_EVENTFLAG_ALT_DOWN,
            Self::Command => EventFlags::WEBVIEW_EVENTFLAG_COMMAND_DOWN,
            Self::CapsLock => EventFlags::WEBVIEW_EVENTFLAG_CAPS_LOCK_ON,
            _ => EventFlags::WEBVIEW_EVENTFLAG_NONE,
        }
    }
}

impl Into<sys::MouseButton> for MouseButton {
    fn into(self) -> sys::MouseButton {
        use sys::MouseButton;

        match self {
            Self::Left => MouseButton::WEBVIEW_MBT_LEFT,
            Self::Middle => MouseButton::WEBVIEW_MBT_MIDDLE,
            Self::Right => MouseButton::WEBVIEW_MBT_RIGHT,
        }
    }
}

struct WebViewContext {
    runtime: Option<Arc<IRuntime>>,
    handler: MixWebviewHnadler,
}

pub(crate) enum MixWebviewHnadler {
    WebViewHandler(Box<dyn WebViewHandler>),
    WindowlessRenderWebViewHandler(Box<dyn WindowlessRenderWebViewHandler>),
}

extern "C" fn on_state_change_callback(state: sys::WebViewState, context: *mut c_void) {
    if context.is_null() {
        return;
    }

    let state = WebViewState::from(state);
    let context = unsafe { &mut *(context as *mut WebViewContext) };

    // Only after all webviews are closed can the runtime be closed. Here, we clear
    // the reference held by the current webview.
    //
    // If all webviews are closed, the runtime reference will be cleared,
    // and only then will the runtime's Drop be triggered.
    if state == WebViewState::Close {
        drop(context.runtime.take());
    }

    match &context.handler {
        MixWebviewHnadler::WebViewHandler(handler) => handler.on_state_change(state),
        MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) => {
            handler.on_state_change(state)
        }
    }
}

extern "C" fn on_ime_rect_callback(rect: sys::Rect, context: *mut c_void) {
    if context.is_null() {
        return;
    }

    let context = unsafe { &*(context as *mut WebViewContext) };

    if let MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) = &context.handler {
        handler.on_ime_rect(Rect {
            x: rect.x as u32,
            y: rect.y as u32,
            width: rect.width as u32,
            height: rect.height as u32,
        })
    }
}

extern "C" fn on_frame_callback(
    texture: *const c_void,
    rect: *mut sys::Rect,
    context: *mut c_void,
) {
    if context.is_null() {
        return;
    }

    let rect = {
        let value = unsafe { &*rect };

        Rect {
            x: value.x as u32,
            y: value.y as u32,
            width: value.width as u32,
            height: value.height as u32,
        }
    };

    let context = unsafe { &*(context as *mut WebViewContext) };

    if let MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) = &context.handler {
        handler.on_frame(
            // Fixed as BGRA texture buffer, not padded and not aligned.
            unsafe {
                std::slice::from_raw_parts(
                    texture as _,
                    rect.width as usize * rect.height as usize * 4,
                )
            },
            rect,
        )
    }
}

extern "C" fn on_title_change_callback(title: *const c_char, context: *mut c_void) {
    if context.is_null() || title.is_null() {
        return;
    }

    let context = unsafe { &*(context as *mut WebViewContext) };

    if let Ok(title) = unsafe { CStr::from_ptr(title) }.to_str() {
        match &context.handler {
            MixWebviewHnadler::WebViewHandler(handler) => handler.on_title_change(title),
            MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) => {
                handler.on_title_change(title)
            }
        }
    }
}
extern "C" fn on_fullscreen_change_callback(fullscreen: bool, context: *mut c_void) {
    if context.is_null() {
        return;
    }

    let context = unsafe { &*(context as *mut WebViewContext) };

    match &context.handler {
        MixWebviewHnadler::WebViewHandler(handler) => handler.on_fullscreen_change(fullscreen),
        MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) => {
            handler.on_fullscreen_change(fullscreen)
        }
    }
}

extern "C" fn on_message_callback(message: *const c_char, context: *mut c_void) {
    if context.is_null() || message.is_null() {
        return;
    }

    let context = unsafe { &*(context as *mut WebViewContext) };

    if let Ok(message) = unsafe { CStr::from_ptr(message) }.to_str() {
        match &context.handler {
            MixWebviewHnadler::WebViewHandler(handler) => handler.on_message(message),
            MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) => {
                handler.on_message(message)
            }
        }
    }
}
