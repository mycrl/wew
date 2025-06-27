// This module doesn't need much attention. It's just an implementation that
// renders the webview's output to a window again. If you don't need to focus on
// how to render to a window, you can ignore this module.
mod render;
mod webview;

use std::sync::Arc;

use anyhow::Result;
use wew::{MessagePumpLoop, Rect, webview::WindowHandle};
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
static URL: &str = "https:/google.com";

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

        // Create a window for the webview
        //
        // Since we're using off-screen rendering, the webview won't create its own
        // native window. You need to handle rendering yourself, so you need to
        // create a native window to handle events and render the webview's
        // content.
        self.window.replace(Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default().with_inner_size(PhysicalSize::new(WIDTH, HEIGHT)),
                )
                .unwrap(),
        ));

        // Allow input method usage on this window
        if let Some(window) = self.window.as_ref() {
            window.set_ime_allowed(true);
        }

        // Create webview instance
        self.webview.replace(
            webview::Webview::new(self.event_loop_proxy.clone(), &self.message_loop).unwrap(),
        );
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            // When the webview's runtime context initialization is complete, this event will be
            // sent.
            //
            // Here we create the webview instance after the webview's runtime is created. This is
            // necessary because creating a webview can only be done after the runtime creation is
            // complete.
            UserEvent::RuntimeContextInitialized => {
                if let Some(window) = self.window.as_ref() {
                    // Create renderer
                    let render = pollster::block_on(render::Render::new(window.clone())).unwrap();

                    // Get the current winit window's native window handle to pass to the webview
                    // for binding relationships with popup windows, etc.
                    let window_handle =
                        WindowHandle::new(match window.window_handle().unwrap().as_raw() {
                            RawWindowHandle::Win32(it) => it.hwnd.get() as _,
                            RawWindowHandle::AppKit(it) => it.ns_view.as_ptr() as _,
                            _ => unimplemented!("Unsupported window handle type"),
                        });

                    // Create webview instance
                    if let Some(webview) = self.webview.as_mut() {
                        webview.create_webview(URL, window_handle, render).unwrap();
                    }
                }
            }
            UserEvent::RequestRedraw => {
                // The runtime requests to drive the message loop once. Here we request window
                // redraw, which will trigger a redraw event.
                if let Some(window) = self.window.as_ref() {
                    window.pre_present_notify();
                }
            }
            UserEvent::ImeRect(rect) => {
                // The webview reports the input method cursor position, set it to the winit
                // window.
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
                // Safe shutdown webview
                drop(self.webview.take());

                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if self.webview.is_some() {
                    // When requesting redraw, also drive the webview's message loop.
                    self.message_loop.poll();
                }
            }
            _ => {
                if let Some(webview) = self.webview.as_mut() {
                    webview.on_event(&event);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // The message pump needs to be driven, so if we're about to wait for events, we
        // still need to request a redraw once so the webview can schedule the
        // next task.
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let event_loop_proxy = Arc::new(event_loop.create_proxy());

    event_loop.set_control_flow(ControlFlow::Wait);

    // For macOS, we need to inject a delegate for winit, otherwise CEF cannot
    // handle macOS text selection events.
    #[cfg(target_os = "macos")]
    wew::utils::startup_nsapplication();

    event_loop.run_app(&mut App::new(event_loop_proxy))?;
    Ok(())
}
