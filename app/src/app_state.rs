use std::{cell::RefCell, rc::Rc};

use crate::input_state::{ActionMap, InputState};
use crate::toolbox::{ToolId, Toolbox};
use crate::tools::brush_engine::stamping_engine::StrokingEngine;
use crate::tools::{
    BrushTool, ColorPicker, DebugSelectRegionTool, EditorCommand, EditorContext, HandTool,
    RectSelectionTool, TransformLayerTool,
};
use crate::ui::{self, Ui, UiContext};
use crate::{ActionState, Key, KeyBinding, ModifierSet};
use framework::renderer::renderer::Renderer;
use framework::Framework;
use image_editor::ImageEditor;
use log::{info, warn};
use wgpu::{CommandBuffer, Surface, SurfaceConfiguration};
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::window::Window;

#[derive(Default)]
pub struct UndoStack {
    undo: Vec<Box<dyn EditorCommand>>,
    redo: Vec<Box<dyn EditorCommand>>,
}

impl UndoStack {
    pub fn push(&mut self, command: Box<dyn EditorCommand>) {
        self.redo.clear();
        self.undo.push(command);
    }

    pub fn do_undo(&mut self, context: &mut EditorContext) {
        let command = self.undo.pop().expect("Empty undo stack!");
        let redo_command = command.undo(context);
        self.redo.push(redo_command);
    }

    pub fn do_redo(&mut self, context: &mut EditorContext) {
        let command = self.redo.pop().expect("Empty redo stack!");
        let undo_command = command.undo(context);
        self.undo.push(undo_command);
    }

    pub fn has_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn has_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn try_undo(&mut self, context: &mut EditorContext) {
        if self.has_undo() {
            self.do_undo(context)
        }
    }

    fn try_redo(&mut self, context: &mut EditorContext) {
        if self.has_redo() {
            self.do_redo(context)
        }
    }
}

fn read_action_bindings(action_map: &mut ActionMap<String>) {
    // TODO: Action bindings aren't actually read from a file yet.
    // In the future add something like an action_bindings.json file to read stuff
    // + an ui to allow users to change the bindings
    action_map.add_action_binding(
        KeyBinding {
            key: (Key::S, ActionState::Pressed),
            modifiers: ModifierSet::new(false, false, true, false),
        },
        "save",
    );
    action_map.add_action_binding(
        KeyBinding {
            key: (Key::Z, ActionState::Pressed),
            modifiers: ModifierSet::new(false, false, true, false),
        },
        "undo",
    );
    action_map.add_action_binding(
        KeyBinding {
            key: (Key::Z, ActionState::Pressed),
            modifiers: ModifierSet::new(true, false, true, false),
        },
        "redo",
    );
    action_map.add_action_binding((Key::B, ActionState::Pressed), "pick_brush");
    action_map.add_action_binding((Key::M, ActionState::Pressed), "pick_move");
    action_map.add_action_binding((Key::E, ActionState::Pressed), "toggle_eraser");
}

pub struct ImageApplication<T: Ui> {
    pub(crate) window: Window,
    pub(crate) final_surface: Surface,
    pub(crate) final_surface_configuration: SurfaceConfiguration,
    instant_renderer: Renderer,
    // This has nothing to do with deferred rendering: deferred in this context means that end will be called
    // at the end of layer rendering, and that tools must not call end (place a check for this)
    deferred_renderer: Renderer,
    image_editor: ImageEditor,
    input_state: InputState,
    toolbox: Toolbox,
    ui: T,
    stamping_engine: Rc<RefCell<StrokingEngine>>,
    brush_tool: Rc<RefCell<BrushTool>>,
    #[allow(dead_code)]
    hand_tool: Rc<RefCell<HandTool>>,
    undo_stack: UndoStack,
    action_map: ActionMap<String>,

    brush_id: ToolId,
    move_tool_id: ToolId,
    #[allow(dead_code)]
    color_picker_id: ToolId,
}

impl<T: Ui> ImageApplication<T> {
    pub(crate) fn new(
        window: Window,
        framework: &mut Framework,
        ui: T,
        final_surface: Surface,
        final_surface_configuration: SurfaceConfiguration,
    ) -> Self {
        let image_editor = ImageEditor::new(framework, &[1024.0, 1024.0]);

        let test_stamp = Toolbox::create_test_stamp(framework);
        let stamping_engine = StrokingEngine::new(test_stamp, framework);
        let stamping_engine = Rc::new(RefCell::new(stamping_engine));
        let brush_tool = Rc::new(RefCell::new(BrushTool::new(stamping_engine.clone(), 1.0)));
        let hand_tool = Rc::new(RefCell::new(HandTool::new()));
        let color_picker = Rc::new(RefCell::new(ColorPicker::new(stamping_engine.clone())));
        let move_tool = Rc::new(RefCell::new(TransformLayerTool::new()));
        let test_tool = Rc::new(RefCell::new(DebugSelectRegionTool::new()));
        let rect_select_tool = Rc::new(RefCell::new(RectSelectionTool::new()));

        let (mut toolbox, brush_id) = Toolbox::new(brush_tool.clone());
        let _ = toolbox.add_tool(hand_tool.clone());
        let color_picker_id = toolbox.add_tool(color_picker.clone());
        let move_tool_id = toolbox.add_tool(move_tool);
        let _ = toolbox.add_tool(test_tool);
        let _ = toolbox.add_tool(rect_select_tool);

        let mut action_map = ActionMap::default();

        read_action_bindings(&mut action_map);

        let instant_renderer = Renderer::new(framework);
        let deferred_renderer = Renderer::new(framework);

        Self {
            window,
            instant_renderer,
            deferred_renderer,
            final_surface,
            final_surface_configuration,
            image_editor,
            input_state: InputState::default(),
            toolbox,
            ui,
            stamping_engine,
            brush_tool,
            hand_tool,
            undo_stack: UndoStack::default(),
            action_map,

            brush_id,
            color_picker_id,
            move_tool_id,
        }
    }

