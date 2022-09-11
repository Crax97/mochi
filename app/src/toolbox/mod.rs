use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use crate::input_state::InputState;
use cgmath::point2;
use framework::{Debug, Framework, TypedBuffer};
use image_editor::{
    layers::{BitmapLayer, BitmapLayerConfiguration},
    stamping_engine::{
        Stamp, StampConfiguration, StampCreationInfo, StampUniformData, StrokingEngine,
    },
    BrushTool, EditorContext, HandTool, ImageEditor, PointerClick, PointerMove, PointerRelease,
};
use image_editor::{BrushEngine, Tool};
use winit::event::MouseButton;

pub struct Toolbox<'framework> {
    tools: Vec<Box<dyn Tool>>,
    brush_engines: Vec<Rc<RefCell<Box<dyn BrushEngine>>>>,
    framework: &'framework Framework,
    pub brush_tool: BrushTool<'framework>,
    hand_tool: HandTool,
    stamping_engine: Rc<RefCell<StrokingEngine<'framework>>>,
    enabled: bool,
}

impl<'framework> Toolbox<'framework> {
    pub fn new(
        framework: &'framework Framework,
        stamping_engine: Rc<RefCell<StrokingEngine<'framework>>>,
    ) -> Self {
        Self {
            tools: vec![],
            brush_engines: vec![],
            framework,
            brush_tool: BrushTool::new(stamping_engine.clone(), 5.0),
            hand_tool: HandTool::new(),
            stamping_engine,
            enabled: true,
        }
    }

    pub fn stamping_engine(&self) -> Ref<StrokingEngine> {
        self.stamping_engine.borrow()
    }

    pub fn update_stamping_engine_data(&mut self, new_data: StampConfiguration) {
        self.stamping_engine.borrow_mut().set_new_settings(new_data);
    }

    pub fn create_test_stamp(camera_buffer: &TypedBuffer, framework: &Framework) -> Stamp {
        let test_stamp_bytes = include_bytes!("test/test_brush.png");
        let image = image::load_from_memory(test_stamp_bytes).unwrap();
        let brush_bitmap = BitmapLayer::new_from_bytes(
            &framework,
            image.as_bytes(),
            BitmapLayerConfiguration {
                label: "Test brush".to_owned(),
                width: image.width(),
                initial_background_color: [0.0, 0.0, 0.0, 0.0],
                height: image.height(),
            },
        );
        Stamp::new(
            brush_bitmap,
            &framework,
            StampCreationInfo { camera_buffer },
        )
    }
    pub fn update(
        &mut self,
        input_state: &InputState,
        mut image_editor: &mut ImageEditor,
        debug: Rc<RefCell<Debug>>,
    ) {
        if !self.enabled {
            return;
        }
        if input_state.is_mouse_button_just_pressed(MouseButton::Left) {
            self.brush_tool.on_pointer_click(
                PointerClick {
                    pointer_location: input_state.normalized_mouse_position(),
                },
                EditorContext {
                    image_editor: &mut image_editor,
                    debug: debug.clone(),
                },
            );
        } else if input_state.is_mouse_button_just_released(MouseButton::Left) {
            self.brush_tool.on_pointer_release(
                PointerRelease {},
                EditorContext {
                    image_editor: &mut image_editor,
                    debug: debug.clone(),
                },
            );
        } else {
            self.brush_tool.on_pointer_move(
                PointerMove {
                    new_pointer_location: input_state.normalized_mouse_position(),
                    delta_normalized: input_state.normalized_mouse_delta(),
                },
                EditorContext {
                    image_editor: &mut image_editor,
                    debug: debug.clone(),
                },
            );
        }
        if input_state.is_mouse_button_just_pressed(MouseButton::Right) {
            self.hand_tool.on_pointer_click(
                PointerClick {
                    pointer_location: input_state.normalized_mouse_position(),
                },
                EditorContext {
                    image_editor: &mut image_editor,
                    debug: debug.clone(),
                },
            );
        } else if input_state.is_mouse_button_just_released(MouseButton::Right) {
            self.hand_tool.on_pointer_release(
                PointerRelease {},
                EditorContext {
                    image_editor: &mut image_editor,
                    debug: debug.clone(),
                },
            );
        } else {
            self.hand_tool.on_pointer_move(
                PointerMove {
                    new_pointer_location: input_state.normalized_mouse_position(),
                    delta_normalized: input_state.normalized_mouse_delta(),
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

    pub(crate) fn set_enabled(&mut self, toolbox_enabled: bool) {
        self.enabled = toolbox_enabled;
    }
}
