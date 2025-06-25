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

use log::LevelFilter;
use parking_lot::Mutex;

use crate::{
    Error, MainThreadMessageLoop, MessagePumpLoop, MultiThreadMessageLoop, NativeWindowWebView,
    WindowlessRenderWebView,
    request::CustomSchemeAttributes,
    sys,
    utils::{Args, CStringExt, ThreadSafePointer, is_main_thread},
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
    cache_path: Option<CString>,

    /// The root directory for installation-specific data and the parent
    /// directory for profile-specific data.
    root_cache_path: Option<CString>,

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

    /// Whether to disable command line arguments
    command_line_args_disabled: bool,

    /// Whether to persist session cookies
    persist_session_cookies: bool,

    /// The user agent
    user_agent: Option<CString>,

    /// The user agent product
    user_agent_product: Option<CString>,

    /// The locale
    locale: Option<CString>,

    /// The log file
    log_file: Option<CString>,

    /// The log severity
    log_severity: Option<LevelFilter>,

    /// The javascript flags
    javascript_flags: Option<CString>,

    /// The resources directory path
    resources_dir_path: Option<CString>,

    /// The locales directory path
    locales_dir_path: Option<CString>,

    /// The background color
    background_color: u32,

    /// Whether to disable signal handlers
    disable_signal_handlers: bool,
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
    pub fn with_cache_path(mut self, value: &str) -> Self {
        self.0.cache_path = Some(CString::new(value).unwrap());
        self
    }

    /// Set the root directory for installation-specific data and the parent
    /// directory for profile-specific data.
    pub fn with_root_cache_path(mut self, value: &str) -> Self {
        self.0.root_cache_path = Some(CString::new(value).unwrap());
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

    /// Set the user agent
    pub fn with_user_agent(mut self, value: &str) -> Self {
        self.0.user_agent = Some(CString::new(value).unwrap());
        self
    }

    /// Set the user agent product
    pub fn with_user_agent_product(mut self, value: &str) -> Self {
        self.0.user_agent_product = Some(CString::new(value).unwrap());
        self
    }

    /// Set the locale
    pub fn with_locale(mut self, value: &str) -> Self {
        self.0.locale = Some(CString::new(value).unwrap());
        self
    }

    /// Set the log file
    pub fn with_log_file(mut self, value: &str) -> Self {
        self.0.log_file = Some(CString::new(value).unwrap());
        self
    }

    /// Set the log severity
    pub fn with_log_severity(mut self, value: LevelFilter) -> Self {
        self.0.log_severity = Some(value);

        self
    }

    /// Set the javascript flags
    pub fn with_javascript_flags(mut self, value: &str) -> Self {
        self.0.javascript_flags = Some(CString::new(value).unwrap());
        self
    }

    /// Set the resources directory path
    pub fn with_resources_dir_path(mut self, value: &str) -> Self {
        self.0.resources_dir_path = Some(CString::new(value).unwrap());
        self
    }

    /// Set the locales directory path
    pub fn with_locales_dir_path(mut self, value: &str) -> Self {
        self.0.locales_dir_path = Some(CString::new(value).unwrap());
        self
    }

    /// Set the background color
    pub fn with_background_color(mut self, value: u32) -> Self {
        self.0.background_color = value;
        self
    }

    /// Set whether to disable signal handlers
    pub fn with_disable_signal_handlers(mut self, value: bool) -> Self {
        self.0.disable_signal_handlers = value;
        self
    }

    /// Set whether to disable command line arguments
    pub fn with_command_line_args_disabled(mut self, value: bool) -> Self {
        self.0.command_line_args_disabled = value;
        self
    }

    /// Set whether to persist session cookies
    pub fn with_persist_session_cookies(mut self, value: bool) -> Self {
        self.0.persist_session_cookies = value;
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
            MainThreadMessageLoop.quit();
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

        if !is_main_thread() {
            return Err(Error::NonUIThread);
        }

        let custom_scheme = attr
            .custom_scheme
            .as_ref()
            .map(|attr| sys::CustomSchemeAttributes {
                name: attr.name.as_raw(),
                domain: attr.domain.as_raw(),
                factory: attr.handler.as_raw_handler().as_ptr(),
            });

        let options = sys::RuntimeSettings {
            cache_path: attr.cache_path.as_raw(),
            root_cache_path: attr.root_cache_path.as_raw(),
            background_color: attr.background_color,
            command_line_args_disabled: attr.command_line_args_disabled,
            disable_signal_handlers: attr.disable_signal_handlers,
            javascript_flags: attr.javascript_flags.as_raw(),
            persist_session_cookies: attr.persist_session_cookies,
            user_agent: attr.user_agent.as_raw(),
            user_agent_product: attr.user_agent_product.as_raw(),
            locale: attr.locale.as_raw(),
            log_file: attr.log_file.as_raw(),
            resources_dir_path: attr.resources_dir_path.as_raw(),
            locales_dir_path: attr.locales_dir_path.as_raw(),
            browser_subprocess_path: attr.browser_subprocess_path.as_raw(),
            windowless_rendering_enabled: attr.windowless_rendering_enabled,
            main_bundle_path: attr.main_bundle_path.as_raw(),
            framework_dir_path: attr.framework_dir_path.as_raw(),
            external_message_pump: attr.external_message_pump,
            multi_threaded_message_loop: attr.multi_threaded_message_loop,
            log_severity: match attr.log_severity.unwrap_or(LevelFilter::Off) {
                LevelFilter::Off => sys::LogLevel::WEBVIEW_LOG_DISABLE,
                LevelFilter::Info => sys::LogLevel::WEBVIEW_LOG_INFO,
                LevelFilter::Error => sys::LogLevel::WEBVIEW_LOG_ERROR,
                LevelFilter::Warn => sys::LogLevel::WEBVIEW_LOG_WARNING,
                LevelFilter::Debug => sys::LogLevel::WEBVIEW_LOG_DEBUG,
                LevelFilter::Trace => sys::LogLevel::WEBVIEW_LOG_VERBOSE,
            },
            custom_scheme: custom_scheme
                .as_ref()
                .map(|it| it as *const _)
                .unwrap_or_else(null),
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
            attr,
            initialized,
            raw: Mutex::new(raw),
            handler: ThreadSafePointer::new(handler),
            _r: PhantomData,
            _w: PhantomData,
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
