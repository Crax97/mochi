use std::ops::DerefMut;
use std::{cell::RefCell, rc::Rc};

use crate::toolbox::{ToolId, Toolbox};
use crate::tools::brush_engine::stamping_engine::StrokingEngine;
use crate::tools::{
    BrushTool, ColorPicker, DebugSelectRegionTool, EditorCommand, EditorContext, HandTool,
    RectSelectionTool, TransformLayerTool,
};
use crate::ui::{self, ToolUiContext, Ui, UiContext};
use application::{
    key::{Key, ModifierSet},
    ActionMap, ActionState, AppContext, AppLoop, InputState, KeyBinding,
};
use application::{AppBoot, AppResized};

use image_editor::ImageEditor;
use log::{info, warn};
use wgpu::TextureView;
use winit::dpi::LogicalSize;

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

pub struct ImageApplication {
    image_editor: ImageEditor,
    toolbox: Toolbox,
    ui: Box<dyn Ui>,
    stamping_engine: Rc<RefCell<StrokingEngine>>,
    brush_tool: Rc<RefCell<BrushTool>>,
    #[allow(dead_code)]
    hand_tool: Rc<RefCell<HandTool>>,
    undo_stack: UndoStack,

    brush_id: ToolId,
    move_tool_id: ToolId,
    #[allow(dead_code)]
    color_picker_id: ToolId,
}

impl AppLoop<(), String> for ImageApplication {
    fn boot(app_boot: AppBoot) -> Self {
        let framework = app_boot.framework;

        framework
            .shader_compiler
            .define("blend_modes", include_str!("blend_modes.wgsl"))
            .unwrap();

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

        let ui = Box::new(ui::create_ui(
            app_boot.surface_configuration,
            app_boot.window,
            framework,
        ));

        Self {
            image_editor,
            toolbox,
            ui,
            stamping_engine,
            brush_tool,
            hand_tool,
            undo_stack: UndoStack::default(),

            brush_id,
            color_picker_id,
            move_tool_id,
        }
    }
    fn setup_action_map(&self, mut action_map: &mut ActionMap<String>) {
        read_action_bindings(&mut action_map);
    }
    fn on_resized(&mut self, resized: AppResized) {
        if resized.new_size.width == 0 || resized.new_size.height == 0 {
            return;
        }
        let half_size = LogicalSize {
            width: resized.new_size.width as f32 * 0.5,
            height: resized.new_size.height as f32 * 0.5,
        };
        let left_right_top_bottom = [
            -half_size.width,
            half_size.width,
            half_size.height,
            -half_size.height,
        ];
        self.image_editor
            .on_resize(left_right_top_bottom, resized.framework);
        self.ui.on_resized(resized);
    }

    fn on_winit_event(&mut self, event: &winit::event::Event<()>) {
        self.ui.on_new_winit_event(&event);
    }

    fn update(&mut self, mut app_context: AppContext) {
        let context = EditorContext {
            framework: app_context.framework,
            image_editor: &mut self.image_editor,
            renderer: &mut app_context.renderer,
        };
        self.toolbox
            .update(&app_context.input_state, &mut self.undo_stack, context);
    }
    fn render(&mut self, mut app_context: AppContext, app_surface: wgpu::TextureView) {
        self.image_editor
            .update_layers(&mut app_context.renderer, app_context.framework);
        self.image_editor
            .render_document(&mut app_context.renderer, app_context.framework);
        self.draw_editor(app_context, app_surface);
    }
    fn dispatch_actions(&mut self, actions: Vec<String>, mut context: AppContext) {
        for action in actions {
            match action.as_str() {
                "save" => {
                    self.image_editor.export_current_image(context.framework);
                }
                "undo" => {
                    self.undo_stack.try_undo(&mut EditorContext {
                        framework: &mut context.framework,
                        image_editor: &mut self.image_editor,
                        renderer: &mut context.renderer,
                    });
                }
                "redo" => {
                    self.undo_stack.try_redo(&mut EditorContext {
                        framework: &mut context.framework,
                        image_editor: &mut self.image_editor,
                        renderer: &mut context.renderer,
                    });
                }
                "pick_brush" => self.toolbox.set_primary_tool(
                    &self.brush_id,
                    EditorContext {
                        framework: &mut context.framework,
                        image_editor: &mut self.image_editor,
                        renderer: &mut context.renderer,
                    },
                ),
                "pick_move" => self.toolbox.set_primary_tool(
                    &self.move_tool_id,
                    EditorContext {
                        framework: &mut context.framework,
                        image_editor: &mut self.image_editor,
                        renderer: &mut context.renderer,
                    },
                ),
                "toggle_eraser" => {
                    self.stamping_engine.borrow_mut().toggle_eraser();
                }
                _ => {
                    warn!("Unrecognised input action! {}", action);
                }
            }
        }
    }
    fn title(&self) -> &str {
        "Mochi Image Editor"
    }
}

impl ImageApplication {
    fn draw_editor(&mut self, mut state: AppContext, out_surface: TextureView) {
        self.ui.begin();
        let ui_ctx = UiContext {
            framework: &mut state.framework,
            image_editor: &mut self.image_editor,
            toolbox: &mut self.toolbox,
            input_state: &state.input_state,
            stamping_engine: self.stamping_engine.clone(),
            brush_tool: self.brush_tool.clone(),
            undo_stack: &mut self.undo_stack,
            renderer: &mut state.renderer,
        };
        let block_editor = self.ui.do_ui(ui_ctx);
        let ui_ctx = ToolUiContext {
            framework: &mut state.framework,
            image_editor: &mut self.image_editor,
            input_state: &state.input_state,
            stamping_engine: self.stamping_engine.clone(),
            brush_tool: self.brush_tool.clone(),
            undo_stack: &mut self.undo_stack,
            renderer: &mut state.renderer,
        };
        let block_editor = self
            .ui
            .do_tool_ui(ui_ctx, self.toolbox.primary_tool().deref_mut())
            || block_editor;
        self.toolbox.set_is_blocked(block_editor);
        state.renderer.begin(
            &self.image_editor.document().render_camera(),
            None,
            state.framework,
        );
        self.toolbox.draw(&mut state.renderer);
        state.renderer.end(
            &self.image_editor.document().render_result(),
            None,
            state.framework,
        );
        self.image_editor
            .render_canvas(&mut state.renderer, &out_surface, state.framework);
        let ui_command = self.ui.present(&out_surface, &state.framework);
        state.framework.queue.submit(std::iter::once(ui_command));
    }
}
