use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use crate::stamping_engine::Stamp;
use crate::{
    input_state::InputState,
    tools::{EditorContext, PointerClick, PointerMove, PointerRelease, Tool},
};
use cgmath::point2;
use framework::{Debug, Framework};
use image_editor::ImageEditor;
use winit::event::MouseButton;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct ToolId(usize);

pub struct Toolbox<'framework> {
    tools: HashMap<ToolId, Rc<RefCell<dyn Tool + 'framework>>>,
    framework: &'framework Framework,
    primary_tool: Rc<RefCell<dyn Tool + 'framework>>,
    secondary_tool: Rc<RefCell<dyn Tool + 'framework>>,
}

impl<'framework> Toolbox<'framework> {
    pub fn new(
        framework: &'framework Framework,
        primary_tool: Rc<RefCell<dyn Tool + 'framework>>,
        secondary_tool: Rc<RefCell<dyn Tool + 'framework>>,
    ) -> (Self, ToolId, ToolId) {
        let mut new_toolbox = Self {
            tools: HashMap::new(),
            framework,
            primary_tool: primary_tool.clone(),
            secondary_tool: secondary_tool.clone(),
        };
        let primary_id = new_toolbox.add_tool(primary_tool);
        let secondary_id = new_toolbox.add_tool(secondary_tool);
        (new_toolbox, primary_id, secondary_id)
    }

    pub fn create_test_stamp() -> Stamp {
        let test_stamp_bytes = include_bytes!("test/test_brush.png");
        let image = image::load_from_memory(test_stamp_bytes).unwrap();
        Stamp::new(image.into_rgba8())
    }

    pub fn add_tool(&mut self, new_tool: Rc<RefCell<dyn Tool + 'framework>>) -> ToolId {
        let id = self.tools.len();
        let id = ToolId(id);
        self.tools.insert(id, new_tool);
        id
    }

    // Panics if id is not a valid index
    pub fn get_tool(&self, id: &ToolId) -> RefMut<dyn Tool + 'framework> {
        self.tools.get(id).expect("Not a valid id!").borrow_mut()
    }

    pub fn primary_tool(&self) -> RefMut<dyn Tool + 'framework> {
        self.primary_tool.borrow_mut()
    }

    pub fn secondary_tool(&self) -> RefMut<dyn Tool + 'framework> {
        self.secondary_tool.borrow_mut()
    }

    pub fn for_each_tool<F: FnMut(&ToolId, Ref<dyn Tool + 'framework>)>(&self, mut f: F) {
        for (id, tool) in self.tools.iter() {
            f(id, tool.borrow());
        }
    }

    pub fn update(
        &mut self,
        input_state: &InputState,
        mut image_editor: &mut ImageEditor,
        debug: Rc<RefCell<Debug>>,
    ) {
        if input_state.is_mouse_button_just_pressed(MouseButton::Left) {
            self.primary_tool().on_pointer_click(
                PointerClick {
                    pointer_location_normalized: input_state.normalized_mouse_position(),
                    pressure: input_state.current_pointer_pressure(),
                },
                EditorContext {
                    image_editor: &mut image_editor,
                    debug: debug.clone(),
                },
            );
        } else if input_state.is_mouse_button_just_released(MouseButton::Left) {
            self.primary_tool().on_pointer_release(
                PointerRelease {},
                EditorContext {
                    image_editor: &mut image_editor,
                    debug: debug.clone(),
                },
            );
        } else {
            self.primary_tool().on_pointer_move(
                PointerMove {
                    new_pointer_location_normalized: input_state.normalized_mouse_position(),
                    delta_normalized: input_state.normalized_mouse_delta(),
                    pressure: input_state.current_pointer_pressure(),
                    new_pointer_location: input_state.mouse_position(),
                    delta: input_state.mouse_delta(),
                },
                EditorContext {
                    image_editor: &mut image_editor,
                    debug: debug.clone(),
                },
            );
        }
        if input_state.is_mouse_button_just_pressed(MouseButton::Right) {
            self.secondary_tool().on_pointer_click(
                PointerClick {
                    pointer_location_normalized: input_state.normalized_mouse_position(),
                    pressure: input_state.current_pointer_pressure(),
                },
                EditorContext {
                    image_editor: &mut image_editor,
                    debug: debug.clone(),
                },
            );
        } else if input_state.is_mouse_button_just_released(MouseButton::Right) {
            self.secondary_tool().on_pointer_release(
                PointerRelease {},
                EditorContext {
                    image_editor: &mut image_editor,
                    debug: debug.clone(),
                },
            );
        } else {
            self.secondary_tool().on_pointer_move(
                PointerMove {
                    new_pointer_location_normalized: input_state.normalized_mouse_position(),
                    delta_normalized: input_state.normalized_mouse_delta(),
                    pressure: input_state.current_pointer_pressure(),
                    new_pointer_location: input_state.mouse_position(),
                    delta: input_state.mouse_delta(),
                },
                EditorContext {
                    image_editor: &mut image_editor,
                    debug: debug.clone(),
                },
            );
        }
        if input_state.is_mouse_button_just_pressed(MouseButton::Middle) {
            image_editor.camera_mut().set_position(point2(0.0, 0.0));
        }
        if input_state.mouse_wheel_delta().abs() > 0.0 {
            image_editor.scale_view(input_state.mouse_wheel_delta());
        }
    }

    pub(crate) fn set_primary_tool(&mut self, new_tool_id: &ToolId) {
        self.primary_tool = self
            .tools
            .get(new_tool_id)
            .expect("Non existent tool")
            .clone();
    }
}
