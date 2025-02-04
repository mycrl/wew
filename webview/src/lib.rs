mod page;
mod strings;
mod wrapper;

use std::{env::args, ffi::c_int};

use std::{sync::Arc, thread};
use tokio::sync::{oneshot, Notify};
use wrapper::{get_args, WebviewWrapper};

pub use webview_sys::{Modifiers, MouseButtons, PageState, TouchEventType, TouchPointerType};

pub use self::{
    page::{BridgeObserver, Page, PageError, PageOptions},
    wrapper::Observer,
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

pub fn execute_subprocess() -> ! {
    if tokio::runtime::Handle::try_current().is_ok() {
        panic!("webview sub process does not work in tokio runtime!");
    }

    let args = get_args();
    unsafe { webview_sys::execute_sub_process(args.len() as c_int, args.as_ptr() as _) };
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
pub enum WebviewError {
    CreateAppFailed,
}

impl std::error::Error for WebviewError {}

impl std::fmt::Display for WebviewError {
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
    inner: WebviewWrapper,
    notify: Notify,
}

impl Webview {
    pub async fn new(options: &WebviewOptions<'_>) -> Result<Arc<Self>, WebviewError> {
        let (tx, rx) = oneshot::channel::<()>();

        let this = Arc::new(Self {
            notify: Notify::new(),
            inner: WebviewWrapper::new(&options, tx)
                .ok_or_else(|| WebviewError::CreateAppFailed)?,
        });

        let this_ = this.clone();
        thread::spawn(move || {
            this_.inner.run();
            this_.notify.notify_waiters();
        });

        rx.await.map_err(|_| WebviewError::CreateAppFailed)?;
        Ok(this)
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
    pub async fn create_page<T>(
        &self,
        url: &str,
        settings: &PageOptions,
        observer: T,
    ) -> Result<Arc<Page>, PageError>
    where
        T: Observer + 'static,
    {
        Page::new(&self.inner, url, settings, observer).await
    }

    pub async fn wait_exit(&self) {
        self.notify.notified().await;
    }
}
