use std::{
    env::args,
    ffi::{c_char, c_int, c_void},
    num::NonZeroIsize,
    ptr::null_mut,
    slice::from_raw_parts,
    sync::{Arc, RwLock},
};

use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    oneshot::Sender,
};

use webview_sys::{
    create_page, create_webview, page_exit, page_get_hwnd, page_resize, page_send_ime_composition,
    page_send_ime_set_composition, page_send_keyboard, page_send_mouse_click,
    page_send_mouse_click_with_pos, page_send_mouse_move, page_send_mouse_wheel, page_send_touch,
    page_set_devtools_state, webview_exit, webview_run, Modifiers, PageState, Rect, TouchEventType,
    TouchPointerType,
};

use crate::{
    page::PageOptions,
    strings::{ffi, StringConvert},
    ActionState, ImeAction, MouseAction, WebviewOptions,
};

#[inline]
fn get_args() -> Vec<*const c_char> {
    args()
        .map(|arg| arg.as_pstr())
        .collect::<Vec<_>>()
        .iter()
        .map(|arg| arg.0)
        .collect()
}

/// CefApp
///
/// The CefApp interface provides access to process-specific callbacks.
/// Important callbacks include:
///
/// OnBeforeCommandLineProcessing which provides the opportunity to
/// programmatically set command-line arguments. See the “Command Line
/// Arguments” section for more information.
///
/// OnRegisterCustomSchemes which provides an opportunity to register custom
/// schemes. See the “”Request Handling” section for more information.
///
/// GetBrowserProcessHandler which returns the handler for functionality
/// specific to the browser process including the OnContextInitialized() method.
///
/// GetRenderProcessHandler which returns the handler for functionality specific
/// to the render process. This includes JavaScript-related callbacks and
/// process messages. See the JavaScriptIntegration Wiki page and the
/// “Inter-Process Communication” section for more information.
///
/// An example CefApp implementation can be seen in cefsimple/simple_app.h and
/// cefsimple/simple_app.cc.
pub(crate) struct WebviewWrapper {
    options: webview_sys::WebviewOptions,
    raw: *mut c_void,
}

unsafe impl Send for WebviewWrapper {}
unsafe impl Sync for WebviewWrapper {}

impl WebviewWrapper {
    extern "C" fn callback(ctx: *mut c_void) {
        if let Err(e) = unsafe { Box::from_raw(ctx as *mut Sender<()>) }.send(()) {
            log::error!(
                "An error occurred when webview pushed a message to the callback. error={:?}",
                e
            );
        }
    }

    pub(crate) fn new(options: &WebviewOptions, tx: Sender<()>) -> Option<Self> {
        let options = webview_sys::WebviewOptions {
            cache_path: ffi::into_opt(options.cache_path) as _,
            scheme_path: ffi::into_opt(options.scheme_path) as _,
            browser_subprocess_path: ffi::into_opt(options.browser_subprocess_path) as _,
        };

        let raw = unsafe {
            create_webview(
                &options as *const _ as _,
                Some(Self::callback),
                Box::into_raw(Box::new(tx)) as *mut _,
            )
        };

        if raw.is_null() {
            return None;
        }

        Some(Self { options, raw })
    }

    /// Create a new browser using the window parameters specified by
    /// |windowInfo|.
    ///
    /// All values will be copied internally and the actual window (if any) will
    /// be created on the UI thread. If |request_context| is empty the global
    /// request context will be used. This method can be called on any browser
    /// process thread and will not block. The optional |extra_info| parameter
    /// provides an opportunity to specify extra information specific to the
    /// created browser that will be passed to
    /// CefRenderProcessHandler::OnBrowserCreated() in the render process.
    pub(crate) fn create_page<T>(
        &self,
        options: &PageOptions<'_>,
        observer: T,
    ) -> (PageWrapper, UnboundedReceiver<ChannelEvents>)
    where
        T: Observer + 'static,
    {
        PageWrapper::new(&self, options, observer)
    }

    pub(crate) fn run(&self) {
        let args = get_args();
        if unsafe { webview_run(self.raw, args.len() as _, args.as_ptr() as _) } != 0 {
            panic!("Webview exited unexpectedly, this is a bug.")
        }
    }
}

impl Drop for WebviewWrapper {
    fn drop(&mut self) {
        unsafe {
            webview_exit(self.raw);
        }

        {
            ffi::free(self.options.browser_subprocess_path);
            ffi::free(self.options.cache_path);
            ffi::free(self.options.scheme_path);
        }
    }
}

