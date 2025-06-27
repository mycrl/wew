use std::{
    ffi::{CString, c_char},
    ptr::{NonNull, null},
};

#[cfg(target_os = "macos")]
use std::{
    ffi::CStr,
    sync::atomic::{AtomicBool, Ordering},
};

#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::GetCurrentThreadId;

#[cfg(target_os = "macos")]
use objc2::{
    class,
    ffi::class_addMethod,
    msg_send,
    runtime::{AnyClass, AnyObject, Bool, Sel},
};

#[cfg(target_os = "linux")]
use libc::{SYS_gettid, c_long, getpid, syscall};

/// A pointer type that is assumed to be thread-safe.
///
/// The creator of this type must ensure that the pointer implementation is
/// thread-safe.
pub(crate) struct ThreadSafePointer<T>(NonNull<T>);

unsafe impl<T> Send for ThreadSafePointer<T> {}
unsafe impl<T> Sync for ThreadSafePointer<T> {}

impl<T> ThreadSafePointer<T> {
    #[inline]
    pub fn new(ptr: *mut T) -> Self {
        Self(NonNull::new(ptr).unwrap())
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }
}

pub(crate) trait AnySrtingCastRaw {
    fn as_raw(&self) -> *const c_char;
}

impl AnySrtingCastRaw for Option<CString> {
    #[inline]
    fn as_raw(&self) -> *const c_char {
        self.as_ref()
            .map(|it| it.as_c_str().as_ptr() as _)
            .unwrap_or_else(null)
    }
}

impl AnySrtingCastRaw for CString {
    #[inline]
    fn as_raw(&self) -> *const c_char {
        self.as_c_str().as_ptr()
    }
}

pub(crate) struct Args {
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
    #[inline]
    pub fn size(&self) -> usize {
        self.raw.len()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const *const c_char {
        self.raw.as_ptr() as _
    }
}

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

/// Perform initialization work for the `NSApplication` class on macOS.
///
/// Since wew is based on CEF, and CEF requires `NSApplication` to implement
/// `isHandlingSendEvent`, otherwise it will cause unexpected crashes. This
/// method automatically fixes this issue by adding the necessary implementation
/// to `NSApplication`.
#[cfg(target_os = "macos")]
pub fn startup_nsapplication() -> bool {
    static HANDLING_SEND_EVENT: AtomicBool = AtomicBool::new(false);

    extern "C" fn is_handling_send_event(_: &AnyObject, _: Sel) -> Bool {
        if HANDLING_SEND_EVENT.load(Ordering::Relaxed) {
            Bool::YES
        } else {
            Bool::NO
        }
    }

    extern "C" fn set_handling_send_event(_: &AnyObject, _: Sel, value: Bool) {
        HANDLING_SEND_EVENT.store(value.as_bool(), Ordering::Relaxed);
    }

    let app = if let Some(app) =
        AnyClass::get(unsafe { &CStr::from_bytes_with_nul_unchecked(b"NSApplication\0") })
    {
        app
    } else {
        return false;
    };

    {
        let sel = Sel::register(unsafe {
            &CStr::from_bytes_with_nul_unchecked(b"isHandlingSendEvent\0")
        });

        if !app.responds_to(sel.clone()) {
            if !unsafe {
                class_addMethod(
                    app as *const _ as *mut _,
                    sel,
                    std::mem::transmute(
                        is_handling_send_event as extern "C" fn(&AnyObject, Sel) -> Bool,
                    ),
                    "c@:\0".as_ptr() as _,
                )
                .as_bool()
            } {
                return false;
            }
        }
    }

    {
        let sel = Sel::register(unsafe {
            &CStr::from_bytes_with_nul_unchecked(b"setHandlingSendEvent:\0")
        });

        if !app.responds_to(sel.clone()) {
            if !unsafe {
                class_addMethod(
                    app as *const _ as *mut _,
                    sel,
                    std::mem::transmute(
                        set_handling_send_event as extern "C" fn(&AnyObject, Sel, Bool),
                    ),
                    "v@:c\0".as_ptr() as _,
                )
                .as_bool()
            } {
                return false;
            }
        }
    }

    true
}

/// Abstraction for obtaining a shared reference
///
/// In this project, a type usually has a corresponding shared reference type,
/// which is generally used internally. This allows for more accurate lifetime
/// management, enabling type A to hold type B and thus avoid premature Drop.
pub(crate) trait GetSharedRef {
    type Ref: Clone;

    fn get_shared_ref(&self) -> Self::Ref;
}
