use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};

use crate::{
    Error, MainThreadMessageLoop, MessagePumpLoop, MultiThreadMessageLoop, NativeWindowWebView,
    WindowlessRenderWebView,
    runtime::{
        MessagePumpRuntimeHandler, MixRuntimeHnadler, Runtime, RuntimeAttributes, RuntimeHandler,
    },
    webview::{
        MixWebviewHnadler, Rect, WebView, WebViewAttributes, WebViewHandler, WebViewState,
        WindowlessRenderWebViewHandler,
    },
};

use async_trait::async_trait;
use futures_util::task::AtomicWaker;
use parking_lot::Mutex;

struct UnPark<T> {
    runing: Arc<AtomicBool>,
    output: Arc<Mutex<Option<T>>>,
    waker: Arc<AtomicWaker>,
}

impl<T> UnPark<T> {
    fn unpark(self, output: T) {
        self.output.lock().replace(output);
        self.waker.wake();
    }
}

impl<T> Drop for UnPark<T> {
    fn drop(&mut self) {
        self.runing.store(false, Ordering::Relaxed);
    }
}

struct Park<T> {
    runing: Arc<AtomicBool>,
    output: Arc<Mutex<Option<T>>>,
    waker: Arc<AtomicWaker>,
}

impl<T> Park<T> {
    fn new() -> (Self, UnPark<T>) {
        let output: Arc<Mutex<Option<T>>> = Default::default();
        let waker: Arc<AtomicWaker> = Default::default();
        let runing = Arc::new(AtomicBool::new(true));

        (
            Self {
                output: output.clone(),
                waker: waker.clone(),
                runing: runing.clone(),
            },
            UnPark {
                output,
                waker,
                runing,
            },
        )
    }
}

impl<T> Drop for Park<T> {
    fn drop(&mut self) {
        self.runing.store(false, Ordering::Relaxed);
    }
}

impl<T> Future for Park<T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(output) = self.output.lock().take() {
            return Poll::Ready(Some(output));
        } else {
            if !self.runing.load(Ordering::Relaxed) {
                return Poll::Ready(None);
            }
        }

        self.waker.register(cx.waker());
        Poll::Pending
    }
}

struct AsyncRuntimeHandler {
    handler: MixRuntimeHnadler,
    unpark: Mutex<Option<UnPark<()>>>,
}

impl RuntimeHandler for AsyncRuntimeHandler {
    fn on_context_initialized(&self) {
        if let Some(unpark) = self.unpark.lock().take() {
            unpark.unpark(());
        }

        match &self.handler {
            MixRuntimeHnadler::RuntimeHandler(handler) => {
                handler.on_context_initialized();
            }
            MixRuntimeHnadler::MessagePumpRuntimeHandler(handler) => {
                handler.on_context_initialized();
            }
        }
    }
}

impl MessagePumpRuntimeHandler for AsyncRuntimeHandler {
    fn on_schedule_message_pump_work(&self, delay: u64) {
        if let MixRuntimeHnadler::MessagePumpRuntimeHandler(handler) = &self.handler {
            handler.on_schedule_message_pump_work(delay);
        }
    }
}

#[async_trait]
pub trait AsyncRuntimeAttributes<T, R, W>
where
    Self: Sized,
{
    async fn async_create_runtime(self, handler: T) -> Result<Runtime<R, W>, Error>;
}

#[async_trait]
impl<W, T> AsyncRuntimeAttributes<T, MainThreadMessageLoop, W>
    for RuntimeAttributes<MainThreadMessageLoop, W>
