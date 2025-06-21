use std::{
    ffi::{CStr, CString, c_char, c_int, c_void},
    marker::PhantomData,
    num::NonZeroIsize,
    ops::Deref,
    ptr::{NonNull, null},
};

use parking_lot::Mutex;
use raw_window_handle::{AppKitWindowHandle, RawWindowHandle, Win32WindowHandle};

use crate::{Error, ThreadSafePointer, WindowlessRenderWebView, sys};

pub use self::sys::WebViewState;

/// Represents a position
///
/// This is mainly used for mouse and touch events
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Represents the state of a key event
///
/// This is mainly used for keyboard events and mouse events, and also for touch
/// events
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum KeyState {
    Down,
    Up,
}

/// Represents a mouse button
///
/// This is mainly used for mouse events
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// Represents a rectangular area
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Represents a mouse event
///
/// This is mainly used for mouse events
#[derive(Debug, Clone)]
pub enum MouseAction {
    /// Click a mouse button
    ///
    /// Position is optional, if position is None, it means the mouse is at the
    /// current position
    Click(MouseButton, KeyState, Option<Position>),
    /// Move the mouse
    Move(Position),
    /// Scroll the mouse wheel
    Wheel(Position),
}

/// Represents an IME event
///
/// This is mainly used for IME events
#[derive(Debug)]
pub enum IMEAction<'a> {
    Composition(&'a str),
    Pre(&'a str, i32, i32),
}

/// Represents modifier keys
///
/// This is mainly used for keyboard events
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Modifiers {
    Shift,
    Ctrl,
    Alt,
    Win,
}

/// Represents the type of key event
///
/// This is mainly used for keyboard events
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum KeyEventType {
    RawKeyDown,
    KeyDown,
    KeyUp,
    Char,
}

/// Represents a key event
///
/// This is mainly used for keyboard events
#[derive(Debug, Copy, Clone)]
pub struct KeyEvent {
    pub size: usize,
    pub ty: KeyEventType,
    pub modifiers: u32,
    pub windows_key_code: u32,
    pub native_key_code: u32,
    pub is_system_key: u32,
    pub character: u16,
    pub unmodified_character: u16,
    pub focus_on_editable_field: u32,
}

#[allow(unused)]
pub trait WebViewHandler: Send + Sync {
    /// Called when the web page state changes
    ///
    /// This callback is called when the web page state changes.
    fn on_state_change(&self, state: WebViewState) {}

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

#[allow(unused)]
pub trait WindowlessRenderWebViewHandler: WebViewHandler {
    /// Called when the IME composition rectangle changes
    ///
    /// This callback is called when the IME composition rectangle changes.
    fn on_ime_rect(&self, rect: Rect) {}

    /// Push a new frame when rendering changes
    ///
    /// This only works in windowless rendering mode.
    fn on_frame(&self, texture: &[u8], width: u32, height: u32) {}
}

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
    /// In windowed mode, setting the window handle will set the browser as a
    /// child view.
    ///
    /// In windowless mode, setting the window handle is used to identify
    /// monitor information and as a parent view for dialog boxes, context
    /// menus, and other elements. If not provided, the main screen monitor will
    /// be used, and some features that require a parent view may not work
    /// properly.
    pub fn with_window_handle(mut self, value: RawWindowHandle) -> Self {
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

    /// Set whether JavaScript is enabled
    ///
    /// This function is used to set whether JavaScript is enabled.
    pub fn with_javascript_enable(mut self, value: bool) -> Self {
        self.0.javascript_enable = value;
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

pub struct WebView<W> {
    _w: PhantomData<W>,
    mouse_event: Mutex<sys::cef_mouse_event_t>,
    handler: ThreadSafePointer<MixWebviewHnadler>,
    raw: Mutex<ThreadSafePointer<c_void>>,
}

impl<W> WebView<W> {
    pub(crate) fn new(
        runtime: &ThreadSafePointer<c_void>,
        url: &str,
        attr: &WebViewAttributes,
        handler: MixWebviewHnadler,
    ) -> Result<Self, Error> {
        let options = sys::WebViewSettings {
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
                    _ => unimplemented!("Unsupported window handle type: {:?}", it),
                }
            } else {
                null()
            },
        };

        let url = CString::new(url).unwrap();
        let handler: *mut MixWebviewHnadler = Box::into_raw(Box::new(handler));
        let ptr = unsafe {
            sys::create_webview(
                runtime.as_ptr(),
                url.as_c_str().as_ptr(),
                &options,
                sys::WebViewHandler {
                    on_state_change: Some(on_state_change_callback),
                    on_ime_rect: Some(on_ime_rect_callback),
                    on_frame: Some(on_frame_callback),
                    on_title_change: Some(on_title_change_callback),
                    on_fullscreen_change: Some(on_fullscreen_change_callback),
                    on_message: Some(on_message_callback),
                    context: handler as _,
                },
            )
        };

        let raw = if ptr.is_null() {
            return Err(Error::FailedToCreateWebView);
        } else {
            ThreadSafePointer(ptr)
        };

        Ok(Self {
            _w: PhantomData::default(),
            mouse_event: Mutex::new(unsafe { std::mem::zeroed() }),
            handler: ThreadSafePointer(handler),
            raw: Mutex::new(raw),
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
            sys::webview_send_message(self.raw.lock().as_ptr(), message.as_c_str().as_ptr());
        }
    }

    /// Get the window handle
    ///
    /// This function is used to get the window handle.
    pub fn window_handle(&self) -> RawWindowHandle {
        let handle = unsafe { sys::webview_get_window_handle(self.raw.lock().as_ptr()) };
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
        unsafe { sys::webview_set_devtools_state(self.raw.lock().as_ptr(), enable) }
    }
}

impl WebView<WindowlessRenderWebView> {
    /// Send a mouse event
    ///
    /// This function is used to send mouse events.
    ///
    /// Note that this function only works in windowless rendering mode.
    pub fn mouse(&self, action: MouseAction) {
        let mut event = self.mouse_event.lock();

        match action {
            MouseAction::Move(pos) => unsafe {
                event.x = pos.x;
                event.y = pos.y;

                sys::webview_mouse_move(self.raw.lock().as_ptr(), *event)
            },
            MouseAction::Wheel(pos) => unsafe {
                sys::webview_mouse_wheel(self.raw.lock().as_ptr(), *event, pos.x, pos.y)
            },
            MouseAction::Click(button, state, pos) => {
                if let Some(pos) = pos {
                    event.x = pos.x;
                    event.y = pos.y;
                }

                unsafe {
                    sys::webview_mouse_click(
                        self.raw.lock().as_ptr(),
                        *event,
                        match button {
                            MouseButton::Left => sys::cef_mouse_button_type_t::MBT_LEFT,
                            MouseButton::Middle => sys::cef_mouse_button_type_t::MBT_MIDDLE,
                            MouseButton::Right => sys::cef_mouse_button_type_t::MBT_RIGHT,
                        },
                        state == KeyState::Down,
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
    pub fn keyboard(&self, event: KeyEvent) {
        unsafe {
            sys::webview_keyboard(
                self.raw.lock().as_ptr(),
                sys::cef_key_event_t {
                    size: event.size,
                    modifiers: event.modifiers,
                    windows_key_code: event.windows_key_code as i32,
                    native_key_code: event.native_key_code as i32,
                    is_system_key: event.is_system_key as i32,
                    character: event.character,
                    unmodified_character: event.unmodified_character,
                    focus_on_editable_field: event.focus_on_editable_field as i32,
                    type_: match event.ty {
                        KeyEventType::RawKeyDown => sys::cef_key_event_type_t::KEYEVENT_RAWKEYDOWN,
                        KeyEventType::KeyDown => sys::cef_key_event_type_t::KEYEVENT_KEYDOWN,
                        KeyEventType::KeyUp => sys::cef_key_event_type_t::KEYEVENT_KEYUP,
                        KeyEventType::Char => sys::cef_key_event_type_t::KEYEVENT_CHAR,
                    },
                },
            )
        }
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
                sys::webview_ime_composition(self.raw.lock().as_ptr(), input.as_c_str().as_ptr())
            },
            IMEAction::Pre(_, x, y) => unsafe {
                sys::webview_ime_set_composition(
                    self.raw.lock().as_ptr(),
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
        unsafe { sys::webview_resize(self.raw.lock().as_ptr(), width as c_int, height as c_int) }
    }
}

impl<W> Drop for WebView<W> {
    fn drop(&mut self) {
        unsafe {
            sys::close_webview(self.raw.lock().as_ptr());
        }

        drop(unsafe { Box::from_raw(self.handler.as_ptr()) });
    }
}

pub(crate) enum MixWebviewHnadler {
    WebViewHandler(Box<dyn WebViewHandler>),
    WindowlessRenderWebViewHandler(Box<dyn WindowlessRenderWebViewHandler>),
}

extern "C" fn on_state_change_callback(state: sys::WebViewState, context: *mut c_void) {
    if context.is_null() {
        return;
    }

    match unsafe { &*(context as *mut MixWebviewHnadler) } {
        MixWebviewHnadler::WebViewHandler(handler) => handler.on_state_change(state),
        MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) => {
            handler.on_state_change(state)
        }
    }
}

extern "C" fn on_ime_rect_callback(rect: sys::cef_rect_t, context: *mut c_void) {
    if context.is_null() {
        return;
    }

    if let MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) =
        unsafe { &*(context as *mut MixWebviewHnadler) }
    {
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
    width: c_int,
    height: c_int,
    context: *mut c_void,
) {
    if context.is_null() {
        return;
    }

    if let MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) =
        unsafe { &*(context as *mut MixWebviewHnadler) }
    {
        handler.on_frame(
            unsafe {
                std::slice::from_raw_parts(texture as _, width as usize * height as usize * 4)
            },
            width as u32,
            height as u32,
        )
    }
}

extern "C" fn on_title_change_callback(title: *const c_char, context: *mut c_void) {
    if context.is_null() || title.is_null() {
        return;
    }

    if let Ok(title) = unsafe { CStr::from_ptr(title) }.to_str() {
        match unsafe { &*(context as *mut MixWebviewHnadler) } {
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

    match unsafe { &*(context as *mut MixWebviewHnadler) } {
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

    if let Ok(message) = unsafe { CStr::from_ptr(message) }.to_str() {
        match unsafe { &*(context as *mut MixWebviewHnadler) } {
            MixWebviewHnadler::WebViewHandler(handler) => handler.on_message(message),
            MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) => {
                handler.on_message(message)
            }
        }
    }
}