/// CefClient
///
/// The CefClient interface provides access to browser-instance-specific
/// callbacks. A single CefClient instance can be shared among any number of
/// browsers. Important callbacks include:
///
/// Handlers for things like browser life span, context menus, dialogs, display
/// notifications, drag events, focus events, keyboard events and more. The
/// majority of handlers are optional. See the class documentation for the side
/// effects, if any, of not implementing a specific handler.
///
/// OnProcessMessageReceived which is called when an IPC message is received
/// from the render process. See the “Inter-Process Communication” section for
/// more information.
///
/// An example CefClient implementation can be seen in
/// cefsimple/simple_handler.h and cefsimple/simple_handler.cc.
pub(crate) struct PageWrapper {
    options: webview_sys::PageOptions,
    pub observer: ObserverWrapper,
    pub raw: *mut c_void,
}

unsafe impl Send for PageWrapper {}
unsafe impl Sync for PageWrapper {}

impl PageWrapper {
    fn new<T>(
        webview: &WebviewWrapper,
        options: &PageOptions<'_>,
        observer: T,
    ) -> (Self, UnboundedReceiver<ChannelEvents>)
    where
        T: Observer + 'static,
    {
        let options = webview_sys::PageOptions {
            url: ffi::into(options.url) as _,
            frame_rate: options.frame_rate,
            width: options.width,
            height: options.height,
            device_scale_factor: options.device_scale_factor,
            is_offscreen: options.is_offscreen,
            window_handle: if let Some(it) = options.window_handle {
                match it {
                    RawWindowHandle::Win32(it) => it.hwnd.get() as _,
                    _ => unimplemented!(),
                }
            } else {
                null_mut()
            },
        };

        let (observer, rx) = ObserverWrapper::new(observer);
        let raw = unsafe {
            create_page(
                webview.raw,
                &options as *const _ as _,
                webview_sys::PageObserver {
                    on_state_change: Some(ObserverWrapper::on_state_change),
                    on_ime_rect: Some(ObserverWrapper::on_ime_rect),
                    on_frame: Some(ObserverWrapper::on_frame),
                    on_title_change: Some(ObserverWrapper::on_title_change),
                    on_fullscreen_change: Some(ObserverWrapper::on_fullscreen_change),
                    on_bridge: Some(ObserverWrapper::on_bridge),
                },
                &observer as *const _ as _,
            )
        };

        (
            Self {
                observer,
                options,
                raw,
            },
            rx,
        )
    }

    /// Send a mouse click event to the browser.
    ///
    /// Send a mouse move event to the browser.
    ///
    /// Send a mouse wheel event to the browser.
    pub fn on_mouse(&self, action: MouseAction) {
        match action {
            MouseAction::Move(pos) => unsafe { page_send_mouse_move(self.raw, pos.x, pos.y) },
            MouseAction::Wheel(pos) => unsafe { page_send_mouse_wheel(self.raw, pos.x, pos.y) },
            MouseAction::Click(button, state, pos) => {
                if let Some(pos) = pos {
                    unsafe {
                        page_send_mouse_click_with_pos(
                            self.raw,
                            button,
                            state.is_pressed(),
                            pos.x,
                            pos.y,
                        )
                    }
                } else {
                    unsafe { page_send_mouse_click(self.raw, button, state.is_pressed()) }
                }
            }
        }
    }

    /// Send a key event to the browser.
    pub fn on_keyboard(&self, scan_code: u32, state: ActionState, modifiers: Modifiers) {
        unsafe { page_send_keyboard(self.raw, scan_code as c_int, state.is_pressed(), modifiers) }
    }

    /// Send a touch event to the browser for a windowless browser.
    pub fn on_touch(
        &self,
        id: i32,
        x: i32,
        y: i32,
        ty: TouchEventType,
        pointer_type: TouchPointerType,
    ) {
        unsafe { page_send_touch(self.raw, id, x, y, ty, pointer_type) }
    }

