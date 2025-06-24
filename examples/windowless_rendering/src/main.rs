mod render;
mod webview;

#[cfg(target_os = "macos")]
mod delegate;

use std::sync::Arc;

use anyhow::Result;
use wew::{MessagePumpLoop, events::Rect, webview::WindowHandle};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowAttributes, WindowId},
};

static WIDTH: u32 = 1280;
static HEIGHT: u32 = 720;
static URL: &str = "https://google.com/";

enum UserEvent {
    RuntimeContextInitialized,
    RequestRedraw,
    ImeRect(Rect),
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
        if self.window.is_some() {
            return;
        }

        self.window.replace(Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default().with_inner_size(PhysicalSize::new(WIDTH, HEIGHT)),
                )
                .unwrap(),
        ));

        if let Some(window) = self.window.as_ref() {
            window.set_ime_allowed(true);
        }

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
            UserEvent::ImeRect(rect) => {
                if let Some(window) = self.window.as_ref() {
                    window.set_ime_cursor_area(
                        PhysicalPosition::new(rect.x as f64, rect.y as f64),
                        PhysicalSize::new(rect.width as f64, rect.height as f64),
                    );
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
            _ => {}
        }

        if let Some(webview) = self.webview.as_mut() {
            webview.on_event(&event);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    if wew::is_subprocess() {
        wew::execute_subprocess();

        return Ok(());
    }

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
