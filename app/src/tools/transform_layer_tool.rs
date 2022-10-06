use crate::tools::EditorContext;
use cgmath::{num_traits::clamp, point2, InnerSpace, Point2};

use super::{tool::Tool, PointerEvent};

pub struct TransformLayerTool {
    is_active: bool,
    last_frame_position: Point2<f32>,
}

impl TransformLayerTool {
    pub fn new() -> Self {
        Self {
            is_active: false,
            last_frame_position: point2(0.0, 0.0),
        }
    }
}

impl Tool for TransformLayerTool {
    fn on_pointer_click(&mut self, event: PointerEvent, _: EditorContext) {
        self.is_active = true;
        self.last_frame_position = event.new_pointer_location;
    }

    fn on_pointer_move(
        &mut self,
        pointer_motion: super::tool::PointerEvent,
        context: EditorContext,
    ) {
        if !self.is_active {
            return;
        }
        let mult = clamp(
            1.0 / context.image_editor.camera().current_scale() * 0.5,
            0.1,
            0.2,
        );
        let new_position = pointer_motion.new_pointer_location;
        let delta = new_position - self.last_frame_position;
        let delta = delta * mult;
        if delta.magnitude2() > 0.5 {
            context.image_editor.mutate_document(|doc| {
                doc.mutate_layer(&doc.current_layer_index(), |layer| {
                    layer.position += delta;
                })
            });
        }
        self.last_frame_position = new_position;
    }

    fn on_pointer_release(&mut self, _pointer_release: PointerEvent, _context: EditorContext) {
        self.is_active = false;
    }
    fn name(&self) -> &'static str {
        "Move tool"
    }
}