    /// Completes the existing composition by optionally inserting the specified
    /// |text| into the composition node.
    ///
    /// Begins a new composition or updates the existing composition.
    ///
    /// Blink has a special node (a composition node) that allows the input
    /// method to change text without affecting other DOM nodes. |text| is the
    /// optional text that will be inserted into the composition node.
    /// |underlines| is an optional set of ranges that will be underlined in the
    /// resulting text. |replacement_range| is an optional range of the existing
    /// text that will be replaced. |selection_range| is an optional range of
    /// the resulting text that will be selected after insertion or replacement.
    /// The |replacement_range| value is only used on OS X.
    ///
    /// This method may be called multiple times as the composition changes.
    /// When the client is done making changes the composition should either be
    /// canceled or completed. To cancel the composition call
    /// ImeCancelComposition. To complete the composition call either
    /// ImeCommitText or ImeFinishComposingText. Completion is usually signaled
    /// when:
    ///
    /// 1, The client receives a WM_IME_COMPOSITION message with a GCS_RESULTSTR
    /// flag (on Windows), or; 2, The client receives a "commit" signal of
    /// GtkIMContext (on Linux), or; 3, insertText of NSTextInput is called
    /// (on Mac).
    ///
    /// This method is only used when window rendering is disabled.
    pub fn on_ime(&self, action: ImeAction) {
        match action {
            ImeAction::Composition(input) => unsafe {
                page_send_ime_composition(self.raw, input.as_pstr().0 as _)
            },
            ImeAction::Pre(input, x, y) => unsafe {
                page_send_ime_set_composition(self.raw, input.as_pstr().0 as _, x, y)
            },
        }
    }

    /// Notify the browser that the widget has been resized.
    ///
    /// The browser will first call CefRenderHandler::GetViewRect to get the new
    /// size and then call CefRenderHandler::OnPaint asynchronously with the
    /// updated regions. This method is only used when window rendering is
    /// disabled.
    pub fn resize(&self, width: u32, height: u32) {
        unsafe { page_resize(self.raw, width as c_int, height as c_int) }
    }

    /// Retrieve the window handle (if any) for this browser.
    ///
    /// If this browser is wrapped in a CefBrowserView this method should be
    /// called on the browser process UI thread and it will return the handle
    /// for the top-level native window.
    pub fn window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Win32(Win32WindowHandle::new(
            NonZeroIsize::new(unsafe { page_get_hwnd(self.raw) } as _).unwrap(),
        ))
    }

    /// Open developer tools (DevTools) in its own browser.
    ///
    /// The DevTools browser will remain associated with this browser.
    pub fn set_devtools_state(&self, is_open: bool) {
        unsafe { page_set_devtools_state(self.raw, is_open) }
    }
}

impl Drop for PageWrapper {
    fn drop(&mut self) {
        unsafe {
            page_exit(self.raw as _);
        }

        ffi::free(self.options.url);
    }
}

#[allow(unused)]
pub trait Observer: Send + Sync {
    /// Implement this interface to handle events related to browser load
    /// status.
    ///
    /// The methods of this class will be called on the browser process UI
    /// thread or render process main thread (TID_RENDERER).
    fn on_state_change(&self, state: PageState) {}
    /// Called when the IME composition range has changed.
    ///
    /// selected_range is the range of characters that have been selected.
    /// |character_bounds| is the bounds of each character in view coordinates.
    fn on_ime_rect(&self, rect: Rect) {}
    /// Called when an element should be painted.
    ///
    /// Pixel values passed to this method are scaled relative to view
    /// coordinates based on the value of CefScreenInfo.device_scale_factor
    /// returned from GetScreenInfo. |type| indicates whether the element is the
    /// view or the popup widget. |buffer| contains the pixel data for the whole
    /// image. |dirtyRects| contains the set of rectangles in pixel coordinates
    /// that need to be repainted. |buffer| will be |width|*|height|*4 bytes in
    /// size and represents a BGRA image with an upper-left origin. This method
    /// is only called when CefWindowInfo::shared_texture_enabled is set to
    /// false.
    fn on_frame(&self, texture: &[u8], width: u32, height: u32) {}
    /// Called when the page title changes.
    fn on_title_change(&self, title: String) {}
    /// Called when web content in the page has toggled fullscreen mode.
    ///
    /// If |fullscreen| is true the content will automatically be sized to fill
    /// the browser content area. If |fullscreen| is false the content will
    /// automatically return to its original size and position. With Alloy style
    /// the client is responsible for triggering the fullscreen transition (for
    /// example, by calling CefWindow::SetFullscreen when using Views). With
    /// Chrome style the fullscreen transition will be triggered automatically.
    /// The CefWindowDelegate::OnWindowFullscreenTransition method will be
    /// called during the fullscreen transition for notification purposes.
    fn on_fullscreen_change(&self, fullscreen: bool) {}
}

pub enum ChannelEvents {
    StateChange(PageState),
}

