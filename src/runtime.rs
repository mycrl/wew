use std::{
    ffi::{CString, c_void},
    marker::PhantomData,
    ops::Deref,
    ptr::null,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use parking_lot::Mutex;

use crate::{
    Args, CStringExt, Error, MainThreadRuntime, MessagePumpRuntime, MultiThreadRuntime,
    NativeWindowWebView, ThreadSafePointer, WindowlessRenderWebView,
    scheme::CustomSchemeAttributes,
    sys,
    webview::{
        MixWebviewHnadler, WebView, WebViewAttributes, WebViewHandler,
        WindowlessRenderWebViewHandler,
    },
};

/// Runtime configuration attributes
#[derive(Default)]
pub struct RuntimeAttributes<'a, R, W> {
    _r: PhantomData<R>,
    _w: PhantomData<W>,

    /// Custom scheme handler
    ///
    /// This is used to handle custom scheme requests.
    custom_scheme: Option<CustomSchemeAttributes<'a>>,

    /// Whether to enable windowless rendering mode
    ///
    /// Do not enable this value if the application does not use windowless
    /// rendering as it may reduce rendering performance on some systems.
    windowless_rendering_enabled: bool,

    /// The directory where data for the global browser cache will be stored on
    /// disk
    cache_dir_path: Option<CString>,

    /// The path to a separate executable that will be launched for
    /// sub-processes
    ///
    /// This executable will be launched to handle sub-processes.
    browser_subprocess_path: Option<CString>,

    /// The path to the CEF framework directory on macOS
    ///
    /// If this value is empty, the framework must exist at
    /// "Contents/Frameworks/Chromium Embedded Framework.framework" in the
    /// top-level app bundle. If this value is non-empty, it must be an
    /// absolute path. Also configurable using the "framework-dir-path"
    /// command-line switch.
    framework_dir_path: Option<CString>,

    /// The path to the main bundle on macOS
    ///
    /// If this value is empty, the main bundle must exist at
    /// "Contents/MacOS/main" in the top-level app bundle. If this value is
    /// non-empty, it must be an absolute path. Also configurable using the
    /// "main-bundle-path" command-line switch.
    main_bundle_path: Option<CString>,

    /// Whether to use external message pump
    ///
    /// If this value is true, the application must implement the message pump
    /// driver.
    external_message_pump: bool,

    /// Whether to use multi-threaded message loop
    multi_threaded_message_loop: bool,
}

impl<'a, W> RuntimeAttributes<'a, MainThreadRuntime, W> {
    pub fn create_runtime<T>(&self, handler: T) -> Result<Runtime<MainThreadRuntime, W>, Error>
    where
        T: RuntimeHandler + 'static,
    {
        Runtime::new(&self, MixRuntimeHnadler::RuntimeHandler(Box::new(handler)))
    }
}

impl<'a, W> RuntimeAttributes<'a, MultiThreadRuntime, W> {
    pub fn create_runtime<T>(&self, handler: T) -> Result<Runtime<MultiThreadRuntime, W>, Error>
    where
        T: RuntimeHandler + 'static,
    {
        Runtime::new(&self, MixRuntimeHnadler::RuntimeHandler(Box::new(handler)))
    }
}

impl<'a, W> RuntimeAttributes<'a, MessagePumpRuntime, W> {
    pub fn create_runtime<T>(&self, handler: T) -> Result<Runtime<MessagePumpRuntime, W>, Error>
    where
        T: MessagePumpRuntimeHandler + 'static,
    {
        Runtime::new(
            &self,
            MixRuntimeHnadler::MessagePumpRuntimeHandler(Box::new(handler)),
        )
    }
}

#[derive(Default)]
pub struct RuntimeAttributesBuilder<'a, R, W>(RuntimeAttributes<'a, R, W>);

impl<'a, R, W> RuntimeAttributesBuilder<'a, R, W> {
    /// Set the custom scheme handler
    ///
    /// This is used to handle custom scheme requests.
    pub fn with_custom_scheme(mut self, scheme: CustomSchemeAttributes<'a>) -> Self {
        self.0.custom_scheme = Some(scheme);
        self
    }

    /// Set the directory where data for the global browser cache will be stored
    /// on disk
    pub fn with_cache_dir_path(mut self, value: &str) -> Self {
        self.0.cache_dir_path = Some(CString::new(value).unwrap());
        self
    }

    /// Set the path to a separate executable that will be launched for
    /// sub-processes
    ///
    /// This executable will be launched to handle sub-processes.
    pub fn with_browser_subprocess_path(mut self, value: &str) -> Self {
        self.0.browser_subprocess_path = Some(CString::new(value).unwrap());
        self
    }

