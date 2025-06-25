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
    Args, CStringExt, Error, MainThreadMessageLoop, MessagePumpLoop, MultiThreadMessageLoop,
    NativeWindowWebView, ThreadSafePointer, WindowlessRenderWebView,
    request::CustomSchemeAttributes,
    sys,
    webview::{
        MixWebviewHnadler, WebView, WebViewAttributes, WebViewHandler,
        WindowlessRenderWebViewHandler,
    },
};

/// Runtime configuration attributes
#[derive(Default)]
pub struct RuntimeAttributes<R, W> {
    _r: PhantomData<R>,
    _w: PhantomData<W>,

    /// Custom scheme handler
    ///
    /// This is used to handle custom scheme requests.
    custom_scheme: Option<CustomSchemeAttributes>,

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

impl<W> RuntimeAttributes<MainThreadMessageLoop, W> {
    pub fn create_runtime<T>(self, handler: T) -> Result<Runtime<MainThreadMessageLoop, W>, Error>
    where
        T: RuntimeHandler + 'static,
    {
        Runtime::new(self, MixRuntimeHnadler::RuntimeHandler(Box::new(handler)))
    }
}

impl<W> RuntimeAttributes<MultiThreadMessageLoop, W> {
    pub fn create_runtime<T>(self, handler: T) -> Result<Runtime<MultiThreadMessageLoop, W>, Error>
    where
        T: RuntimeHandler + 'static,
    {
        Runtime::new(self, MixRuntimeHnadler::RuntimeHandler(Box::new(handler)))
    }
}

impl<W> RuntimeAttributes<MessagePumpLoop, W> {
    pub fn create_runtime<T>(self, handler: T) -> Result<Runtime<MessagePumpLoop, W>, Error>
    where
        T: MessagePumpRuntimeHandler + 'static,
    {
        Runtime::new(
            self,
            MixRuntimeHnadler::MessagePumpRuntimeHandler(Box::new(handler)),
        )
    }
}

/// Runtime configuration attributes builder
#[derive(Default)]
pub struct RuntimeAttributesBuilder<R, W>(RuntimeAttributes<R, W>);

impl<R, W> RuntimeAttributesBuilder<R, W> {
    /// Set the custom scheme handler
    ///
    /// This is used to handle custom scheme requests.
    pub fn with_custom_scheme(mut self, scheme: CustomSchemeAttributes) -> Self {
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

impl RuntimeAttributesBuilder<MultiThreadMessageLoop, NativeWindowWebView> {
    pub fn build(mut self) -> RuntimeAttributes<MultiThreadMessageLoop, NativeWindowWebView> {
        self.0.windowless_rendering_enabled = false;
        self.0.multi_threaded_message_loop = true;
        self.0.external_message_pump = false;
        self.0
    }
}

impl RuntimeAttributesBuilder<MainThreadMessageLoop, NativeWindowWebView> {
    pub fn build(mut self) -> RuntimeAttributes<MainThreadMessageLoop, NativeWindowWebView> {
        self.0.windowless_rendering_enabled = false;
        self.0.multi_threaded_message_loop = false;
        self.0.external_message_pump = false;
        self.0
    }
}

impl RuntimeAttributesBuilder<MessagePumpLoop, NativeWindowWebView> {
    pub fn build(mut self) -> RuntimeAttributes<MessagePumpLoop, NativeWindowWebView> {
        self.0.windowless_rendering_enabled = false;
        self.0.multi_threaded_message_loop = false;
        self.0.external_message_pump = true;
        self.0
    }
}

impl RuntimeAttributesBuilder<MultiThreadMessageLoop, WindowlessRenderWebView> {
    pub fn build(mut self) -> RuntimeAttributes<MultiThreadMessageLoop, WindowlessRenderWebView> {
        self.0.windowless_rendering_enabled = true;
        self.0.multi_threaded_message_loop = true;
        self.0.external_message_pump = false;
        self.0
    }
}

impl RuntimeAttributesBuilder<MainThreadMessageLoop, WindowlessRenderWebView> {
    pub fn build(mut self) -> RuntimeAttributes<MainThreadMessageLoop, WindowlessRenderWebView> {
        self.0.windowless_rendering_enabled = true;
        self.0.multi_threaded_message_loop = false;
        self.0.external_message_pump = false;
        self.0
    }
}

impl RuntimeAttributesBuilder<MessagePumpLoop, WindowlessRenderWebView> {
    pub fn build(mut self) -> RuntimeAttributes<MessagePumpLoop, WindowlessRenderWebView> {
        self.0.windowless_rendering_enabled = true;
        self.0.multi_threaded_message_loop = false;
        self.0.external_message_pump = true;
        self.0
    }
}

impl<R, W> Deref for RuntimeAttributesBuilder<R, W> {
    type Target = RuntimeAttributes<R, W>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Runtime handler
///
/// This trait is used to handle runtime events.
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

/// Message pump runtime handler
///
/// A runtime specific to the message pump mechanism.
#[allow(unused_variables)]
pub trait MessagePumpRuntimeHandler: RuntimeHandler {
    /// Called when scheduling message pump work
    ///
    /// This callback is called when scheduling message pump work.
    ///
    /// The `delay` parameter indicates how long to wait before calling `poll`,
    /// the unit is milliseconds.
    fn on_schedule_message_pump_work(&self, delay: u64) {}
}

pub(crate) static RUNTIME_RUNNING: AtomicBool = AtomicBool::new(false);

#[allow(unused)]
pub(crate) struct IRuntime<R, W> {
    _r: PhantomData<R>,
    _w: PhantomData<W>,
    initialized: Arc<AtomicBool>,
    attr: RuntimeAttributes<R, W>,
    handler: ThreadSafePointer<RuntimeContext>,
    pub(crate) raw: Mutex<Arc<ThreadSafePointer<c_void>>>,
}

impl<R, W> Drop for IRuntime<R, W> {
    fn drop(&mut self) {
        // If using multi-threaded message loop, quit the message loop.
        if self.attr.multi_threaded_message_loop {
            MainThreadMessageLoop::default().quit();
        }

        unsafe {
            sys::close_runtime(self.raw.lock().as_ptr());
        }

        drop(unsafe { Box::from_raw(self.handler.as_ptr()) });

        RUNTIME_RUNNING.store(false, Ordering::Relaxed);
    }
}

/// Global unique runtime
///
/// The runtime is used to manage multi-process models and message loops.
#[derive(Clone)]
pub struct Runtime<R, W>(pub(crate) Arc<IRuntime<R, W>>);

impl<R, W> Runtime<R, W> {
    pub(crate) fn new(
        attr: RuntimeAttributes<R, W>,
        handler: MixRuntimeHnadler,
    ) -> Result<Self, Error> {
        // Only one runtime is allowed per process, mainly because the runtime is bound
        // to the message loop.
        if RUNTIME_RUNNING.load(Ordering::Relaxed) {
            return Err(Error::RuntimeAlreadyExists);
        }

        let custom_scheme = if let Some(attr) = attr.custom_scheme.as_ref() {
            Some(sys::CustomSchemeAttributes {
                name: attr.name.as_raw(),
                domain: attr.domain.as_raw(),
                factory: attr.handler.as_raw_handler().as_ptr(),
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

        let initialized: Arc<AtomicBool> = Default::default();
        let handler: *mut RuntimeContext = Box::into_raw(Box::new(RuntimeContext {
            initialized: initialized.clone(),
            handler,
        }));

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
            Arc::new(ThreadSafePointer::new(ptr))
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

        RUNTIME_RUNNING.store(true, Ordering::Relaxed);

        Ok(Self(Arc::new(IRuntime {
            _r: PhantomData::default(),
            _w: PhantomData::default(),
            handler: ThreadSafePointer::new(handler),
            raw: Mutex::new(raw),
            initialized,
            attr,
        })))
    }
}

impl<R> Runtime<R, WindowlessRenderWebView> {
    pub fn create_webview<T>(
        &self,
        url: &str,
        attr: WebViewAttributes,
        handler: T,
    ) -> Result<WebView<R, WindowlessRenderWebView>, Error>
    where
        T: WindowlessRenderWebViewHandler + 'static,
        R: Clone,
    {
        if !self.0.initialized.load(Ordering::Relaxed) {
            return Err(Error::RuntimeNotInitialization);
        }

        WebView::new(
            self.clone(),
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
        attr: WebViewAttributes,
        handler: T,
    ) -> Result<WebView<R, NativeWindowWebView>, Error>
    where
        T: WebViewHandler + 'static,
        R: Clone,
    {
        if !self.0.initialized.load(Ordering::Relaxed) {
            return Err(Error::RuntimeNotInitialization);
        }

        WebView::new(
            self.clone(),
            url,
            attr,
            MixWebviewHnadler::WebViewHandler(Box::new(handler)),
        )
    }
}

struct RuntimeContext {
    handler: MixRuntimeHnadler,
    initialized: Arc<AtomicBool>,
}

pub(crate) enum MixRuntimeHnadler {
    RuntimeHandler(Box<dyn RuntimeHandler>),
    MessagePumpRuntimeHandler(Box<dyn MessagePumpRuntimeHandler>),
}

extern "C" fn on_context_initialized(context: *mut c_void) {
    if context.is_null() {
        return;
    }

    let context = unsafe { &*(context as *mut RuntimeContext) };

    context.initialized.store(true, Ordering::Relaxed);

    match &context.handler {
        MixRuntimeHnadler::RuntimeHandler(handler) => handler.on_context_initialized(),
        MixRuntimeHnadler::MessagePumpRuntimeHandler(handler) => handler.on_context_initialized(),
    }
}

extern "C" fn on_schedule_message_pump_work(delay: i64, context: *mut c_void) {
    if context.is_null() {
        return;
    }

    let context = unsafe { &*(context as *mut RuntimeContext) };
    if let MixRuntimeHnadler::MessagePumpRuntimeHandler(handler) = &context.handler {
        handler.on_schedule_message_pump_work(delay as u64);
    }
}
