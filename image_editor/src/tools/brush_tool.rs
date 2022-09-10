use std::{cell::RefCell, rc::Rc};

use cgmath::{point2, vec2, MetricSpace, Point2};
use wgpu::CommandEncoderDescriptor;

use crate::{
    tools::{EditorContext, PointerClick, PointerMove},
    ImageEditor, StrokeContext,
};

use super::{BrushEngine, StrokePath, Tool};

pub struct BrushTool {
    engine: Option<Rc<RefCell<dyn BrushEngine>>>,
    is_active: bool,
    last_mouse_position: Point2<f32>,
    step: f32,
}

impl BrushTool {
    pub fn new(step: f32) -> Self {
        Self {
            engine: None,
            step,
            is_active: false,
            last_mouse_position: point2(0.0, 0.0),
        }
    }
}

impl BrushTool {
    fn reposition_point_for_draw(image_editor: &ImageEditor, point: Point2<f32>) -> Point2<f32> {
        image_editor.camera().ndc_into_world(point)
    }
}

impl Tool for BrushTool {
    fn on_pointer_click(&mut self, pointer_click: PointerClick, context: EditorContext) {
        if !self.engine.is_some() {
            return;
        }
        self.is_active = true;
        self.last_mouse_position = BrushTool::reposition_point_for_draw(
            &context.image_editor,
            pointer_click.pointer_location,
        );
    }

    fn on_pointer_move(&mut self, pointer_motion: PointerMove, context: EditorContext) {
        if !self.is_active {
            return;
        }
        if !self.engine.is_some() {
            return;
        }

        let new_pointer_position = BrushTool::reposition_point_for_draw(
            context.image_editor,
            pointer_motion.new_pointer_location,
        );

        let distance_from_prev_point = new_pointer_position.distance(self.last_mouse_position);
        if distance_from_prev_point < self.step {
            return;
        }

        context.debug.borrow_mut().draw_debug_point(
            pointer_motion.new_pointer_location,
            vec2(3.0, 3.0),
            [1.0, 0.0, 0.0, 1.0],
        );
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
            editor: &context.image_editor,
            command_encoder: &mut encoder,
            assets: context.image_editor.assets(),
            debug: context.debug.clone(),
        };
        self.engine
            .as_deref()
            .unwrap()
            .borrow_mut()
            .stroke(path, context);
        framework.queue.submit(std::iter::once(encoder.finish()));
        self.last_mouse_position = new_pointer_position;
    }

    fn on_pointer_release(
        &mut self,
        _pointer_release: crate::PointerRelease,
        _context: EditorContext,
    ) {
        if !self.engine.is_some() {
            return;
        }
        self.is_active = false
    }
}
