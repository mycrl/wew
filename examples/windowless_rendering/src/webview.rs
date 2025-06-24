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
    runtime::{MessagePumpRuntimeHandler, Runtime, RuntimeHandler},
    webview::{
        WebView, WebViewAttributesBuilder, WebViewHandler, WindowHandle,
        WindowlessRenderWebViewHandler,
    },
};

use winit::{event::WindowEvent, event_loop::EventLoopProxy};

use crate::{HEIGHT, UserEvent, WIDTH, render::Render};

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
    fn on_frame(&self, texture: &[u8], width: u32, height: u32) {
        self.render.lock().render(texture, width, height);
    }

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
        let (message_pump, rx) = channel();
        let event_loop_proxy_ = event_loop_proxy.clone();
        thread::spawn(move || {
            while let Ok(delay) = rx.recv() {
                thread::sleep(Duration::from_millis(delay));

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
    fn on_context_initialized(&self) {
        let _ = self
            .event_loop_proxy
            .send_event(UserEvent::RuntimeContextInitialized);
    }
}

impl MessagePumpRuntimeHandler for RuntimeObserver {
    fn on_schedule_message_pump_work(&self, _delay: u64) {
        let _ = self.message_pump.send(1000);
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
        let mut runtime_attributes_builder =
            message_loop.create_runtime_attributes_builder::<WindowlessRenderWebView>();

        runtime_attributes_builder = runtime_attributes_builder
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
            .with_cache_dir_path(option_env!("CACHE_PATH").unwrap());

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
        if let Some(webview) = self.webview.as_ref() {
            self.event_adapter.on_winit_window_event(webview, event);
        }
    }
}
