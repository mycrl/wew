use std::{env::current_exe, sync::Arc};

use anyhow::Result;
use wew::{
    MessageLoopAbstract, MessagePumpLoop, WindowlessRenderWebView,
    keyboard::{EventFlags, KeyEventType, KeyboardScanCodeAdapter},
    runtime::{MessagePumpRuntimeHandler, Runtime, RuntimeHandler},
    webview::{
        KeyState, MouseAction, MouseButton, Position, WebView, WebViewAttributesBuilder,
        WebViewHandler, WindowHandle, WindowlessRenderWebViewHandler,
    },
};

use winit::{
    event::{ElementState, KeyEvent, Modifiers, MouseButton as WinitMouseButton},
    event_loop::EventLoopProxy,
    platform::scancode::PhysicalKeyExtScancode,
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
}

impl RuntimeHandler for RuntimeObserver {
    fn on_context_initialized(&self) {
        let _ = self
            .event_loop_proxy
            .send_event(UserEvent::RuntimeContextInitialized);
    }
}

impl MessagePumpRuntimeHandler for RuntimeObserver {}

pub struct Webview {
    #[allow(unused)]
    runtime: Runtime<MessagePumpLoop, WindowlessRenderWebView>,
    webview: Option<WebView<MessagePumpLoop, WindowlessRenderWebView>>,
    keyboard_adapter: KeyboardScanCodeAdapter,
    modifiers: EventFlags,
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
                        "../Frameworks/windowless-rendering Helper.app/Contents/MacOS/windowless-rendering Helper",
                    )
                    .unwrap(),
                )
                .with_cache_dir_path(option_env!("CACHE_PATH").unwrap());
        }

        let runtime = runtime_attributes_builder
            .build()
            .create_runtime(RuntimeObserver { event_loop_proxy })?;

        Ok(Self {
            keyboard_adapter: KeyboardScanCodeAdapter::default(),
            modifiers: EventFlags::None,
            webview: None,
            runtime,
        })
    }

    pub fn create_webview(&mut self, window_handle: WindowHandle, render: Render) -> Result<()> {
        let webview = self.runtime.create_webview(
            "https://www.google.com",
            WebViewAttributesBuilder::default()
                .with_width(WIDTH)
                .with_height(HEIGHT)
                .with_window_handle(window_handle)
                .build(),
            WebViewObserver { render },
        )?;

        self.webview.replace(webview);
        Ok(())
    }

    pub fn on_modifiers_change(&mut self, modifiers: &Modifiers) {
        let state = modifiers.state();

        if state.shift_key() {
            self.modifiers = EventFlags::ShiftDown;
        } else if state.control_key() {
            self.modifiers = EventFlags::ControlDown;
        } else if state.alt_key() {
            self.modifiers = EventFlags::AltDown;
        } else {
            self.modifiers = EventFlags::None;
        }
    }

    pub fn on_keyboard_input(&mut self, event: &KeyEvent) {
        println!("winit keyboard event: {:?}", event);

        if let Some(code) = event.physical_key.to_scancode() {
            self.keyboard_adapter.get_key_event(
                code,
                match event.state {
                    ElementState::Pressed => KeyEventType::KeyDown,
                    ElementState::Released => KeyEventType::KeyUp,
                },
                self.modifiers,
            );
        } else {
            println!("winit keyboard event get scancode return none!");
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
                match state {
                    ElementState::Pressed => KeyState::Down,
                    ElementState::Released => KeyState::Up,
                },
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
}
