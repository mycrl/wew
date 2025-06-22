use std::sync::LazyLock;

use anyhow::Result;
use wew::{
    MainThreadMessageLoop, MessageLoopAbstract, NativeWindowWebView, execute_subprocess,
    is_subprocess,
    runtime::RuntimeHandler,
    sync::{AsyncRuntime, AsyncRuntimeAttributes},
    webview::{WebViewAttributes, WebViewHandler},
};

struct WebViewObserver;

impl WebViewHandler for WebViewObserver {}

struct RuntimeObserver;

impl RuntimeHandler for RuntimeObserver {}

async fn create_webview(message_loop: MainThreadMessageLoop) -> Result<()> {
    let runtime = message_loop
        .create_runtime_attributes_builder::<NativeWindowWebView>()
        .build()
        .async_create_runtime(RuntimeObserver)
        .await?;

    let webview = runtime
        .async_create_webview(
            "https://google.com",
            WebViewAttributes::default(),
            WebViewObserver,
        )
        .await?;

    std::mem::forget(webview);
    std::mem::forget(runtime);
    Ok(())
}

static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

fn main() -> Result<()> {
    if is_subprocess() {
        execute_subprocess();
    }

    let message_loop = MainThreadMessageLoop::default();

    // Only after the message loop starts running will the runtime's `Future`
    // return, so we run this in a separate thread.
    RUNTIME.spawn(create_webview(message_loop));

    message_loop.block_run();
    Ok(())
}
