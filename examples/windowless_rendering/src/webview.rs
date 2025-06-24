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
use wew::{
    MessageLoopAbstract, MessagePumpLoop, WindowlessRenderWebView,
    keyboard::{KeyboardModifiers, WinitKeyboardAdapter},
    runtime::{MessagePumpRuntimeHandler, Runtime, RuntimeHandler},
    webview::{
        MouseAction, MouseButton, Position, WebView, WebViewAttributesBuilder, WebViewHandler,
        WindowHandle, WindowlessRenderWebViewHandler,
    },
};

use winit::{
    event::{ElementState, KeyEvent, Modifiers, MouseButton as WinitMouseButton},
    event_loop::EventLoopProxy,
};

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
    render: Render,
}

impl WebViewHandler for WebViewObserver {}

impl WindowlessRenderWebViewHandler for WebViewObserver {
    fn on_frame(&self, texture: &[u8], _width: u32, _height: u32) {
        self.render.render(texture);
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
    modifiers: KeyboardModifiers,
}

impl Webview {
    pub fn new(
        event_loop_proxy: Arc<EventLoopProxy<UserEvent>>,
        message_loop: &MessagePumpLoop,
    ) -> Result<Self> {
        let mut runtime_attributes_builder =
            message_loop.create_runtime_attributes_builder::<WindowlessRenderWebView>();

        if cfg!(target_os = "macos") {
            runtime_attributes_builder = runtime_attributes_builder
                .with_browser_subprocess_path(
                    &join_with_current_dir(
                        if cfg!(target_os = "windows") {
                            "hylarana-app-helper.exe"
                        } else if cfg!(target_os = "macos") {
                            "../Frameworks/windowless-rendering Helper.app/Contents/MacOS/windowless-rendering Helper"
                        } else {
                            unimplemented!()
                        }
                    )
                    .unwrap(),
                )
                .with_cache_dir_path(option_env!("CACHE_PATH").unwrap());
        }

        let runtime = runtime_attributes_builder
            .build()
            .create_runtime(RuntimeObserver::new(event_loop_proxy))?;

        Ok(Self {
            modifiers: KeyboardModifiers::None,
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
            WebViewObserver { render },
        )?;

        webview.focus(true);

        self.webview.replace(webview);
        Ok(())
    }

    pub fn on_modifiers_change(&mut self, modifiers: &Modifiers) {
        let state = modifiers.state();

        if state.shift_key() {
            self.modifiers = KeyboardModifiers::Shift;
        } else if state.control_key() {
            self.modifiers = KeyboardModifiers::Ctrl;
        } else if state.alt_key() {
            self.modifiers = KeyboardModifiers::Alt;
        } else {
            self.modifiers = KeyboardModifiers::None;
        }
    }

    pub fn on_keyboard_input(&mut self, event: &KeyEvent) {
        if let Some(webview) = self.webview.as_ref() {
            for it in WinitKeyboardAdapter::get_key_event(event) {
                webview.keyboard(&it);
            }
        }
    }

    pub fn on_mouse_input(&self, state: ElementState, button: WinitMouseButton) {
        if let Some(webview) = self.webview.as_ref() {
            webview.mouse(&MouseAction::Click(
                match button {
                    WinitMouseButton::Left => MouseButton::Left,
                    WinitMouseButton::Right => MouseButton::Right,
                    _ => MouseButton::Middle,
                },
                state.is_pressed(),
                None,
            ));
        }
    }

    pub fn on_mouse_wheel(&self, x: i32, y: i32) {
        if let Some(webview) = self.webview.as_ref() {
            webview.mouse(&MouseAction::Wheel(Position { x, y }));
        }
    }

    pub fn on_mouse_move(&self, x: i32, y: i32) {
        if let Some(webview) = self.webview.as_ref() {
            webview.mouse(&MouseAction::Move(Position { x, y }));
        }
    }

    pub fn on_focus(&self, state: bool) {
        if let Some(webview) = self.webview.as_ref() {
            webview.focus(state);
        }
    }
}