    /// Set the path to the CEF framework directory on macOS
    ///
    /// If this value is empty, the framework must exist at
    /// "Contents/Frameworks/Chromium Embedded Framework.framework" in the
    /// top-level app bundle. If this value is non-empty, it must be an
    /// absolute path. Also configurable using the "framework-dir-path"
    /// command-line switch.
    pub fn with_framework_dir_path(mut self, value: &str) -> Self {
        self.0.framework_dir_path = Some(CString::new(value).unwrap());
        self
    }

    /// Set the path to the main bundle on macOS
    ///
    /// If this value is empty, the main bundle must exist at
    /// "Contents/MacOS/main" in the top-level app bundle. If this value is
    /// non-empty, it must be an absolute path. Also configurable using the
    /// "main-bundle-path" command-line switch.
    pub fn with_main_bundle_path(mut self, value: &str) -> Self {
        self.0.main_bundle_path = Some(CString::new(value).unwrap());
        self
    }
}

impl<'a, W> RuntimeAttributesBuilder<'a, MultiThreadRuntime, W> {
    pub fn build(mut self) -> RuntimeAttributes<'a, MultiThreadRuntime, W> {
        self.0.multi_threaded_message_loop = true;
        self.0.external_message_pump = false;
        self.0
    }
}

impl<'a, W> RuntimeAttributesBuilder<'a, MainThreadRuntime, W> {
    pub fn build(mut self) -> RuntimeAttributes<'a, MainThreadRuntime, W> {
        self.0.multi_threaded_message_loop = false;
        self.0.external_message_pump = false;
        self.0
    }
}

impl<'a, W> RuntimeAttributesBuilder<'a, MessagePumpRuntime, W> {
    pub fn build(mut self) -> RuntimeAttributes<'a, MessagePumpRuntime, W> {
        self.0.multi_threaded_message_loop = false;
        self.0.external_message_pump = true;
        self.0
    }
}

