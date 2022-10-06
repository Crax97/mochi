use crate::tools::{EditorContext, PointerRelease};
use cgmath::{num_traits::clamp, InnerSpace};

use super::tool::Tool;

pub struct TransformLayerTool {
    is_active: bool,
}

impl TransformLayerTool {
    pub fn new() -> Self {
        Self { is_active: false }
    }
}

impl Tool for TransformLayerTool {
    fn on_pointer_click(&mut self, _: super::tool::PointerClick, _: EditorContext) {
        self.is_active = true;
    }

    fn on_pointer_move(
        &mut self,
        pointer_motion: super::tool::PointerMove,
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
        let delta = pointer_motion.delta * mult;
        if delta.magnitude2() > 0.5 {
            context.image_editor.mutate_document(|doc| {
                doc.mutate_layer(&doc.current_layer_index(), |layer| {
                    layer.position += delta;
                })
            });
        }
    }

    fn on_pointer_release(&mut self, _pointer_release: PointerRelease, _context: EditorContext) {
        self.is_active = false;
    }
    fn name(&self) -> &'static str {
        "Move tool"
    }
}
