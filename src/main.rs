mod renderer;
mod vertex;

use std::sync::Arc;

use anyhow::Result;
use renderer::Renderer;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("wgpu Triangle"))
                .unwrap(),
        );

        let renderer = match pollster::block_on(Renderer::new(event_loop, window.clone())) {
            Ok(renderer) => renderer,
            Err(error) => {
                eprintln!("Renderer initialization failed: {error:#}");
                event_loop.exit();
                return;
            }
        };

        self.window = Some(window);
        self.renderer = Some(renderer);

        self.renderer.as_ref().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
            }

            WindowEvent::RedrawRequested => {
                if let Err(error) = renderer.render() {
                    eprintln!("Rendering failed: {error:#}");
                    event_loop.exit();
                }
            }

            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();

    event_loop.run_app(&mut app)?;

    Ok(())
}
