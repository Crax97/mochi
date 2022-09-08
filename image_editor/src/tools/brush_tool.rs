use cgmath::{point2, Point2};
use wgpu::CommandEncoderDescriptor;

use crate::{
    tools::{EditorContext, PointerClick, PointerMove},
    StrokeContext,
};

use super::{BrushEngine, StrokePath, Tool};

pub struct BrushTool {
    engine: Box<dyn BrushEngine>,
    is_active: bool,
    last_mouse_position: Point2<f32>,
    step: f32,
}

impl BrushTool {
    pub fn new(initial_engine: Box<dyn BrushEngine>, step: f32) -> Self {
        Self {
            engine: initial_engine,
            step,
            is_active: false,
            last_mouse_position: point2(0.0, 0.0),
        }
    }
}

impl Tool for BrushTool {
    fn on_pointer_click(&mut self, pointer_click: PointerClick, context: EditorContext) {
        self.is_active = true;
        self.last_mouse_position = context
            .image_editor
            .ndc_position_into_world(pointer_click.pointer_location);
    }

    fn on_pointer_move(&mut self, pointer_motion: PointerMove, context: EditorContext) {
        if !self.is_active {
            return;
        }
        let new_pointer_position = context
            .image_editor
            .ndc_position_into_world(pointer_motion.new_pointer_location);
        let path = StrokePath::linear_start_to_end(
            self.last_mouse_position,
            new_pointer_position,
            self.step,
        );
        let framework = context.image_editor.framework();
        let mut encoder = framework
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("BrushTool stroke rendering"),
            });

        let context = StrokeContext {
            layer: context.image_editor.selected_layer(),
            command_encoder: &mut encoder,
            assets: context.image_editor.assets(),
        };
        self.engine.stroke(path, context);
        framework.queue.submit(std::iter::once(encoder.finish()));
    }

    fn on_pointer_release(
        &mut self,
        _pointer_release: crate::PointerRelease,
        _context: EditorContext,
    ) {
        self.is_active = false
    }

    fn on_selected(&mut self, context: EditorContext) {}

    fn on_deselected(&mut self, context: EditorContext) {}
}
