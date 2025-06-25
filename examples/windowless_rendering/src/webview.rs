use std::{
    env::current_exe,
    sync::{
        Arc,
        mpsc::{Sender, channel},
    },
    thread,
    time::Duration,
};

use anyhow::Result;
use parking_lot::Mutex;
use wew::{
    MessageLoopAbstract, MessagePumpLoop, WindowlessRenderWebView,
    events::{EventAdapter, Rect},
    log::LevelFilter,
    runtime::{MessagePumpRuntimeHandler, Runtime, RuntimeHandler},
    webview::{
        WebView, WebViewAttributesBuilder, WebViewHandler, WindowHandle,
        WindowlessRenderWebViewHandler,
    },
};

use winit::{event::WindowEvent, event_loop::EventLoopProxy};

use crate::{HEIGHT, UserEvent, WIDTH, render::Render};

// Join path, but not at the top level directory, which is the directory where
// the current executable is located.
fn join_with_current_dir(chlid: &str) -> Option<String> {
    let mut path = current_exe().ok()?;

    path.pop();
    Some(
        path.join(chlid)
            .canonicalize()
            .ok()?
            .to_str()?
            .to_string()
            .replace("\\\\?\\", "")
            .replace("\\", "/"),
    )
}

pub struct WebViewObserver {
    event_loop_proxy: Arc<EventLoopProxy<UserEvent>>,
    render: Mutex<Render>,
}

impl WebViewHandler for WebViewObserver {}

impl WindowlessRenderWebViewHandler for WebViewObserver {
    // When the webview needs to render, this function will be called.
    //
    // Here we call the renderer to render the webview's output to the window.
    fn on_frame(&self, texture: &[u8], width: u32, height: u32) {
        self.render.lock().render(texture, width, height);
    }

    // Notify winit of the input cursor position.
    fn on_ime_rect(&self, rect: Rect) {
        let _ = self.event_loop_proxy.send_event(UserEvent::ImeRect(rect));
    }
}

pub struct RuntimeObserver {
    event_loop_proxy: Arc<EventLoopProxy<UserEvent>>,
    message_pump: Sender<u64>,
}

impl RuntimeObserver {
    fn new(event_loop_proxy: Arc<EventLoopProxy<UserEvent>>) -> Self {
        // Handle `schedule_message_pump_work` events in a separate thread, and dispatch
        // to winit's message loop after waiting for the specified delay.
        //
        // This is necessary because you need to schedule the next time to drive the
        // runtime according to the webview's rendering mechanism.
        let (message_pump, rx) = channel();
        let event_loop_proxy_ = event_loop_proxy.clone();
        thread::spawn(move || {
            while let Ok(delay) = rx.recv() {
                if delay > 0 {
                    thread::sleep(Duration::from_millis(delay));
                }

                let _ = event_loop_proxy_.send_event(UserEvent::RequestRedraw);
            }
        });

        Self {
            event_loop_proxy,
            message_pump,
        }
    }
}

impl RuntimeHandler for RuntimeObserver {
    // The runtime has been created successfully and can proceed with the next
    // operations. Here we notify this event to the outside.
    fn on_context_initialized(&self) {
        let _ = self
            .event_loop_proxy
            .send_event(UserEvent::RuntimeContextInitialized);
    }
}

impl MessagePumpRuntimeHandler for RuntimeObserver {
    // Queue the runtime driving event into the queue.
    fn on_schedule_message_pump_work(&self, delay: u64) {
        let _ = self.message_pump.send(delay);
    }
}

pub struct Webview {
    #[allow(unused)]
    runtime: Runtime<MessagePumpLoop, WindowlessRenderWebView>,
    webview: Option<WebView<MessagePumpLoop, WindowlessRenderWebView>>,
    event_loop_proxy: Arc<EventLoopProxy<UserEvent>>,
    event_adapter: EventAdapter,
}

impl Webview {
    pub fn new(
        event_loop_proxy: Arc<EventLoopProxy<UserEvent>>,
        message_loop: &MessagePumpLoop,
    ) -> Result<Self> {
        // Create runtime attributes builder
        //
        // Here we specify that the webview type is off-screen rendering.
        let mut runtime_attributes_builder =
            message_loop.create_runtime_attributes_builder::<WindowlessRenderWebView>();

        runtime_attributes_builder = runtime_attributes_builder
            // Since it's a separate executable file as a subprocess, we need to specify the path 
            // to the subprocess executable file here.
            .with_browser_subprocess_path(
                &join_with_current_dir(
                    if cfg!(target_os = "windows") {
                        "./windowless-rendering-helper.exe"
                    } else if cfg!(target_os = "macos") {
                        "../Frameworks/windowless-rendering Helper.app/Contents/MacOS/windowless-rendering Helper"
                    } else {
                        unimplemented!()
                    }
                )
                .unwrap(),
            )
            // Set cache path, here we use environment variables passed by the build script.
            .with_root_cache_path(option_env!("CACHE_PATH").unwrap())
            .with_cache_path(option_env!("CACHE_PATH").unwrap())
            .with_log_severity(LevelFilter::Info);

        // Create runtime, wait for the `on_context_initialized` event to be triggered
        // before considering the creation successful.
        let runtime = runtime_attributes_builder
            .build()
            .create_runtime(RuntimeObserver::new(event_loop_proxy.clone()))?;

        Ok(Self {
            event_loop_proxy,
            event_adapter: EventAdapter::default(),
            webview: None,
            runtime,
        })
    }

    pub fn create_webview(
        &mut self,
        url: &str,
        window_handle: WindowHandle,
        render: Render,
    ) -> Result<()> {
        // Create webview instance
        //
        // Use the same size and window handle as winit.
        let webview = self.runtime.create_webview(
            url,
            WebViewAttributesBuilder::default()
                .with_width(WIDTH)
                .with_height(HEIGHT)
                .with_window_handle(window_handle)
                .build(),
            WebViewObserver {
                event_loop_proxy: self.event_loop_proxy.clone(),
                render: Mutex::new(render),
            },
        )?;

        self.webview.replace(webview);
        Ok(())
    }

    pub fn on_event(&mut self, event: &WindowEvent) {
        // For winit event types, wew provides corresponding adapters. Here we directly
        // use the winit event type adapter to handle winit events to drive the
        // webview.
        if let Some(webview) = self.webview.as_ref() {
            self.event_adapter.on_winit_window_event(webview, event);
        }
    }
}
