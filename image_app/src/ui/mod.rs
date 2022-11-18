use std::{cell::RefCell, rc::Rc};

use application::{AppResized, InputState};
use framework::{renderer::renderer::Renderer, Framework};
use image_editor::ImageEditor;
use wgpu::{CommandBuffer, SurfaceConfiguration, TextureView};
use winit::window::Window;

use crate::{
    image_editor_app_loop::UndoStack,
    toolbox::Toolbox,
    tools::{brush_engine::stamping_engine::StrokingEngine, BrushTool, Tool},
};

mod egui_ui;

pub struct UiContext<'app> {
    pub framework: &'app mut Framework,
    pub image_editor: &'app mut ImageEditor,
    pub renderer: &'app mut Renderer,
    pub toolbox: &'app mut Toolbox,
    pub input_state: &'app InputState,
    pub undo_stack: &'app mut UndoStack,

    pub stamping_engine: Rc<RefCell<StrokingEngine>>,
    pub brush_tool: Rc<RefCell<BrushTool>>,
}

pub struct ToolUiContext<'app> {
    pub framework: &'app mut Framework,
    pub image_editor: &'app mut ImageEditor,
    pub renderer: &'app mut Renderer,
    pub input_state: &'app InputState,
    pub undo_stack: &'app mut UndoStack,

    pub stamping_engine: Rc<RefCell<StrokingEngine>>,
    pub brush_tool: Rc<RefCell<BrushTool>>,
}

pub trait Ui {
    fn begin(&mut self);
    fn on_new_winit_event(&mut self, event: &winit::event::Event<()>);

    fn do_ui(&mut self, ctx: UiContext) -> bool;
    fn do_tool_ui(&mut self, app_ctx: ToolUiContext, tool: &mut dyn Tool) -> bool;
    fn on_resized(&mut self, resized: AppResized);
    fn present(&mut self, output_view: &TextureView, framework: &Framework) -> CommandBuffer;
}

pub fn create_ui(
    surface_configuration: &SurfaceConfiguration,
    window: &Window,
    framework: &Framework,
) -> impl Ui {
    egui_ui::EguiUI::new(surface_configuration, window, framework)
}
