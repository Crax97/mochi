use std::marker::PhantomData;

use framework::{renderer::renderer::Renderer, Framework};
use wgpu::{Surface, SurfaceConfiguration, TextureViewDescriptor};
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopBuilder},
    window::Window,
};

use crate::{
    app_loop::{AppContext, AppLoop},
    ActionMap, AppBoot, InputState,
};

pub struct AppDescription {
    pub initial_width: u32,
    pub initial_height: u32,
}

struct AppState<E, A: Clone, T: AppLoop<E, A>> {
    _ph_data: PhantomData<E>,
    instance: T,
    window: Window,
    framework: Framework,
    renderer: Renderer,
    input_state: InputState,
    action_map: ActionMap<A>,
    surface: Surface,
    surface_configuration: SurfaceConfiguration,
}

pub struct Application<U: 'static> {
    event_loop: EventLoop<U>,
    window: Window,
}

impl<U: 'static> Application<U> {
    pub fn new(description: AppDescription) -> anyhow::Result<Self> {
        let event_loop = EventLoopBuilder::with_user_event().build();
        let window = winit::window::WindowBuilder::new()
            .with_title("Image editor")
            .with_min_inner_size(PhysicalSize {
                width: description.initial_width,
                height: description.initial_height,
            })
            .build(&event_loop)?;

        Ok(Self { window, event_loop })
    }

    pub fn run<A: Clone + 'static, T: AppLoop<U, A> + 'static>(self) -> anyhow::Result<()> {
        let input_state = InputState::new();

        let mut framework = Framework::new(&wgpu::DeviceDescriptor {
            label: Some("Image Editor framework"),
            features: wgpu::Features::DEPTH32FLOAT_STENCIL8,
            limits: wgpu::Limits {
                max_bind_groups: 5,
                ..Default::default()
            },
        })?;

        let surface = unsafe { framework.instance.create_surface(&self.window) };
        let surface_configuration = application_functions::create_surface(
            &self.window,
            &surface,
            self.window.inner_size(),
            &mut framework,
        );

        let action_map = ActionMap::default();
        let renderer = Renderer::new(&mut framework);

        let instance = T::boot(AppBoot {
            framework: &mut framework,
            window: &self.window,
            surface: &surface,
            surface_configuration: &surface_configuration,
        });

        let state = Box::leak(Box::new(AppState {
            _ph_data: PhantomData,
            instance,
            window: self.window,
            framework,
            renderer,
            input_state,
            surface,
            action_map,
            surface_configuration,
        }));

        self.event_loop.run(move |event, _, control_flow| {
            use winit::event::Event;
            state.input_state.update(&event);
            let actions = state.action_map.update(&state.input_state);
            state.instance.on_winit_event(&event);
            state.instance.dispatch_actions(
                actions,
                AppContext {
                    renderer: &mut state.renderer,
                    framework: &mut state.framework,
                    input_state: &mut state.input_state,
                },
            );

            application_functions::update_application(state);
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested if state.instance.should_shutdown() => {
                        *control_flow = winit::event_loop::ControlFlow::ExitWithCode(0);
                    }
                    WindowEvent::Resized(new_size) => {
                        application_functions::on_resized(state, new_size);
                    }
                    _ => {}
                },

                Event::UserEvent(_) => {}
                Event::MainEventsCleared => {}
                Event::RedrawEventsCleared => {
                    if state.instance.should_render() {
                        state.window.request_redraw();
                    }
                }
                Event::RedrawRequested(_) => {
                    application_functions::render_application(state);
                }
                _ => {}
            };

            *control_flow = winit::event_loop::ControlFlow::Wait
        });
    }
}

mod application_functions {
    use crate::AppResized;

    use super::*;
    pub(super) fn create_surface(
        window: &Window,
        surface: &Surface,
        surface_size: PhysicalSize<u32>,
        framework: &mut Framework,
    ) -> SurfaceConfiguration {
        let surface_configuration = SurfaceConfiguration {
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&framework.adapter)[0],
            width: surface_size.width,
            height: surface_size.height,
            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        };
        surface.configure(&framework.device, &surface_configuration);
        surface_configuration
    }
    pub(super) fn update_application<E, A: Clone, T: AppLoop<E, A>>(state: &mut AppState<E, A, T>) {
        state.instance.update(AppContext {
            renderer: &mut state.renderer,
            framework: &mut state.framework,
            input_state: &mut state.input_state,
        });
        state.framework.update_asset_maps();
    }
    pub(super) fn render_application<E, A: Clone, T: AppLoop<E, A>>(state: &mut AppState<E, A, T>) {
        if let Ok(next_texture) = state.surface.get_current_texture() {
            let texture_view = next_texture
                .texture
                .create_view(&TextureViewDescriptor::default());
            state.instance.render(
                AppContext {
                    renderer: &mut state.renderer,
                    framework: &mut state.framework,
                    input_state: &mut state.input_state,
                },
                texture_view,
            );
            next_texture.present();
        }
    }
    pub(super) fn on_resized<E, A: Clone, T: AppLoop<E, A>>(
        state: &mut AppState<E, A, T>,
        new_size: PhysicalSize<u32>,
    ) {
        if new_size.height == 0 || new_size.width == 0 {
            return;
        }
        let surface_configuration = application_functions::create_surface(
            &state.window,
            &state.surface,
            new_size,
            &mut state.framework,
        );
        state.surface_configuration = surface_configuration;
        state.instance.on_resized(AppResized {
            framework: &mut state.framework,
            window: &state.window,
            surface: &state.surface,
            surface_configuration: &state.surface_configuration,
            new_size,
        });
    }
}
