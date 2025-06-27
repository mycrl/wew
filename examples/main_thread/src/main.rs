use std::{
    sync::mpsc::{Sender, channel},
    thread,
};

use wew::{
    MainThreadMessageLoop, MessageLoopAbstract, NativeWindowWebView,
    log::LevelFilter,
    runtime::RuntimeHandler,
    webview::{WebViewAttributes, WebViewHandler, WebViewState},
};

struct RuntimeObserver {
    tx: Sender<()>,
}

impl RuntimeHandler for RuntimeObserver {
    fn on_context_initialized(&self) {
        self.tx.send(()).unwrap();
    }
}

struct WebViewObserver;

impl WebViewHandler for WebViewObserver {
    fn on_state_change(&self, state: WebViewState) {
        if state == WebViewState::Close {
            std::process::exit(0);
        }
    }
}

fn main() {
    #[cfg(target_os = "macos")]
    wew::utils::startup_nsapplication();

    let message_loop = MainThreadMessageLoop::default();

    let mut runtime_attributes_builder =
        message_loop.create_runtime_attributes_builder::<NativeWindowWebView>();

    runtime_attributes_builder = runtime_attributes_builder
        // Set cache path, here we use environment variables passed by the build script.
        .with_root_cache_path(option_env!("CACHE_PATH").unwrap())
        .with_cache_path(option_env!("CACHE_PATH").unwrap())
        .with_log_severity(LevelFilter::Info);

    let (tx, rx) = channel();

    // Create runtime, wait for the `on_context_initialized` event to be triggered
    // before considering the creation successful.
    let runtime = runtime_attributes_builder
        .build()
        .create_runtime(RuntimeObserver { tx })
        .unwrap();

    thread::spawn(move || {
        rx.recv().unwrap();

        let webview = runtime
            .create_webview(
                "https://www.google.com",
                WebViewAttributes::default(),
                WebViewObserver,
            )
            .unwrap();

        std::mem::forget(webview);
        std::mem::forget(runtime);
    });

    message_loop.block_run();
}
