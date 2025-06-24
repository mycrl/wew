pub mod events;
pub mod request;
pub mod runtime;
pub mod webview;

use std::{
    env::args,
    ffi::{CString, c_char},
    ptr::{NonNull, null},
};

use self::runtime::{RUNTIME_RUNNING, RuntimeAttributesBuilder};

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

/// A pointer type that is assumed to be thread-safe.
///
/// The creator of this type must ensure that the pointer implementation is
/// thread-safe.
struct ThreadSafePointer<T>(NonNull<T>);

unsafe impl<T> Send for ThreadSafePointer<T> {}
unsafe impl<T> Sync for ThreadSafePointer<T> {}

impl<T> ThreadSafePointer<T> {
    fn new(ptr: *mut T) -> Self {
        Self(NonNull::new(ptr).unwrap())
    }

    fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }
}

trait CStringExt {
    fn as_raw(&self) -> *const c_char;
}

impl CStringExt for Option<CString> {
    fn as_raw(&self) -> *const c_char {
        self.as_ref()
            .map(|it| it.as_c_str().as_ptr() as _)
            .unwrap_or_else(|| null())
    }
}

impl CStringExt for CString {
    fn as_raw(&self) -> *const c_char {
        self.as_c_str().as_ptr()
    }
}

struct Args {
    #[allow(unused)]
    inner: Vec<CString>,
    raw: Vec<*const c_char>,
}

unsafe impl Send for Args {}
unsafe impl Sync for Args {}

impl Default for Args {
    fn default() -> Self {
        let inner = args()
            .map(|it| CString::new(it).unwrap())
            .collect::<Vec<_>>();

        let raw = inner.iter().map(|it| it.as_raw()).collect::<Vec<_>>();

        Self { inner, raw }
    }
}

impl Args {
    fn size(&self) -> usize {
        self.raw.len()
    }

    fn as_ptr(&self) -> *const *const c_char {
        self.raw.as_ptr() as _
    }
}

#[derive(Debug)]
pub enum Error {
    FailedToCreateRuntime,
    RuntimeAlreadyExists,
    RuntimeNotInitialization,
    FailedToCreateWebView,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

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

    let args = Args::default();
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
    args().find(|it| it.contains("--type")).is_some()
}
