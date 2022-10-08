use std::{cell::RefCell, rc::Rc};

use framework::Framework;
use image_editor::ImageEditor;
use renderer::render_pass::texture2d_draw_pass::Texture2dDrawPass;
use wgpu::{CommandBuffer, SurfaceConfiguration, TextureView};
use winit::window::Window;

use crate::{
    app_state::UndoStack,
    input_state::InputState,
    toolbox::Toolbox,
    tools::{brush_engine::stamping_engine::StrokingEngine, BrushTool},
};

mod egui_ui;

pub struct UiContext<'app, 'framework> {
    pub image_editor: &'app mut ImageEditor<'framework>,
    pub toolbox: &'app mut Toolbox<'framework>,
    pub input_state: &'app InputState,
    pub draw_pass: &'app mut Texture2dDrawPass<'framework>,
    pub undo_stack: &'app mut UndoStack,

    pub stamping_engine: Rc<RefCell<StrokingEngine<'framework>>>,
    pub brush_tool: Rc<RefCell<BrushTool<'framework>>>,
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
