mod observer;
mod page;

use std::{
    env::args,
    ffi::{c_char, c_int},
    sync::{
        mpsc::channel,
        Arc, Mutex,
    },
    thread,
};

pub use webview_sys::{Modifiers, MouseButtons, PageState, TouchEventType, TouchPointerType};

pub use self::{
    observer::Observer,
    page::{Page, PageOptions},
};

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
pub enum ImeAction<'a> {
    Composition(&'a str),
    Pre(&'a str, i32, i32),
}

pub(crate) struct Args(Vec<*const c_char>);

impl Default for Args {
    fn default() -> Self {
        Self(args().map(|it| ffi::into(&it)).collect::<Vec<_>>())
    }
}

impl Drop for Args {
    fn drop(&mut self) {
        for it in &self.0 {
            ffi::free(*it);
        }

        self.0.clear();
    }
}

impl Args {
    pub fn len(&self) -> c_int {
        self.0.len() as c_int
    }

    pub fn as_ptr(&self) -> *mut *const c_char {
        self.0.as_ptr() as _
    }
}

/// webview sub process does not work in tokio runtime!
pub fn execute_subprocess() -> ! {
    let args = Args::default();
    unsafe { webview_sys::execute_sub_process(args.len(), args.as_ptr()) };
    unreachable!("sub process closed, this is a bug!")
}

pub fn is_subprocess() -> bool {
    args().find(|v| v.contains("--type")).is_some()
}

#[derive(Debug, Default)]
pub struct WebviewOptions<'a> {
    pub cache_path: Option<&'a str>,
    pub browser_subprocess_path: Option<&'a str>,
    pub scheme_path: Option<&'a str>,
}

#[derive(Debug)]
pub enum Error {
    CreateWebviewError,
    CreatePageError,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
pub struct Webview {
    pub(crate) wrapper: Arc<wrapper::Webview>,
    condvar: Arc<Mutex<()>>,
}

impl Webview {
    pub fn new(options: &WebviewOptions<'_>) -> Result<Self, Error> {
        let condvar = Arc::new(Mutex::new(()));
        let (tx, rx) = channel();
        let wrapper =
            Arc::new(wrapper::Webview::new(&options, tx).ok_or_else(|| Error::CreateWebviewError)?);

        let condvar_ = condvar.clone();
        let wrapper_ = wrapper.clone();
        thread::spawn(move || {
            let condvar = condvar_.lock().unwrap();

            wrapper_.run();
            drop(condvar)
        });

        rx.recv().map_err(|_| Error::CreateWebviewError)?;
        Ok(Self { condvar, wrapper })
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
    pub fn create_page<T>(
        &self,
        url: &str,
        settings: &PageOptions,
        observer: T,
    ) -> Result<Arc<Page>, Error>
    where
        T: Observer + 'static,
    {
        Page::new(&self, url, settings, observer)
    }

    pub fn wait_exit(&self) {
        let _unused = self.condvar.lock().unwrap();
    }
}

pub(crate) mod wrapper {
    use std::{
        ffi::c_void,
        sync::mpsc::{Receiver, Sender},
    };

    use webview_sys::{create_webview, webview_exit, webview_run, PageState};

    use crate::{ffi, page::wrapper::Page, Args, Observer, PageOptions, WebviewOptions};

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
    pub(crate) struct Webview(pub *mut c_void);

    unsafe impl Send for Webview {}
    unsafe impl Sync for Webview {}

    impl Webview {
        extern "C" fn callback(ctx: *mut c_void) {
            if let Err(e) = unsafe { Box::from_raw(ctx as *mut Sender<()>) }.send(()) {
                log::error!(
                    "An error occurred when webview pushed a message to the callback. error={:?}",
                    e
                );
            }
        }

        pub(crate) fn new(options: &WebviewOptions, tx: Sender<()>) -> Option<Self> {
            let mut options = webview_sys::WebviewOptions {
                cache_path: ffi::into_opt(options.cache_path),
                scheme_path: ffi::into_opt(options.scheme_path),
                browser_subprocess_path: ffi::into_opt(options.browser_subprocess_path),
            };

            let raw = unsafe {
                create_webview(
                    &mut options,
                    Some(Self::callback),
                    Box::into_raw(Box::new(tx)) as *mut _,
                )
            };

            {
                ffi::free(options.cache_path);
                ffi::free(options.scheme_path);
                ffi::free(options.browser_subprocess_path);
            }

            if raw.is_null() {
                return None;
            }

            Some(Self(raw))
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
            url: &str,
            options: &PageOptions,
            observer: T,
        ) -> (Page, Receiver<PageState>)
        where
            T: Observer + 'static,
        {
            Page::new(&self, url, options, observer)
        }

        pub(crate) fn run(&self) {
            let args = Args::default();
            if unsafe { webview_run(self.0, args.len(), args.as_ptr()) } != 0 {
                panic!("Webview exited unexpectedly, this is a bug.")
            }
        }
    }

    impl Drop for Webview {
        fn drop(&mut self) {
            unsafe {
                webview_exit(self.0);
            }
        }
    }
}

pub mod ffi {
    use std::{
        ffi::{c_char, CStr, CString},
        ptr::null,
    };

    pub fn into(value: &str) -> *const c_char {
        CString::new(value).unwrap().into_raw()
    }

    pub fn into_opt(value: Option<&str>) -> *const c_char {
        value
            .map(|it| CString::new(it).unwrap().into_raw() as _)
            .unwrap_or_else(|| null())
    }

    pub fn from(value: *const c_char) -> Option<String> {
        if !value.is_null() {
            unsafe { CStr::from_ptr(value) }
                .to_str()
                .map(|s| s.to_string())
                .ok()
        } else {
            None
        }
    }

    pub fn free(value: *const c_char) {
        if !value.is_null() {
            drop(unsafe { CString::from_raw(value as _) })
        }
    }
}
