mod utils;

/// Used to handle window events.
pub mod events;

/// Network request related, including implementing custom request interception.
pub mod request;

/// This module is used to manage the runtime.
pub mod runtime;

/// `WebView` module and related types.
pub mod webview;

use self::runtime::{RUNTIME_RUNNING, RuntimeAttributesBuilder};

#[cfg(feature = "winit")]
pub use winit;

pub use log;

#[allow(
    dead_code,
    unused_imports,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]
mod sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Debug)]
pub enum Error {
    /// The current thread is not the main thread.
    NonUIThread,
    FailedToCreateRuntime,
    /// Only one runtime can be created in a process. Repeated creation will
    /// trigger this error.
    RuntimeAlreadyExists,
    /// If the runtime is not initialized, creating WebView and other operations
    /// will trigger this error.
    RuntimeNotInitialization,
    FailedToCreateWebView,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Message loop abstraction
///
/// Message loop abstraction, used to implement different message loop types.
pub trait MessageLoopAbstract: Default + Clone + Copy {
    /// Create a runtime attributes builder
    ///
    /// This function is used to create a runtime attributes builder.
    fn create_runtime_attributes_builder<W: Default>(&self) -> RuntimeAttributesBuilder<Self, W> {
        RuntimeAttributesBuilder::<Self, W>::default()
    }
}

/// Multi-threaded message loop
///
/// Using multi-threaded message runtime will create a separate thread inside
/// the runtime to run the message loop.
///
/// Note that macOS does not support this type of message loop.
#[derive(Default, Clone, Copy)]
pub struct MultiThreadMessageLoop;

impl MessageLoopAbstract for MultiThreadMessageLoop {}

/// Main thread message loop
///
/// You need to manually run the message loop in the main thread of the process.
#[derive(Default, Clone, Copy)]
pub struct MainThreadMessageLoop;

impl MessageLoopAbstract for MainThreadMessageLoop {}

impl MainThreadMessageLoop {
    /// Run the message loop on main thread
    ///
    /// This function is used to run the message loop on main thread.
    ///
    /// Note that this function will block the current thread until the message
    /// loop ends.
    pub fn block_run(&self) {
        unsafe { sys::run_message_loop() }
    }

    /// Quit the message loop on main thread
    ///
    /// This function is used to quit the message loop on main thread.
    ///
    /// Calling this function will cause `block_run` to exit and return.
    pub fn quit(&self) {
        unsafe {
            sys::quit_message_loop();
        }
    }
}

/// Message loop pump
///
/// If you need to integrate with existing message loops, the message pump
/// mechanism provides a way for you to drive the message loop yourself.
#[derive(Default, Clone, Copy)]
pub struct MessagePumpLoop;

impl MessageLoopAbstract for MessagePumpLoop {}

impl MessagePumpLoop {
    /// Drive the message loop pump on main thread
    ///
    /// This function is used to poll the message loop on main thread.
    ///
    /// Note that this function won't block the current thread, external code
    /// needs to drive the message loop pump.
    pub fn poll(&self) {
        use std::sync::atomic::Ordering;

        if RUNTIME_RUNNING.load(Ordering::Relaxed) {
            unsafe { sys::poll_message_loop() }
        }
    }
}

/// WebView abstraction
///
/// WebView abstraction, used to implement different WebView types.
pub trait WebViewAbstract: Default {}

/// Off-screen rendering mode
///
/// When using off-screen rendering mode, the WebView will not be displayed on
/// screen, but the rendering results will be pushed through
/// `WindowlessRenderWebViewHandler::on_frame`, and you can handle the video
/// frames yourself. Also, in this mode, mouse and keyboard events need to be
/// passed to the WebView by yourself.
#[derive(Default, Clone, Copy)]
pub struct WindowlessRenderWebView;

impl WebViewAbstract for WindowlessRenderWebView {}

/// Native window mode
///
/// When using native window mode, the WebView will create a native window and
/// display it on screen.
#[derive(Default, Clone, Copy)]
pub struct NativeWindowWebView;

impl WebViewAbstract for NativeWindowWebView {}

/// Execute subprocess
///
/// This function is used to execute subprocesses.
///
/// ### Please be careful!
///
/// Do not call this function in an asynchronous runtime, such as tokio,
/// which can lead to unexpected crashes!
///
/// Enabling the `tokio` feature allows for automatic checking.
pub fn execute_subprocess() -> bool {
    #[cfg(feature = "tokio")]
    {
        if tokio::runtime::Handle::try_current().is_ok() {
            panic!("execute_subprocess is not allowed in tokio runtime");
        }
    }

    if !utils::is_main_thread() {
        return false;
    }

    let args = utils::Args::default();
    (unsafe { sys::execute_subprocess(args.size() as _, args.as_ptr() as _) }) == 0
}

/// Check if current process is a subprocess
///
/// This function is used to check if the current process is a subprocess.
///
/// Note that if the current process is a subprocess, it will block until the
/// subprocess exits.
pub fn is_subprocess() -> bool {
    // This check is not very strict, but processes with a "type" parameter can
    // generally be considered subprocesses, unless the main process also uses
    // this parameter.
    std::env::args().find(|it| it.contains("--type")).is_some()
}
