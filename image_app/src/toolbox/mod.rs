use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use crate::tools::{EditorContext, PointerEvent, Tool};
use crate::{image_editor_app_loop::UndoStack, stamping_engine::Stamp};
use application::InputState;
use cgmath::point2;
use framework::{renderer::renderer::Renderer, Framework};
use image_editor::layers::{BitmapLayer, BitmapLayerConfiguration};
use winit::event::MouseButton;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct ToolId(usize);

pub struct Toolbox {
    tools: HashMap<ToolId, Rc<RefCell<dyn Tool>>>,
    primary_tool_id: ToolId,
    primary_tool: Rc<RefCell<dyn Tool>>,
    blocked: bool,
}

impl Toolbox {
    pub fn new(primary_tool: Rc<RefCell<dyn Tool>>) -> (Self, ToolId) {
        let mut new_toolbox = Self {
            tools: HashMap::new(),
            primary_tool: primary_tool.clone(),
            blocked: false,
            primary_tool_id: ToolId(0),
        };
        let primary_id = new_toolbox.add_tool(primary_tool);
        new_toolbox.primary_tool_id = primary_id.clone();
        (new_toolbox, primary_id)
    }

    pub fn create_test_stamp(framework: &mut Framework) -> Stamp {
        let test_stamp_bytes = include_bytes!("test/test_brush.png");
        let image = image::load_from_memory(test_stamp_bytes).unwrap();
        let brush_bitmap = BitmapLayer::new_from_bytes(
            "Test brush",
            image.as_bytes(),
            BitmapLayerConfiguration {
                width: image.width(),
                height: image.height(),
            },
            framework,
        );
        Stamp::new(brush_bitmap)
    }

    pub fn add_tool(&mut self, new_tool: Rc<RefCell<dyn Tool>>) -> ToolId {
        let id = self.tools.len();
        let id = ToolId(id);
        self.tools.insert(id, new_tool);
        id
    }

    // Panics if id is not a valid index
    #[allow(dead_code)]
    pub fn get_tool(&self, id: &ToolId) -> RefMut<dyn Tool> {
        self.tools.get(id).expect("Not a valid id!").borrow_mut()
    }

    pub fn primary_tool(&self) -> RefMut<dyn Tool> {
        self.primary_tool.borrow_mut()
    }
    pub fn primary_tool_id(&self) -> &ToolId {
        &self.primary_tool_id
    }

    pub fn for_each_tool<F: FnMut(&ToolId, Ref<dyn Tool>)>(&self, mut f: F) {
        for (id, tool) in self.tools.iter() {
            f(id, tool.borrow());
        }
    }

    pub fn set_is_blocked(&mut self, blocked: bool) {
        self.blocked = blocked;
    }

    pub fn update(
        &mut self,
        input_state: &InputState,
        undo_stack: &mut UndoStack,
        mut context: EditorContext,
    ) {
        if self.blocked {
            return;
        }
        let event = PointerEvent {
            new_pointer_location_normalized: input_state.normalized_mouse_position(),
            new_pointer_location: input_state.mouse_position(),
            pressure: input_state.current_pointer_pressure(),
            window_width: input_state.window_size(),
        };
        let cmd = if input_state.is_mouse_button_just_pressed(MouseButton::Left) {
            self.primary_tool().on_pointer_click(event, &mut context)
        } else if input_state.is_mouse_button_just_released(MouseButton::Left) {
            self.primary_tool().on_pointer_release(event, &mut context)
        } else {
            self.primary_tool().on_pointer_move(event, &mut context)
        };
        if let Some(cmd) = cmd {
            undo_stack.push(cmd);
        }
        if input_state.is_mouse_button_just_pressed(MouseButton::Middle) {
            context
                .image_editor
                .camera_mut()
                .set_position(point2(0.0, 0.0));
        }
        if input_state.mouse_wheel_delta().abs() > 0.0 {
            context
                .image_editor
                .scale_view(input_state.mouse_wheel_delta());
        }
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        self.primary_tool().draw(renderer);
    }

    pub(crate) fn set_primary_tool(&mut self, new_tool_id: &ToolId, mut context: EditorContext) {
        self.primary_tool.borrow_mut().on_deselected(&mut context);
        self.primary_tool_id = new_tool_id.clone();
        self.primary_tool = self
            .tools
            .get(new_tool_id)
            .expect("Non existent tool")
            .clone();
        self.primary_tool.borrow_mut().on_selected(&mut context);
    }
}
