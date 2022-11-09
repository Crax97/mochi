use std::{cell::RefCell, rc::Rc};

use framework::{renderer::renderer::Renderer, Framework};
use image_editor::ImageEditor;
use wgpu::{CommandBuffer, SurfaceConfiguration, TextureView};
use winit::window::Window;

use crate::{
    app_state::UndoStack,
    input_state::InputState,
    toolbox::Toolbox,
    tools::{brush_engine::stamping_engine::StrokingEngine, BrushTool},
};

mod egui_ui;

pub struct UiContext<'app> {
    pub framework: &'app mut Framework,
    pub image_editor: &'app mut ImageEditor,
    pub renderer: &'app mut Renderer,
    pub deferred_renderer: &'app mut Renderer,
    pub toolbox: &'app mut Toolbox,
    pub input_state: &'app InputState,
    pub undo_stack: &'app mut UndoStack,

    pub stamping_engine: Rc<RefCell<StrokingEngine>>,
    pub brush_tool: Rc<RefCell<BrushTool>>,
}

pub trait Ui {
    fn begin(&mut self);
    fn on_new_winit_event(&mut self, event: &winit::event::Event<()>);
    fn do_ui(&mut self, ctx: UiContext) -> bool;
    fn present(
        &mut self,
        window: &Window,
        surface_configuration: SurfaceConfiguration,
        output_view: &TextureView,
        framework: &Framework,
    ) -> CommandBuffer;
}

pub fn create_ui(
    surface_configuration: &SurfaceConfiguration,
    window: &Window,
    framework: &Framework,
) -> impl Ui {
    egui_ui::EguiUI::new(surface_configuration, window, framework)
}
