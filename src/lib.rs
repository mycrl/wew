pub mod runtime;
pub mod scheme;
pub mod webview;

use std::{
    env::args,
    ffi::{CString, c_char},
    ptr::null,
};

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
struct ThreadSafePointer<T>(*mut T);

unsafe impl<T> Send for ThreadSafePointer<T> {}
unsafe impl<T> Sync for ThreadSafePointer<T> {}

impl<T> ThreadSafePointer<T> {
    fn as_ptr(&self) -> *mut T {
        self.0
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

        let raw = inner
            .iter()
            .map(|it| it.as_c_str().as_ptr())
            .collect::<Vec<_>>();

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
    FailedToCreateWebView,
    RuntimeAlreadyExists,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait RuntimeAbstract: Default {}

#[derive(Default)]
pub struct MultiThreadRuntime;

impl RuntimeAbstract for MultiThreadRuntime {}

#[derive(Default)]
pub struct MainThreadRuntime;

impl RuntimeAbstract for MainThreadRuntime {}

#[derive(Default)]
pub struct MessagePumpRuntime;

impl RuntimeAbstract for MessagePumpRuntime {}

pub trait WebViewAbstract: Default {}

#[derive(Default)]
pub struct WindowlessRenderWebView;

impl WebViewAbstract for WindowlessRenderWebView {}

#[derive(Default)]
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
pub fn execute_subprocess() -> bool {
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
