use framework::{renderer::renderer::Renderer, Framework};
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopBuilder},
    window::Window,
};

use crate::{
    app_loop::{AppContext, AppLoop},
    app_state, InputState,
};

pub struct AppDescription {
    pub initial_width: u32,
    pub initial_height: u32,
}

struct AppState<T: AppLoop> {
    instance: T,
    window: Window,
}

pub struct Application<T: AppLoop, U: 'static> {
    event_loop: EventLoop<U>,
    state: AppState<T>,
}

impl<T: AppLoop + 'static, U: 'static> Application<T, U> {
    fn new(app_loop: T, description: AppDescription) -> anyhow::Result<Self> {
        let event_loop = EventLoopBuilder::with_user_event().build();
        let window = winit::window::WindowBuilder::new()
            .with_title("Image editor")
            .with_min_inner_size(PhysicalSize {
                width: description.initial_width,
                height: description.initial_height,
            })
            .build(&event_loop)?;

        Ok(Self {
            state: AppState {
                instance: app_loop,
                window,
            },
            event_loop,
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let input_state = InputState::default();
        let input_state = Box::leak(Box::new(input_state));

        let framework = Framework::new(&wgpu::DeviceDescriptor {
            label: Some("Image Editor framework"),
            features: wgpu::Features::DEPTH32FLOAT_STENCIL8,
            limits: wgpu::Limits {
                max_bind_groups: 5,
                ..Default::default()
            },
        })?;
        let mut framework = Box::leak(Box::new(framework));

        let state = Box::leak(Box::new(self.state));

        state.instance.boot(&mut framework);
        let renderer = Box::leak(Box::new(Renderer::new(&mut framework)));

        self.event_loop.run(move |event, _, control_flow| {
            input_state.update(&event);
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested if state.instance.should_shutdown() => {
                        *control_flow = winit::event_loop::ControlFlow::ExitWithCode(0);
                    }
                    WindowEvent::Resized(new_size) => {
                        state.instance.on_resized(new_size, framework);
                    }
                    _ => {}
                },
                winit::event::Event::UserEvent(_) => {}
                winit::event::Event::MainEventsCleared => {
                    state.instance.update(AppContext {
                        renderer,
                        framework,
                    });
                }
                winit::event::Event::RedrawEventsCleared => {
                    if state.instance.should_render() {
                        state.window.request_redraw();
                    }
                }
                winit::event::Event::RedrawRequested(_) => {
                    state.instance.render(AppContext {
                        renderer,
                        framework,
                    });
                }
                _ => {}
            };
            *control_flow = winit::event_loop::ControlFlow::Poll
        });
    }
}
