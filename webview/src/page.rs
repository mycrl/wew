use std::{
    sync::{mpsc::channel, Arc},
    thread,
};

use raw_window_handle::RawWindowHandle;
use webview_sys::{Modifiers, PageState, TouchEventType, TouchPointerType};

use crate::{ActionState, Error, ImeAction, MouseAction, Observer, Webview};

#[derive(Debug)]
pub struct PageOptions {
    pub window_handle: Option<RawWindowHandle>,
    pub frame_rate: u32,
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f32,
    pub is_offscreen: bool,
}

unsafe impl Send for PageOptions {}
unsafe impl Sync for PageOptions {}

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
pub struct Page(wrapper::Page);

impl Page {
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
    pub(crate) fn new<T>(
        webview: &Webview,
        url: &str,
        options: &PageOptions,
        observer: T,
    ) -> Result<Arc<Self>, Error>
    where
        T: Observer + 'static,
    {
        let (inner, receiver) = webview.wrapper.create_page(url, options, observer);

        let (tx, rx) = channel::<bool>();
        thread::spawn(move || {
            let mut tx = Some(tx);

            while let Ok(state) = receiver.recv() {
                match state {
                    PageState::LoadError => {
                        tx.take().map(|tx| tx.send(false));
                    }
                    PageState::Load => {
                        tx.take().map(|tx| tx.send(true));
                    }
                    _ => (),
                }
            }
        });

        if !rx.recv().map_err(|_| Error::CreatePageError)? {
            return Err(Error::CreatePageError);
        }

        Ok(Arc::new(Self(inner)))
    }

    /// Send a mouse click event to the browser.
    ///
    /// Send a mouse move event to the browser.
    ///
    /// Send a mouse wheel event to the browser.
    pub fn on_mouse(&self, action: MouseAction) {
        self.0.on_mouse(action);
    }

    /// Send a key event to the browser.
    pub fn on_keyboard(&self, scan_code: u32, state: ActionState, modifiers: Modifiers) {
        self.0.on_keyboard(scan_code, state, modifiers);
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
        self.0.on_touch(id, x, y, ty, pointer_type);
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
        self.0.on_ime(action);
    }

    /// Notify the browser that the widget has been resized.
    ///
    /// The browser will first call CefRenderHandler::GetViewRect to get the new
    /// size and then call CefRenderHandler::OnPaint asynchronously with the
    /// updated regions. This method is only used when window rendering is
    /// disabled.
    pub fn resize(&self, width: u32, height: u32) {
        self.0.resize(width, height);
    }

    /// Retrieve the window handle (if any) for this browser.
    ///
    /// If this browser is wrapped in a CefBrowserView this method should be
    /// called on the browser process UI thread and it will return the handle
    /// for the top-level native window.
    pub fn window_handle(&self) -> RawWindowHandle {
        self.0.window_handle()
    }

    /// Open developer tools (DevTools) in its own browser.
    ///
    /// The DevTools browser will remain associated with this browser.
    pub fn set_devtools_state(&self, is_open: bool) {
        self.0.set_devtools_state(is_open);
    }

    pub fn send_message(&self, message: &str) {
        self.0.send_message(message);
    }
}

pub(crate) mod wrapper {
    use std::{
        ffi::{c_int, c_void},
        num::NonZeroIsize,
        ptr::null,
        sync::mpsc::Receiver,
    };

    use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
    use webview_sys::{
        create_page, page_exit, page_get_hwnd, page_resize, page_send_ime_composition,
        page_send_ime_set_composition, page_send_keyboard, page_send_message,
        page_send_mouse_click, page_send_mouse_click_with_pos, page_send_mouse_move,
        page_send_mouse_wheel, page_send_touch, page_set_devtools_state, Modifiers, PageState,
        TouchEventType, TouchPointerType,
    };

    use crate::{
        ffi,
        observer::wrapper::{create_page_observer, Observer as ObserverWrapper},
        wrapper::Webview,
        ActionState, ImeAction, MouseAction, Observer,
    };

    use super::PageOptions;

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
    pub(crate) struct Page {
        pub observer: *mut ObserverWrapper,
        pub raw: *mut c_void,
    }

    unsafe impl Send for Page {}
    unsafe impl Sync for Page {}

    impl Page {
        pub(crate) fn new<T>(
            webview: &Webview,
            url: &str,
            options: &PageOptions,
            observer: T,
        ) -> (Self, Receiver<PageState>)
        where
            T: Observer + 'static,
        {
            let options = webview_sys::PageOptions {
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
                    null()
                },
            };

            let (observer, rx) = ObserverWrapper::new(observer);
            let observer = Box::into_raw(Box::new(observer));

            let url = ffi::into(url);
            let raw = unsafe {
                create_page(
                    webview.0,
                    url,
                    &options,
                    create_page_observer(),
                    observer as _,
                )
            };

            {
                ffi::free(url);
            }

            (Self { observer, raw }, rx)
        }

        pub(crate) fn send_message(&self, message: &str) {
            let message = ffi::into(message);

            unsafe {
                page_send_message(self.raw, message);
            }

            ffi::free(message);
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
            unsafe {
                page_send_keyboard(self.raw, scan_code as c_int, state.is_pressed(), modifiers)
            }
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
            let input = match action {
                ImeAction::Composition(it) | ImeAction::Pre(it, _, _) => ffi::into(it),
            };

            match action {
                ImeAction::Composition(_) => unsafe { page_send_ime_composition(self.raw, input) },
                ImeAction::Pre(_, x, y) => unsafe {
                    page_send_ime_set_composition(self.raw, input, x, y)
                },
            }

            ffi::free(input);
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
                NonZeroIsize::new(unsafe { page_get_hwnd(self.raw) as _ }).unwrap(),
            ))
        }

        /// Open developer tools (DevTools) in its own browser.
        ///
        /// The DevTools browser will remain associated with this browser.
        pub fn set_devtools_state(&self, is_open: bool) {
            unsafe { page_set_devtools_state(self.raw, is_open) }
        }
    }

    impl Drop for Page {
        fn drop(&mut self) {
            unsafe {
                page_exit(self.raw);
            }

            drop(unsafe { Box::from_raw(self.observer) });
        }
    }
}
