use std::{
    sync::mpsc::{channel, Sender},
    thread,
    time::Duration,
};

use minifb::{MouseButton, MouseMode, Window, WindowOptions};
use tokio::runtime::Runtime;
use webview::{
    execute_subprocess, is_subprocess, ActionState, MouseAction, MouseButtons, Observer,
    PageOptions, Position, Webview, WebviewOptions,
};

struct PageObserver {
    sender: Sender<Vec<u8>>,
}

impl Observer for PageObserver {
    fn on_frame(&self, buf: &[u8], _: u32, _: u32) {
        self.sender.send(buf.to_vec()).unwrap();
    }
}

async fn run_cef() -> anyhow::Result<()> {
    let (sender, receiver) = channel();
    let app = Webview::new(&WebviewOptions {
        cache_path: None,
        browser_subprocess_path: None,
        scheme_path: None,
    })
    .await?;

    let settings = PageOptions {
        url: "https://google.com",
        frame_rate: 30,
        width: 800,
        height: 600,
        device_scale_factor: 1.0,
        is_offscreen: true,
        window_handle: None,
    };

    tokio::spawn(async move {
        let browser = app.create_page(&settings, PageObserver { sender }).await?;

        let mut window = Window::new(
            "simple",
            settings.width as usize,
            settings.height as usize,
            WindowOptions::default(),
        )?;

        window.limit_update_rate(Some(Duration::from_millis(
            1000 / settings.frame_rate as u64,
        )));

        let mut frame = vec![0u8; (settings.width * settings.height * 4) as usize];
        loop {
            if let Some((x, y)) = window
                .get_mouse_pos(MouseMode::Clamp)
                .map(|(x, y)| (x as i32, y as i32))
            {
                if window.get_mouse_down(MouseButton::Left) {
                    browser.on_mouse(MouseAction::Click(
                        MouseButtons::kLeft,
                        ActionState::Down,
                        Some(Position { x, y }),
                    ));

                    browser.on_mouse(MouseAction::Click(
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
            thread::sleep(Duration::from_millis(1000 / settings.frame_rate as u64));
        }

        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    // app.closed().await;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    if is_subprocess() {
        execute_subprocess();
    }

    Runtime::new()?.block_on(async { run_cef().await })?;
    Ok(())
}
