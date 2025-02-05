use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use raw_window_handle::RawWindowHandle;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{runtime::Handle, sync::oneshot::channel, time::timeout};

use webview_sys::{Modifiers, PageState, TouchEventType, TouchPointerType};

use crate::{
    wrapper::{ChannelEvents, PageWrapper},
    ActionState, ImeAction, MouseAction, Observer, WebviewWrapper,
};

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

#[derive(Debug)]
pub enum PageError {
    CreateBrowserFailed,
    BridgeSerdeError,
    BridgeTimeout,
    BridgeCallError,
}

impl std::error::Error for PageError {}

impl std::fmt::Display for PageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
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
pub struct Page {
    runtime: Handle,
    inner: PageWrapper,
}

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
    pub(crate) async fn new<T>(
        webview: &WebviewWrapper,
        url: &str,
        options: &PageOptions,
        observer: T,
    ) -> Result<Arc<Self>, PageError>
    where
        T: Observer + 'static,
    {
        let (inner, mut receiver) = webview.create_page(url, options, observer);

        let (tx, rx) = channel::<bool>();
        tokio::spawn(async move {
            let mut tx = Some(tx);

            while let Some(events) = receiver.recv().await {
                match events {
                    ChannelEvents::StateChange(state) => match state {
                        PageState::LoadError => {
                            tx.take().map(|tx| tx.send(false));
                        }
                        PageState::Load => {
                            tx.take().map(|tx| tx.send(true));
                        }
                        _ => (),
                    },
                }
            }
        });

        if !rx.await.map_err(|_| PageError::CreateBrowserFailed)? {
            return Err(PageError::CreateBrowserFailed);
        }

        Ok(Arc::new(Self {
            runtime: Handle::current(),
            inner,
        }))
    }

    pub async fn call_bridge<Q, S>(&self, req: &Q) -> Result<Option<S>, PageError>
    where
        Q: Serialize,
        S: DeserializeOwned,
    {
        let (tx, rx) = channel::<Option<String>>();
        let req = serde_json::to_string(req).map_err(|_| PageError::BridgeSerdeError)?;

        self.inner.call(&req, tx);

        Ok(
            if let Some(ret) = timeout(Duration::from_secs(10), rx)
                .await
                .map_err(|_| PageError::BridgeTimeout)?
                .map_err(|_| PageError::BridgeCallError)?
            {
                Some(serde_json::from_str(&ret).map_err(|_| PageError::BridgeSerdeError)?)
            } else {
                None
            },
        )
    }

    pub fn on_bridge<Q, S, H>(&self, observer: H)
    where
        Q: DeserializeOwned + Send + 'static,
        S: Serialize + 'static,
        H: BridgeObserver<Req = Q, Res = S> + 'static,
    {
        let runtime = self.runtime.clone();
        let prcesser = Arc::new(BridgeHandler::new(observer));
        let _ = unsafe { &*self.inner.observer }
            .ctx
            .write()
            .unwrap()
            .insert(Arc::new(move |req, callback| {
                let prcesser = prcesser.clone();
                runtime.spawn(async move {
                    callback(prcesser.handle(&req).await);
                });
            }));
    }

    /// Send a mouse click event to the browser.
    ///
    /// Send a mouse move event to the browser.
    ///
    /// Send a mouse wheel event to the browser.
    pub fn on_mouse(&self, action: MouseAction) {
        self.inner.on_mouse(action);
    }

    /// Send a key event to the browser.
    pub fn on_keyboard(&self, scan_code: u32, state: ActionState, modifiers: Modifiers) {
        self.inner.on_keyboard(scan_code, state, modifiers);
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
        self.inner.on_touch(id, x, y, ty, pointer_type);
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
        self.inner.on_ime(action);
    }

    /// Notify the browser that the widget has been resized.
    ///
    /// The browser will first call CefRenderHandler::GetViewRect to get the new
    /// size and then call CefRenderHandler::OnPaint asynchronously with the
    /// updated regions. This method is only used when window rendering is
    /// disabled.
    pub fn resize(&self, width: u32, height: u32) {
        self.inner.resize(width, height);
    }

    /// Retrieve the window handle (if any) for this browser.
    ///
    /// If this browser is wrapped in a CefBrowserView this method should be
    /// called on the browser process UI thread and it will return the handle
    /// for the top-level native window.
    pub fn window_handle(&self) -> RawWindowHandle {
        self.inner.window_handle()
    }

    /// Open developer tools (DevTools) in its own browser.
    ///
    /// The DevTools browser will remain associated with this browser.
    pub fn set_devtools_state(&self, is_open: bool) {
        self.inner.set_devtools_state(is_open);
    }
}

#[async_trait]
pub trait BridgeObserver: Send + Sync {
    type Req: DeserializeOwned + Send;
    type Res: Serialize + 'static;
    type Err: ToString;

    async fn on(&self, req: Self::Req) -> Result<Self::Res, Self::Err>;
}

pub(crate) struct BridgeHandler<Q, S, E> {
    processor: Arc<dyn BridgeObserver<Req = Q, Res = S, Err = E>>,
}

impl<Q, S, E> BridgeHandler<Q, S, E>
where
    Q: DeserializeOwned + Send,
    S: Serialize + 'static,
    E: ToString,
{
    pub(crate) fn new<T: BridgeObserver<Req = Q, Res = S, Err = E> + 'static>(
        processer: T,
    ) -> Self {
        Self {
            processor: Arc::new(processer),
        }
    }

    pub(crate) async fn handle(&self, req: &str) -> Result<String, String> {
        serde_json::to_string(
            &self
                .processor
                .on(serde_json::from_str(unsafe { std::mem::transmute(req) })
                    .map_err(|s| s.to_string())?)
                .await
                .map_err(|s| s.to_string())?,
        )
        .map_err(|s| s.to_string())
    }
}