impl<'a, R, W> Deref for RuntimeAttributesBuilder<'a, R, W> {
    type Target = RuntimeAttributes<'a, R, W>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(unused_variables)]
pub trait RuntimeHandler: Send + Sync {
    /// Called when the context is initialized
    ///
    /// This callback is called when the application's context is initialized.
    ///
    /// Note that initialization only begins when the message loop starts
    /// running, so you need to drive the message loop as soon as possible after
    /// creating the runtime.
    fn on_context_initialized(&self) {}
}

#[allow(unused_variables)]
pub trait MessagePumpRuntimeHandler: RuntimeHandler {
    /// Called when scheduling message pump work
    ///
    /// This callback is called when scheduling message pump work.
    ///
    /// The `delay` parameter indicates how long to wait before calling `poll`.
    fn on_schedule_message_pump_work(&self, delay: u64) {}
}

static RUNTIME_RUNNING: AtomicBool = AtomicBool::new(false);

#[allow(unused)]
pub struct Runtime<R, W> {
    _r: PhantomData<R>,
    _w: PhantomData<W>,
    handler: ThreadSafePointer<MixRuntimeHnadler>,
    raw: Mutex<Arc<ThreadSafePointer<c_void>>>,
    multi_threaded_message_loop: bool,
}

impl<R, W> Runtime<R, W> {
    fn new(attr: &RuntimeAttributes<R, W>, handler: MixRuntimeHnadler) -> Result<Self, Error> {
        // Only one runtime is allowed per process, mainly because the runtime is bound
        // to the message loop.
        if RUNTIME_RUNNING.load(Ordering::Relaxed) {
            return Err(Error::RuntimeAlreadyExists);
        } else {
            RUNTIME_RUNNING.store(true, Ordering::Relaxed);
        }

        let custom_scheme = if let Some(attr) = attr.custom_scheme.as_ref() {
            Some(sys::CustomSchemeAttributes {
                name: attr.name.as_c_str().as_ptr(),
                domain: attr.domain.as_c_str().as_ptr(),
                factory: attr.handler.as_raw(),
            })
        } else {
            None
        };

        let options = sys::RuntimeSettings {
            cache_dir_path: attr.cache_dir_path.as_raw(),
            browser_subprocess_path: attr.browser_subprocess_path.as_raw(),
            windowless_rendering_enabled: attr.windowless_rendering_enabled,
            main_bundle_path: attr.main_bundle_path.as_raw(),
            framework_dir_path: attr.framework_dir_path.as_raw(),
            external_message_pump: attr.external_message_pump,
            multi_threaded_message_loop: attr.multi_threaded_message_loop,
            custom_scheme: custom_scheme
                .as_ref()
                .map(|it| it as *const _)
                .unwrap_or_else(|| null()),
        };

        let handler: *mut MixRuntimeHnadler = Box::into_raw(Box::new(handler));
        let ptr = unsafe {
            sys::create_runtime(
                &options,
                sys::RuntimeHandler {
                    on_context_initialized: Some(on_context_initialized),
                    on_schedule_message_pump_work: Some(on_schedule_message_pump_work),
                    context: handler as _,
                },
            )
        };

        let raw = if ptr.is_null() {
            return Err(Error::FailedToCreateRuntime);
        } else {
            Arc::new(ThreadSafePointer(ptr))
        };

        {
            let args = Args::default();

            // If using multi-threaded message loop, run the message loop in a separate
            // thread.
            if attr.multi_threaded_message_loop {
                let raw = raw.clone();
                thread::spawn(move || unsafe {
                    sys::execute_runtime(raw.as_ptr(), args.size() as _, args.as_ptr() as _);
                });
            } else {
                unsafe {
                    sys::execute_runtime(raw.as_ptr(), args.size() as _, args.as_ptr() as _);
                }
            }
        }

        Ok(Self {
            _r: PhantomData::default(),
            _w: PhantomData::default(),
            multi_threaded_message_loop: attr.multi_threaded_message_loop,
            handler: ThreadSafePointer(handler),
            raw: Mutex::new(raw),
        })
    }
}

impl<R> Runtime<R, WindowlessRenderWebView> {
    pub fn create_webview<T>(
        &self,
        url: &str,
        attr: &WebViewAttributes,
        handler: T,
    ) -> Result<WebView<WindowlessRenderWebView>, Error>
    where
        T: WindowlessRenderWebViewHandler + 'static,
    {
        WebView::new(
            &self.raw.lock(),
            url,
            attr,
            MixWebviewHnadler::WindowlessRenderWebViewHandler(Box::new(handler)),
        )
    }
}

impl<R> Runtime<R, NativeWindowWebView> {
    pub fn create_webview<T>(
        &self,
        url: &str,
        attr: &WebViewAttributes,
        handler: T,
    ) -> Result<WebView<NativeWindowWebView>, Error>
    where
        T: WebViewHandler + 'static,
    {
        WebView::new(
            &self.raw.lock(),
            url,
            attr,
            MixWebviewHnadler::WebViewHandler(Box::new(handler)),
        )
    }
}

impl<R, W> Drop for Runtime<R, W> {
    fn drop(&mut self) {
        // If using multi-threaded message loop, quit the message loop.
        if self.multi_threaded_message_loop {
            unsafe {
                sys::quit_message_loop();
            }
        }

        unsafe {
            sys::close_runtime(self.raw.lock().as_ptr());
        }

        drop(unsafe { Box::from_raw(self.handler.as_ptr()) });

        RUNTIME_RUNNING.store(false, Ordering::Relaxed);
    }
}

impl<W> Runtime<MessagePumpRuntime, W> {
    /// Drive the message loop pump on main thread
    ///
    /// This function is used to poll the message loop on main thread.
    ///
    /// Note that this function won't block the current thread, external code
    /// needs to drive the message loop pump.
    pub fn poll(&self) {
        unsafe { sys::poll_message_loop() }
    }
}

impl<W> Runtime<MainThreadRuntime, W> {
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

enum MixRuntimeHnadler {
    RuntimeHandler(Box<dyn RuntimeHandler>),
    MessagePumpRuntimeHandler(Box<dyn MessagePumpRuntimeHandler>),
}

extern "C" fn on_context_initialized(context: *mut c_void) {
    if context.is_null() {
        return;
    }

    match unsafe { &*(context as *mut MixRuntimeHnadler) } {
        MixRuntimeHnadler::RuntimeHandler(handler) => handler.on_context_initialized(),
        MixRuntimeHnadler::MessagePumpRuntimeHandler(handler) => handler.on_context_initialized(),
    }
}

extern "C" fn on_schedule_message_pump_work(delay: i64, context: *mut c_void) {
    if context.is_null() {
        return;
    }

    if let MixRuntimeHnadler::MessagePumpRuntimeHandler(handler) =
        unsafe { &*(context as *mut MixRuntimeHnadler) }
    {
        handler.on_schedule_message_pump_work(delay as u64);
    }
}
