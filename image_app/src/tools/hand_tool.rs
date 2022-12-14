use crate::tools::{EditorContext, PointerEvent};
use cgmath::{point2, InnerSpace, Point2};

use super::{tool::Tool, EditorCommand};

pub struct HandTool {
    is_active: bool,
    last_frame_position: Point2<f32>,
}

impl HandTool {
    pub fn new() -> Self {
        Self {
            is_active: false,
            last_frame_position: point2(0.0, 0.0),
        }
    }
}

impl Tool for HandTool {
    fn on_pointer_click(
        &mut self,
        pointer_event: PointerEvent,
        _: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = true;
        self.last_frame_position = pointer_event.new_pointer_location;
        None
    }

    fn on_pointer_move(
        &mut self,
        pointer_motion: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        if !self.is_active {
            return None;
        }
        let new_position = pointer_motion.new_pointer_location;
        let delta = new_position - self.last_frame_position;
        if delta.magnitude2() > 0.5 {
            context.image_editor.pan_camera(delta);
        }
        self.last_frame_position = new_position;
        None
    }

    fn on_pointer_release(
        &mut self,
        _pointer_release: PointerEvent,
        _context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = false;
        None
    }
    fn name(&self) -> &'static str {
        "Hand tool"
    }
}
