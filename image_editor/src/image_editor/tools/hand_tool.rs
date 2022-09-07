use cgmath::{point2, vec3, Point2};

use crate::{EditorContext, ImageEditor};

use super::tool::Tool;

pub struct HandTool {
    last_frame_location: Point2<f32>,
    is_active: bool,
}

impl HandTool {
    pub fn new() -> Self {
        Self {
            last_frame_location: point2(0.0, 0.0),
            is_active: false,
        }
    }
}

impl Tool for HandTool {
    fn on_pointer_click(
        &mut self,
        pointer_click: super::tool::PointerClick,
        context: EditorContext,
    ) {
        self.is_active = true;
        self.last_frame_location = pointer_click.pointer_location;
    }

    fn on_pointer_move(
        &mut self,
        pointer_motion: super::tool::PointerMove,
        context: EditorContext,
    ) {
        if !self.is_active {
            return;
        }
        let delta = pointer_motion.new_pointer_location - self.last_frame_location;
        let camera = context.image_editor.pan_camera(delta);
        self.last_frame_location = pointer_motion.new_pointer_location;
        println!("Mouse move! {:?}", pointer_motion)
    }

    fn on_pointer_release(
        &mut self,
        pointer_release: crate::PointerRelease,
        context: EditorContext,
    ) {
        self.is_active = false;
    }
}
