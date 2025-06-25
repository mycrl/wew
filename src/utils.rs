use std::{
    ffi::{CString, c_char},
    ptr::{NonNull, null},
};

#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::GetCurrentThreadId;

#[cfg(target_os = "macos")]
use objc2::{class, msg_send};

#[cfg(target_os = "linux")]
use libc::{SYS_gettid, c_long, getpid, syscall};

/// Check if the current thread is the main thread.
///
/// # Returns
///
/// `true` if the current thread is the main thread, `false` otherwise.
pub fn is_main_thread() -> bool {
    #[allow(unused)]
    let mut is_main_thread = false;

    #[cfg(target_os = "windows")]
    {
        static mut MAIN_THREAD_ID: u32 = 0;

        #[used]
        #[allow(non_upper_case_globals)]
        #[unsafe(link_section = ".CRT$XCU")]
        static INIT_MAIN_THREAD_ID: unsafe fn() = {
            unsafe fn initer() {
                unsafe { MAIN_THREAD_ID = GetCurrentThreadId() };
            }

            initer
        };

        is_main_thread = unsafe { GetCurrentThreadId() == MAIN_THREAD_ID };
    }

    #[cfg(target_os = "macos")]
    {
        is_main_thread = unsafe { msg_send![class!(NSThread), isMainThread] };
    }

    #[cfg(target_os = "linux")]
    {
        is_main_thread = { syscall(SYS_gettid) == getpid() as c_long };
    }

    is_main_thread
}

/// A pointer type that is assumed to be thread-safe.
///
/// The creator of this type must ensure that the pointer implementation is
/// thread-safe.
pub struct ThreadSafePointer<T>(NonNull<T>);

unsafe impl<T> Send for ThreadSafePointer<T> {}
unsafe impl<T> Sync for ThreadSafePointer<T> {}

impl<T> ThreadSafePointer<T> {
    pub fn new(ptr: *mut T) -> Self {
        Self(NonNull::new(ptr).unwrap())
    }

    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }
}

pub trait CStringExt {
    fn as_raw(&self) -> *const c_char;
}

impl CStringExt for Option<CString> {
    fn as_raw(&self) -> *const c_char {
        self.as_ref()
            .map(|it| it.as_c_str().as_ptr() as _)
            .unwrap_or_else(null)
    }
}

impl CStringExt for CString {
    fn as_raw(&self) -> *const c_char {
        self.as_c_str().as_ptr()
    }
}

pub struct Args {
    #[allow(unused)]
    inner: Vec<CString>,
    raw: Vec<*const c_char>,
}

unsafe impl Send for Args {}
unsafe impl Sync for Args {}

impl Default for Args {
    fn default() -> Self {
        let inner = std::env::args()
            .map(|it| CString::new(it).unwrap())
            .collect::<Vec<_>>();

        let raw = inner.iter().map(|it| it.as_raw()).collect::<Vec<_>>();

        Self { inner, raw }
    }
}

impl Args {
    pub fn size(&self) -> usize {
        self.raw.len()
    }

    pub fn as_ptr(&self) -> *const *const c_char {
        self.raw.as_ptr() as _
    }
}
