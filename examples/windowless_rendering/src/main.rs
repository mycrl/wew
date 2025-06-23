mod render;
mod webview;

#[cfg(target_os = "macos")]
mod delegate;

use std::sync::Arc;

use anyhow::Result;
use wew::{MessagePumpLoop, webview::WindowHandle};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowAttributes, WindowId},
};

static WIDTH: u32 = 1280;
static HEIGHT: u32 = 720;
static URL: &str = "https://keyboard.bmcx.com/";

enum UserEvent {
    RuntimeContextInitialized,
    RequestRedraw,
}

struct App {
    message_loop: MessagePumpLoop,
    window: Option<Arc<Window>>,
    webview: Option<webview::Webview>,
    event_loop_proxy: Arc<EventLoopProxy<UserEvent>>,
}

impl App {
    fn new(event_loop_proxy: Arc<EventLoopProxy<UserEvent>>) -> Self {
        Self {
            event_loop_proxy,
            message_loop: MessagePumpLoop::default(),
            webview: None,
            window: None,
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window.replace(Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default().with_inner_size(PhysicalSize::new(WIDTH, HEIGHT)),
                )
                .unwrap(),
        ));

        self.webview.replace(
            webview::Webview::new(self.event_loop_proxy.clone(), &self.message_loop).unwrap(),
        );
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::RuntimeContextInitialized => {
                if let Some(window) = self.window.as_ref() {
                    let render = pollster::block_on(render::Render::new(window.clone())).unwrap();

                    let window_handle =
                        WindowHandle::new(match window.window_handle().unwrap().as_raw() {
                            RawWindowHandle::Win32(it) => it.hwnd.get() as _,
                            RawWindowHandle::AppKit(it) => it.ns_view.as_ptr() as _,
                            _ => unimplemented!("Unsupported window handle type"),
                        });

                    if let Some(webview) = self.webview.as_mut() {
                        webview.create_webview(URL, window_handle, render).unwrap();
                    }
                }
            }
            UserEvent::RequestRedraw => {
                if let Some(window) = self.window.as_ref() {
                    window.pre_present_notify();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.message_loop.poll();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                if let Some(webview) = self.webview.as_mut() {
                    webview.on_modifiers_change(&modifiers);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(webview) = self.webview.as_mut() {
                    webview.on_keyboard_input(&event);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(webview) = self.webview.as_ref() {
                    webview.on_mouse_input(state, button);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(webview) = self.webview.as_ref() {
                    let (x, y) = match delta {
                        MouseScrollDelta::PixelDelta(pos) => (pos.x as i32, pos.y as i32),
                        MouseScrollDelta::LineDelta(x, y) => ((x * 20.0) as i32, (y * 20.0) as i32),
                    };

                    webview.on_mouse_wheel(x, y);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(webview) = self.webview.as_ref() {
                    webview.on_mouse_move(position.x as i32, position.y as i32);
                }
            }
            WindowEvent::Focused(state) => {
                if let Some(webview) = self.webview.as_ref() {
                    webview.on_focus(state);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let event_loop_proxy = Arc::new(event_loop.create_proxy());

    event_loop.set_control_flow(ControlFlow::Wait);

    // fix cef send event handle for winit 0.29
    #[cfg(target_os = "macos")]
    unsafe {
        delegate::inject_delegate();
    }

    event_loop.run_app(&mut App::new(event_loop_proxy))?;
    Ok(())
}
