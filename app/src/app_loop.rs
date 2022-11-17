use framework::{renderer::renderer::Renderer, Framework};
use winit::dpi::PhysicalSize;

pub struct AppContext<'a> {
    pub renderer: &'a mut Renderer,
    pub framework: &'a mut Framework,
}

pub trait AppLoop {
    fn boot(&mut self, framework: &mut Framework) {}
    fn update(&mut self, app_context: AppContext) {}
    fn shutdown(&mut self) {}
    fn should_shutdown(&self) -> bool {
        false
    }
    fn on_resized(&mut self, new_size: PhysicalSize<u32>, framework: &mut Framework) {}
    fn render(&mut self, app_context: AppContext) {}
    fn should_render(&self) -> bool {
        true
    }
    fn title(&self) -> &str;
}
