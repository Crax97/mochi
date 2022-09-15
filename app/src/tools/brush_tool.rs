use std::{cell::RefCell, rc::Rc};

use cgmath::{point2, MetricSpace, Point2};
use image_editor::ImageEditor;
use wgpu::CommandEncoderDescriptor;

use crate::{
    tools::{EditorContext, PointerClick, PointerMove},
    StrokeContext, StrokePoint,
};

use super::{BrushEngine, StrokePath, Tool};

pub struct BrushTool<'framework> {
    engine: Rc<RefCell<dyn BrushEngine + 'framework>>,
    is_active: bool,
    last_mouse_position: Point2<f32>,
    last_pressure: f32,
    pub min_size: f32,
    pub max_size: f32,
    pub step: f32,
}

impl<'framework> BrushTool<'framework> {
    pub fn new(initial_engine: Rc<RefCell<dyn BrushEngine + 'framework>>, step: f32) -> Self {
        Self {
            engine: initial_engine.clone(),
            step,
            is_active: false,
            last_mouse_position: point2(0.0, 0.0),
            last_pressure: 0.0,
            min_size: 5.0,
            max_size: 8.0,
        }
    }

    fn reposition_point_for_draw(image_editor: &ImageEditor, point: Point2<f32>) -> Point2<f32> {
        image_editor.camera().ndc_into_world(point)
    }
}

impl<'framework> Tool for BrushTool<'framework> {
    fn on_pointer_click(&mut self, pointer_click: PointerClick, context: EditorContext) {
        self.is_active = true;
        self.last_mouse_position = BrushTool::reposition_point_for_draw(
            &context.image_editor,
            pointer_click.pointer_location_normalized,
        );
        self.last_pressure = pointer_click.pressure;
    }

    fn on_pointer_move(&mut self, pointer_motion: PointerMove, context: EditorContext) {
        if !self.is_active {
            return;
        }

        let new_pointer_position = BrushTool::reposition_point_for_draw(
            context.image_editor,
            pointer_motion.new_pointer_location_normalized,
        );

        let distance_from_last_point = self.last_mouse_position.distance(new_pointer_position);
        if distance_from_last_point < self.step {
            return;
        }

        let size_delta = self.max_size - self.min_size;
        let start_size = self.min_size + size_delta * self.last_pressure;
        let end_size = self.min_size + size_delta * pointer_motion.pressure;

        let start = StrokePoint {
            position: self.last_mouse_position,
            size: start_size,
        };
        let end = StrokePoint {
            position: new_pointer_position,
            size: end_size,
        };

        let path = StrokePath::linear_start_to_end(start, end, self.step);
        let framework = context.image_editor.framework();
        let mut encoder = framework
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("BrushTool stroke rendering"),
            });

        {
            let context = StrokeContext {
                layer: context.image_editor.selected_layer(),
                editor: &context.image_editor,
                command_encoder: &mut encoder,
                assets: &context.image_editor.assets(),
                debug: context.debug.clone(),
            };

            self.engine.borrow_mut().stroke(path, context);
            framework.queue.submit(std::iter::once(encoder.finish()));
            self.last_mouse_position = new_pointer_position;
            self.last_pressure = pointer_motion.pressure;
        }

        context.image_editor.mutate_document().mark_dirty();
    }

    fn on_pointer_release(
        &mut self,
        _pointer_release: crate::PointerRelease,
        _context: EditorContext,
    ) {
        self.is_active = false;
    }
}