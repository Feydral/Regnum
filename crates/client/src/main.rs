mod camera;
mod depth;
mod frame;
mod game;
mod gpu_assets;
mod mesh;
mod renderer;
mod vertex;

use std::{sync::Arc, time::Instant};

use anyhow::Result;
use renderer::Renderer;

use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

use game::Game;
use regnum_assets::AssetManager;

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,

    assets: AssetManager,
    game: Game,

    captured: bool,
    last_frame: Instant,
}

impl App {
    fn new() -> Self {
        let mut assets = AssetManager::default();
        let game = Game::new(&mut assets);

        Self {
            window: None,
            renderer: None,
            assets,
            game,
            captured: false,
            last_frame: Instant::now(),
        }
    }

    fn set_captured(&mut self, captured: bool) {
        let Some(window) = &self.window else { return };

        if captured {
            let grabbed = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            if grabbed.is_err() {
                return;
            }
            window.set_cursor_visible(false);
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.game.release_input();
        }
        self.captured = captured;
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Regnum"))
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
        self.last_frame = Instant::now();
        self.set_captured(true);

        self.renderer.as_ref().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.renderer.is_none() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                self.renderer
                    .as_mut()
                    .unwrap()
                    .resize(size.width, size.height);
            }

            WindowEvent::Focused(false) => self.set_captured(false),

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } if !self.captured => self.set_captured(true),

            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };
                let pressed = event.state == ElementState::Pressed;

                if event.repeat {
                    return;
                }

                if pressed && code == KeyCode::Escape {
                    if self.captured {
                        self.set_captured(false);
                    } else {
                        event_loop.exit();
                    }
                    return;
                }

                self.game.key(code, pressed, &mut self.assets);
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32().min(0.1);
                self.last_frame = now;

                self.game.update(dt);
                let frame = self.game.render();

                let renderer = self.renderer.as_mut().unwrap();
                if let Err(error) = renderer.render(&frame, &mut self.assets) {
                    eprintln!("Rendering failed: {error:#}");
                    event_loop.exit();
                }
            }

            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if self.captured
            && let DeviceEvent::MouseMotion { delta: (dx, dy) } = event
        {
            self.game.mouse_motion(dx, dy);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(renderer) = &self.renderer {
            renderer.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
