use std::time::Instant;

#[allow(deprecated)]
use raw_window_handle::HasRawWindowHandle;
use taskbar_interface::TaskbarInterface;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

struct App {
    window: Option<Window>,
    indicator: Option<TaskbarInterface>,
    start: Instant,
    last_progress_update: Instant,
}

impl App {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            window: None,
            indicator: None,
            start: now,
            last_progress_update: now,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = WindowAttributes::default();
            let window = event_loop.create_window(window_attributes).unwrap();

            #[allow(unused_mut, deprecated)]
            let mut indicator = TaskbarInterface::new(window.raw_window_handle().unwrap()).unwrap();
            #[cfg(all(unix, not(target_os = "macos")))]
            let _ = indicator.set_unity_app_uri("application://myapp.desktop");

            self.window = Some(window);
            self.indicator = Some(indicator);
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
                let now = Instant::now();
                if now.duration_since(self.last_progress_update).as_millis() >= 500 {
                    if let Some(indicator) = &mut self.indicator {
                        let progress = self.start.elapsed().as_secs_f64().fract();
                        let _ = indicator.set_progress(progress);
                    }
                    self.last_progress_update = now;
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