pub(crate) struct ObserverWrapper {
    pub inner: Arc<dyn Observer>,
    pub tx: Arc<UnboundedSender<ChannelEvents>>,
    pub ctx: Arc<
        RwLock<Option<Arc<dyn Fn(String, Box<dyn FnOnce(Result<String, String>) + Send + Sync>)>>>,
    >,
}

unsafe impl Send for ObserverWrapper {}
unsafe impl Sync for ObserverWrapper {}

impl ObserverWrapper {
    fn new<T>(observer: T) -> (Self, UnboundedReceiver<ChannelEvents>)
    where
        T: Observer + 'static,
    {
        let (tx, rx) = unbounded_channel();
        (
            Self {
                ctx: Arc::new(RwLock::new(None)),
                inner: Arc::new(observer),
                tx: Arc::new(tx),
            },
            rx,
        )
    }

    /// Implement this interface to handle events related to browser load
    /// status.
    ///
    /// The methods of this class will be called on the browser process UI
    /// thread or render process main thread (TID_RENDERER).
    extern "C" fn on_state_change(state: PageState, this: *mut c_void) {
        (unsafe { &*(this as *mut Self) })
            .tx
            .send(ChannelEvents::StateChange(state))
            .expect("channel is closed, message send failed!");
    }

    /// Called when the IME composition range has changed.
    ///
    /// selected_range is the range of characters that have been selected.
    /// |character_bounds| is the bounds of each character in view coordinates.
    extern "C" fn on_ime_rect(rect: Rect, this: *mut c_void) {
        (unsafe { &*(this as *mut Self) }).inner.on_ime_rect(rect);
    }

    /// Called when an element should be painted.
    ///
    /// Pixel values passed to this method are scaled relative to view
    /// coordinates based on the value of CefScreenInfo.device_scale_factor
    /// returned from GetScreenInfo. |type| indicates whether the element is the
    /// view or the popup widget. |buffer| contains the pixel data for the whole
    /// image. |dirtyRects| contains the set of rectangles in pixel coordinates
    /// that need to be repainted. |buffer| will be |width|*|height|*4 bytes in
    /// size and represents a BGRA image with an upper-left origin. This method
    /// is only called when CefWindowInfo::shared_texture_enabled is set to
    /// false.
    extern "C" fn on_frame(texture: *const c_void, width: c_int, height: c_int, this: *mut c_void) {
        (unsafe { &*(this as *mut Self) }).inner.on_frame(
            unsafe { from_raw_parts(texture as *const _, width as usize * height as usize * 4) },
            width as u32,
            height as u32,
        );
    }

    /// Called when the page title changes.
    extern "C" fn on_title_change(title: *const c_char, this: *mut c_void) {
        if let Some(title) = ffi::from(title) {
            (unsafe { &*(this as *mut Self) })
                .inner
                .on_title_change(title);
        }
    }

    /// Called when web content in the page has toggled fullscreen mode.
    ///
    /// If |fullscreen| is true the content will automatically be sized to fill
    /// the browser content area. If |fullscreen| is false the content will
    /// automatically return to its original size and position. With Alloy style
    /// the client is responsible for triggering the fullscreen transition (for
    /// example, by calling CefWindow::SetFullscreen when using Views). With
    /// Chrome style the fullscreen transition will be triggered automatically.
    /// The CefWindowDelegate::OnWindowFullscreenTransition method will be
    /// called during the fullscreen transition for notification purposes.
    extern "C" fn on_fullscreen_change(fullscreen: bool, this: *mut c_void) {
        (unsafe { &*(this as *mut Self) })
            .inner
            .on_fullscreen_change(fullscreen);
    }

    extern "C" fn on_bridge(
        req: *const c_char,
        this: *mut c_void,
        ctx: *mut c_void,
        callback: Option<unsafe extern "C" fn(*mut c_void, webview_sys::Result)>,
    ) {
        let callback = if let Some(it) = callback {
            it
        } else {
            return;
        };

        if let Some(req) = ffi::from(req) {
            if let Some(func) = (unsafe { &*(this as *mut Self) })
                .ctx
                .read()
                .unwrap()
                .as_ref()
            {
                let ctx = ctx as usize;
                func(
                    req,
                    Box::new(move |it| unsafe {
                        callback(
                            ctx as *mut c_void,
                            match it {
                                Ok(it) => webview_sys::Result {
                                    success: it.as_pstr().0 as _,
                                    failure: null_mut(),
                                },
                                Err(it) => webview_sys::Result {
                                    failure: it.as_pstr().0 as _,
                                    success: null_mut(),
                                },
                            },
                        );
                    }),
                );
            }
        }
    }
}
