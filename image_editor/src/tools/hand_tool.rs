use crate::tools::{EditorContext, PointerClick, PointerMove, PointerRelease};
use cgmath::{point2, Point2, Vector2};

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
        _context: EditorContext,
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
        let scaled_movement = context
            .image_editor
            .camera()
            .vec_ndc_into_world(pointer_motion.delta_normalized);
        let scaled_movement = Vector2 {
            x: scaled_movement.x,
            y: -scaled_movement.y
        };
        context.image_editor.pan_camera(scaled_movement);
        self.last_frame_location = pointer_motion.new_pointer_location;
    }

    fn on_pointer_release(&mut self, _pointer_release: PointerRelease, _context: EditorContext) {
        self.is_active = false;
    }
}