    pub(crate) fn on_resized(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
        framework: &mut Framework,
    ) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        // self.final_pass
        //     .update_size([new_size.width as f32, new_size.height as f32]);
        let half_size = LogicalSize {
            width: new_size.width as f32 * 0.5,
            height: new_size.height as f32 * 0.5,
        };
        let left_right_top_bottom = [
            -half_size.width,
            half_size.width,
            half_size.height,
            -half_size.height,
        ];
        let new_surface_configuration = SurfaceConfiguration {
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.final_surface.get_supported_formats(&framework.adapter)[0],
            width: new_size.width as u32,
            height: new_size.height as u32,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        };
        self.final_surface
            .configure(&framework.device, &new_surface_configuration);
        self.final_surface_configuration = new_surface_configuration;
        self.image_editor
            .on_resize(left_right_top_bottom, framework);
    }

    pub(crate) fn on_event(
        &mut self,
        event: &winit::event::Event<()>,
        framework: &mut Framework,
    ) -> ControlFlow {
        self.input_state.update(&event);
        let actions = self.action_map.update(&self.input_state);
        self.ui.on_new_winit_event(&event);

        self.dispatch_actions(actions, framework);
        let context = EditorContext {
            framework,
            image_editor: &mut self.image_editor,
            renderer: &mut self.instant_renderer,
        };
        self.toolbox
            .update(&self.input_state, &mut self.undo_stack, context);

        match event {
            winit::event::Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        // if app.handle_on_close_requested() == AppFlow::Exit {
                        // *control_flow = ControlFlow::ExitWithCode(0);
                        // }
                        return ControlFlow::ExitWithCode(0);
                    }
                    WindowEvent::Resized(new_size) => {
                        self.on_resized(*new_size, framework);
                    }
                    _ => {}
                }
            }
            winit::event::Event::UserEvent(_) => {}
            winit::event::Event::RedrawRequested(_) => {
                self.ui.begin();

                let ui_ctx = UiContext {
                    framework,
                    image_editor: &mut self.image_editor,
                    toolbox: &mut self.toolbox,
                    input_state: &self.input_state,
                    stamping_engine: self.stamping_engine.clone(),
                    brush_tool: self.brush_tool.clone(),
                    undo_stack: &mut self.undo_stack,
                    renderer: &mut self.instant_renderer,
                    deferred_renderer: &mut self.deferred_renderer,
                };
                let block_editor = self.ui.do_ui(ui_ctx);
                self.toolbox.set_is_blocked(block_editor);
                self.image_editor
                    .update_layers(&mut self.instant_renderer, framework);

                let current_texture = match self.final_surface.get_current_texture() {
                    Ok(surface) => surface,
                    Err(e) => match e {
                        wgpu::SurfaceError::Outdated => {
                            info!("RedrawRequested: early return because the current_texture is outdated (user resizing window maybe?)");
                            return ControlFlow::Wait;
                        }
                        _ => {
                            panic!("While presenting the last image: {e}");
                        }
                    },
                };

                let mut commands: Vec<CommandBuffer> = vec![];

                let app_surface_view = current_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.image_editor
                    .render_document(&mut self.instant_renderer, framework);

                self.deferred_renderer.begin(
                    &self.image_editor.document().final_layer().camera(),
                    None,
                    framework,
                );
                self.toolbox.draw(&mut self.deferred_renderer);
                self.deferred_renderer.end(
                    &self.image_editor.document().final_layer().texture(),
                    None,
                    framework,
                );

                self.image_editor.render_canvas(
                    &mut self.instant_renderer,
                    &app_surface_view,
                    framework,
                );

                let surface_configuration = self.final_surface_configuration.clone();

                let ui_command = self.ui.present(
                    &self.window,
                    surface_configuration,
                    &app_surface_view,
                    &framework,
                );
                commands.push(ui_command);

                framework.queue.submit(commands);
                current_texture.present();
            }
            _ => {}
        }

        self.window.request_redraw();
        framework.update_asset_maps();
        ControlFlow::Poll
    }

    fn dispatch_actions(&mut self, actions: Vec<String>, framework: &mut Framework) {
        for action in actions {
            match action.as_str() {
                "save" => {
                    self.image_editor.export_current_image(framework);
                }
                "undo" => {
                    self.undo_stack.try_undo(&mut EditorContext {
                        framework,
                        image_editor: &mut self.image_editor,
                        renderer: &mut self.instant_renderer,
                    });
                }
                "redo" => {
                    self.undo_stack.try_redo(&mut EditorContext {
                        framework,
                        image_editor: &mut self.image_editor,
                        renderer: &mut self.instant_renderer,
                    });
                }
                "pick_brush" => self.toolbox.set_primary_tool(&self.brush_id),
                "pick_move" => self.toolbox.set_primary_tool(&self.move_tool_id),
                "toggle_eraser" => {
                    self.stamping_engine.borrow_mut().toggle_eraser();
                }
                _ => {
                    warn!("Unrecognised input action! {}", action);
                }
            }
        }
    }
}
