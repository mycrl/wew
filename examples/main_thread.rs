use anyhow::Result;
use webview::{
    MainThreadRuntime, NativeWindowWebView, execute_subprocess, is_subprocess,
    runtime::{RuntimeAttributesBuilder, RuntimeHandler},
    webview::{WebViewAttributes, WebViewHandler},
};

struct WebViewObserver;

impl WebViewHandler for WebViewObserver {}

struct RuntimeObserver;

impl RuntimeHandler for RuntimeObserver {}

fn main() -> Result<()> {
    if is_subprocess() {
        execute_subprocess();
    }

    let runtime = RuntimeAttributesBuilder::<MainThreadRuntime, NativeWindowWebView>::default()
        .build()
        .create_runtime(RuntimeObserver)?;

    let webview = runtime.create_webview(
        "https://google.com",
        &WebViewAttributes::default(),
        WebViewObserver,
    )?;

    runtime.block_run();

    drop(webview);
    drop(runtime);

    Ok(())
}
