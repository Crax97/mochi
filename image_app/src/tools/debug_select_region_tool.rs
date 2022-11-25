use super::{tool::Tool, EditorCommand};
use crate::tools::{EditorContext, PointerEvent};
use cgmath::Point2;

pub struct DebugSelectRegionTool {
    is_active: bool,
    begin_position: Option<Point2<f32>>,
}

impl DebugSelectRegionTool {
    pub fn new() -> Self {
        Self {
            is_active: false,
            begin_position: None,
        }
    }
}

impl Tool for DebugSelectRegionTool {
    fn on_pointer_click(
        &mut self,
        event: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = true;
        self.begin_position = context
            .image_editor
            .transform_point_into_pixel_position(event.new_pointer_location_normalized);
        None
    }

    fn on_pointer_release(
        &mut self,
        _pointer_event: PointerEvent,
        _context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        None
    }
    fn name(&self) -> &'static str {
        "Region test tool"
    }
}
