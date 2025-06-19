use std::{
    sync::mpsc::{Sender, channel},
    thread,
    time::Duration,
};

use minifb::{MouseButton, MouseMode, Window, WindowOptions};
use webview::{
    ActionState, MouseAction, MouseButtons, Position, Runtime, RuntimeAttributesBuilder,
    RuntimeHandler, WebViewAttributes, WebViewHandler,
};

struct WebViewObserver {
    sender: Sender<Vec<u8>>,
}

impl WebViewHandler for WebViewObserver {
    fn on_frame(&self, pixel: &[u8], _: u32, _: u32) {
        self.sender.send(pixel.to_vec()).unwrap();
    }
}

struct RuntimeObserver;

impl RuntimeHandler for RuntimeObserver {}

fn main() -> anyhow::Result<()> {
    if Runtime::is_subprocess() {
        if !Runtime::execute_subprocess() {
            panic!("subprocess execute failed!");
        }
    }

    let runtime = RuntimeAttributesBuilder::default()
        .build()
        .create_runtime(RuntimeObserver)?;

    let (sender, receiver) = channel();
    let settings = WebViewAttributes::default();
    let webview =
        runtime.create_webview("https://google.com", &settings, WebViewObserver { sender })?;

    let mut window = Window::new(
        "google.com",
        settings.width as usize,
        settings.height as usize,
        WindowOptions::default(),
    )?;

    window.set_target_fps(settings.windowless_frame_rate as usize);

    let mut frame = vec![0u8; (settings.width * settings.height * 4) as usize];
    loop {
        if let Some((x, y)) = window
            .get_mouse_pos(MouseMode::Clamp)
            .map(|(x, y)| (x as i32, y as i32))
        {
            if window.get_mouse_down(MouseButton::Left) {
                webview.mouse(MouseAction::Click(
                    MouseButtons::kLeft,
                    ActionState::Down,
                    Some(Position { x, y }),
                ));

                webview.mouse(MouseAction::Click(
                    MouseButtons::kLeft,
                    ActionState::Up,
                    None,
                ));
            }
        }

        if let Ok(f) = receiver.try_recv() {
            frame = f;
        }

        let (_, shorts, _) = unsafe { frame.align_to::<u32>() };
        window.update_with_buffer(shorts, settings.width as usize, settings.height as usize)?;
        thread::sleep(Duration::from_millis(
            1000 / settings.windowless_frame_rate as u64,
        ));
    }
}