where
    W: Send + Sync,
    T: RuntimeHandler + 'static,
{
    async fn async_create_runtime(
        self,
        handler: T,
    ) -> Result<Runtime<MainThreadMessageLoop, W>, Error> {
        let (park, unpark) = Park::<()>::new();

        match Runtime::new(
            self,
            MixRuntimeHnadler::MessagePumpRuntimeHandler(Box::new(AsyncRuntimeHandler {
                handler: MixRuntimeHnadler::RuntimeHandler(Box::new(handler)),
                unpark: Mutex::new(Some(unpark)),
            })),
        ) {
            Ok(runtime) => {
                if park.await.is_none() {
                    return Err(Error::FailedToCreateRuntime);
                }

                Ok(runtime)
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl<W, T> AsyncRuntimeAttributes<T, MultiThreadMessageLoop, W>
    for RuntimeAttributes<MultiThreadMessageLoop, W>
where
    W: Send + Sync,
    T: RuntimeHandler + 'static,
{
    async fn async_create_runtime(
        self,
        handler: T,
    ) -> Result<Runtime<MultiThreadMessageLoop, W>, Error> {
        let (park, unpark) = Park::<()>::new();

        match Runtime::new(
            self,
            MixRuntimeHnadler::MessagePumpRuntimeHandler(Box::new(AsyncRuntimeHandler {
                handler: MixRuntimeHnadler::RuntimeHandler(Box::new(handler)),
                unpark: Mutex::new(Some(unpark)),
            })),
        ) {
            Ok(runtime) => {
                if park.await.is_none() {
                    return Err(Error::FailedToCreateRuntime);
                }

                Ok(runtime)
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl<W, T> AsyncRuntimeAttributes<T, MessagePumpLoop, W> for RuntimeAttributes<MessagePumpLoop, W>
where
    W: Send + Sync,
    T: MessagePumpRuntimeHandler + 'static,
{
    async fn async_create_runtime(self, handler: T) -> Result<Runtime<MessagePumpLoop, W>, Error> {
        let (park, unpark) = Park::<()>::new();

        match Runtime::new(
            self,
            MixRuntimeHnadler::MessagePumpRuntimeHandler(Box::new(AsyncRuntimeHandler {
                handler: MixRuntimeHnadler::MessagePumpRuntimeHandler(Box::new(handler)),
                unpark: Mutex::new(Some(unpark)),
            })),
        ) {
            Ok(runtime) => {
                if park.await.is_none() {
                    return Err(Error::FailedToCreateRuntime);
                }

                Ok(runtime)
            }
            Err(e) => Err(e),
        }
    }
}

struct AsyncWebViewHandler {
    handler: MixWebviewHnadler,
    unpark: Mutex<Option<UnPark<bool>>>,
}

impl WebViewHandler for AsyncWebViewHandler {
    fn on_fullscreen_change(&self, fullscreen: bool) {
        match &self.handler {
            MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) => {
                handler.on_fullscreen_change(fullscreen);
            }
            MixWebviewHnadler::WebViewHandler(handler) => {
                handler.on_fullscreen_change(fullscreen);
            }
        }
    }

    fn on_message(&self, message: &str) {
        match &self.handler {
            MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) => {
                handler.on_message(message);
            }
            MixWebviewHnadler::WebViewHandler(handler) => {
                handler.on_message(message);
            }
        }
    }

    fn on_state_change(&self, state: WebViewState) {
        if state == WebViewState::LoadError || state == WebViewState::Loaded {
            if let Some(unpark) = self.unpark.lock().take() {
                unpark.unpark(state == WebViewState::Loaded);
            }
        }

        match &self.handler {
            MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) => {
                handler.on_state_change(state);
            }
            MixWebviewHnadler::WebViewHandler(handler) => {
                handler.on_state_change(state);
            }
        }
    }

    fn on_title_change(&self, title: &str) {
        match &self.handler {
            MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) => {
                handler.on_title_change(title);
            }
            MixWebviewHnadler::WebViewHandler(handler) => {
                handler.on_title_change(title);
            }
        }
    }
}

impl WindowlessRenderWebViewHandler for AsyncWebViewHandler {
    fn on_frame(&self, texture: &[u8], width: u32, height: u32) {
        if let MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) = &self.handler {
            handler.on_frame(texture, width, height);
        }
    }

    fn on_ime_rect(&self, rect: Rect) {
        if let MixWebviewHnadler::WindowlessRenderWebViewHandler(handler) = &self.handler {
            handler.on_ime_rect(rect);
        }
    }
}

#[async_trait]
pub trait AsyncRuntime<T, R, W>
where
    Self: Sized,
{
    async fn async_create_webview(
        &self,
        url: &str,
        attr: WebViewAttributes,
        handler: T,
    ) -> Result<WebView<R, W>, Error>;
}

#[async_trait]
impl<T, R> AsyncRuntime<T, R, WindowlessRenderWebView> for Runtime<R, WindowlessRenderWebView>
where
    T: WindowlessRenderWebViewHandler + 'static,
    R: Sync + Send + Clone,
{
    async fn async_create_webview(
        &self,
        url: &str,
        attr: WebViewAttributes,
        handler: T,
    ) -> Result<WebView<R, WindowlessRenderWebView>, Error> {
        let (park, unpark) = Park::<bool>::new();

        match WebView::new(
            self.clone(),
            url,
            attr,
            MixWebviewHnadler::WindowlessRenderWebViewHandler(Box::new(AsyncWebViewHandler {
                handler: MixWebviewHnadler::WindowlessRenderWebViewHandler(Box::new(handler)),
                unpark: Mutex::new(Some(unpark)),
            })),
        ) {
            Ok(webview) => {
                if let Some(result) = park.await {
                    if result {
                        return Ok(webview);
                    }
                }

                Err(Error::FailedToCreateWebView)
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl<T, R> AsyncRuntime<T, R, NativeWindowWebView> for Runtime<R, NativeWindowWebView>
where
    T: WebViewHandler + 'static,
    R: Sync + Send + Clone,
{
    async fn async_create_webview(
        &self,
        url: &str,
        attr: WebViewAttributes,
        handler: T,
    ) -> Result<WebView<R, NativeWindowWebView>, Error> {
        let (park, unpark) = Park::<bool>::new();

        match WebView::new(
            self.clone(),
            url,
            attr,
            MixWebviewHnadler::WindowlessRenderWebViewHandler(Box::new(AsyncWebViewHandler {
                handler: MixWebviewHnadler::WebViewHandler(Box::new(handler)),
                unpark: Mutex::new(Some(unpark)),
            })),
        ) {
            Ok(webview) => {
                if let Some(result) = park.await {
                    if result {
                        return Ok(webview);
                    }
                }

                Err(Error::FailedToCreateWebView)
            }
            Err(e) => Err(e),
        }
    }
}
