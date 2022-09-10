use framework::Framework;
use image_editor::ImageEditor;
use wgpu::{CommandBuffer, SurfaceConfiguration, TextureView};
use winit::window::Window;

use crate::{input_state::InputState, toolbox::Toolbox};

mod egui_ui;

pub struct UiContext<'app, 'framework> {
    pub image_editor: &'app mut ImageEditor<'framework>,
    pub toolbox: &'app mut Toolbox<'framework>,
    pub input_state: &'app InputState,
}

pub trait Ui {
    fn begin(&mut self);
    fn on_new_winit_event(&mut self, event: &winit::event::Event<()>);
    fn do_ui(&mut self, ctx: UiContext) -> bool;
    fn present(
        &mut self,
        framework: &Framework,
        window: &Window,
        surface_configuration: SurfaceConfiguration,
        output_view: &TextureView,
    ) -> CommandBuffer;
}

pub fn create_ui(
    framework: &Framework,
    surface_configuration: &SurfaceConfiguration,
    window: &Window,
) -> impl Ui {
    egui_ui::EguiUI::new(framework, surface_configuration, window)
}
