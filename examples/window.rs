use std::thread;

use anyhow::Result;
use webview::{
    Runtime, RuntimeAttributesBuilder, RuntimeHandler, WebViewAttributes, WebViewHandler,
};

struct WebViewObserver;

impl WebViewHandler for WebViewObserver {}

struct RuntimeObserver;

impl RuntimeHandler for RuntimeObserver {}

fn main() -> Result<()> {
    if Runtime::is_subprocess() {
        Runtime::execute_subprocess();
        return Ok(());
    }

    let runtime = RuntimeAttributesBuilder::default()
        .build()
        .create_runtime(RuntimeObserver)?;

    let webview = runtime.create_webview(
        "https://google.com",
        &WebViewAttributes::default(),
        WebViewObserver,
    )?;

    thread::park();

    drop(webview);
    drop(runtime);
    Ok(())
}
