use framework::{renderer::renderer::Renderer, Framework};
use wgpu::{Surface, SurfaceConfiguration, TextureView};
use winit::{dpi::PhysicalSize, event::Event, window::Window};

use crate::{ActionMap, InputState};

pub struct AppContext<'a> {
    pub renderer: &'a mut Renderer,
    pub framework: &'a mut Framework,
    pub input_state: &'a InputState,
}

pub struct AppBoot<'a> {
    pub framework: &'a mut Framework,
    pub window: &'a Window,
    pub surface: &'a Surface,
    pub surface_configuration: &'a SurfaceConfiguration,
}
pub struct AppResized<'a> {
    pub framework: &'a mut Framework,
    pub window: &'a Window,
    pub surface: &'a Surface,
    pub surface_configuration: &'a SurfaceConfiguration,
    pub new_size: PhysicalSize<u32>,
}

pub trait AppLoop<T, A: Clone> {
    fn boot(boot_info: AppBoot) -> Self;
    fn update(&mut self, _app_context: AppContext) {}
    fn shutdown(&mut self) {}
    fn should_shutdown(&self) -> bool {
        false
    }
    fn can_shutdown(&self) -> bool {
        true
    }
    fn on_winit_event(&mut self, _event: &Event<T>) {}
    fn on_resized(&mut self, _app_resized: AppResized) {}
    fn render(&mut self, _app_context: AppContext, _app_surface: TextureView) {}
    fn should_render(&self) -> bool {
        true
    }

    fn setup_action_map(&self, _action_map: &mut ActionMap<A>) {}

    fn dispatch_actions(&mut self, _actions: Vec<A>, _app_context: AppContext) {}
    fn title(&self) -> &str;
}
