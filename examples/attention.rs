use std::error::Error;
use std::time::Instant;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
#[allow(deprecated)]
use raw_window_handle::{HasRawWindowHandle, HasWindowHandle};
use taskbar_interface::TaskbarInterface;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

struct App {
    window: Option<Window>,
    gl_context: Option<glutin::context::PossiblyCurrentContext>,
    gl_surface: Option<glutin::surface::Surface<WindowSurface>>,
    indicator: Option<TaskbarInterface>,
    start: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            gl_context: None,
            gl_surface: None,
            indicator: None,
            start: Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title("Hello world!")
                .with_inner_size(LogicalSize::new(1024.0, 768.0));

            let template = ConfigTemplateBuilder::new();
            let display_builder =
                DisplayBuilder::new().with_window_attributes(Some(window_attributes));

            let (window, gl_config) = display_builder
                .build(event_loop, template, |configs| {
                    configs
                        .reduce(|accum, config| {
                            let transparency_check =
                                config.supports_transparency().unwrap_or(false)
                                    & !accum.supports_transparency().unwrap_or(false);

                            if transparency_check || config.num_samples() > accum.num_samples() {
                                config
                            } else {
                                accum
                            }
                        })
                        .unwrap()
                })
                .unwrap();
            let window = window.unwrap();

            println!("Picked a config with {} samples", gl_config.num_samples());

            let raw_window_handle = window.window_handle().unwrap().as_raw();

            let context_attributes = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
                .build(Some(raw_window_handle));

            let fallback_context_attributes = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::Gles(None))
                .build(Some(raw_window_handle));

            let gl_display = gl_config.display();

            let not_current_gl_context = unsafe {
                gl_display
                    .create_context(&gl_config, &context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(&gl_config, &fallback_context_attributes)
                            .expect("failed to create context")
                    })
            };

            let attrs = window
                .build_surface_attributes(SurfaceAttributesBuilder::new())
                .unwrap();
            let gl_surface = unsafe {
                gl_display
                    .create_window_surface(&gl_config, &attrs)
                    .unwrap()
            };
            let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

            #[allow(unused_mut, deprecated)]
            let mut indicator = TaskbarInterface::new(window.raw_window_handle().unwrap()).unwrap();
            #[cfg(all(unix, not(target_os = "macos")))]
            let _ = indicator.set_unity_app_uri("application://myapp.desktop");

            self.window = Some(window);
            self.gl_context = Some(gl_context);
            self.gl_surface = Some(gl_surface);
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
                if let Some(indicator) = &mut self.indicator {
                    let _ = indicator.needs_attention(self.start.elapsed().as_secs() % 10 <= 5);
                }

                if let (Some(gl_surface), Some(gl_context)) = (&self.gl_surface, &self.gl_context) {
                    let _ = gl_surface.swap_buffers(gl_context);
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

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
