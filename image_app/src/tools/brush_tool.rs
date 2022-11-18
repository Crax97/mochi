use std::{cell::RefCell, rc::Rc};

use cgmath::{point2, MetricSpace, Point2};
use image_editor::ImageEditor;

use crate::{
    tools::{EditorContext, PointerEvent},
    StrokeContext, StrokePoint,
};

use super::{BrushEngine, EditorCommand, StrokePath, Tool};

pub struct BrushTool {
    engine: Rc<RefCell<dyn BrushEngine>>,
    is_active: bool,
    last_mouse_position: Point2<f32>,
    last_pressure: f32,
    pub size: f32,
    pub pressure_delta: f32,
    pub step: f32,
}

impl BrushTool {
    pub fn new(initial_engine: Rc<RefCell<dyn BrushEngine>>, step: f32) -> Self {
        Self {
            engine: initial_engine.clone(),
            step,
            is_active: false,
            last_mouse_position: point2(0.0, 0.0),
            last_pressure: 0.0,
            size: 5.0,
            pressure_delta: 5.0,
        }
    }

    fn reposition_point_for_draw(
        image_editor: &ImageEditor,
        point: Point2<f32>,
    ) -> Option<Point2<f32>> {
        image_editor.transform_point_into_pixel_position(point)
    }
}

impl Tool for BrushTool {
    fn on_pointer_click(
        &mut self,
        pointer_click: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = true;
        let pt = BrushTool::reposition_point_for_draw(
            &context.image_editor,
            pointer_click.new_pointer_location_normalized,
        );
        if let Some(pos) = pt {
            self.last_mouse_position = pos;
            self.last_pressure = pointer_click.pressure;
            self.engine.borrow_mut().begin_stroking(context)
        } else {
            None
        }
    }

    fn on_pointer_move(
        &mut self,
        pointer_motion: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        if !self.is_active {
            return None;
        }

        let new_pointer_position = BrushTool::reposition_point_for_draw(
            context.image_editor,
            pointer_motion.new_pointer_location_normalized,
        );
        if let Some(new_pointer_position) = new_pointer_position {
            let current_layer = context
                .image_editor
                .document()
                .current_layer_index()
                .clone();
            context.image_editor.mutate_document(|doc| {
                doc.mutate_layer(&current_layer, |layer_m| layer_m.mark_dirty())
            });
            let distance_from_last_point = self.last_mouse_position.distance(new_pointer_position);
            if distance_from_last_point < self.step {
                return None;
            }

            let start_size = self.size + self.pressure_delta * self.last_pressure;
            let end_size = self.size + self.pressure_delta * pointer_motion.pressure;

            let start = StrokePoint {
                position: self.last_mouse_position,
                size: start_size,
            };
            let end = StrokePoint {
                position: new_pointer_position,
                size: end_size,
            };

            let path = StrokePath::linear_start_to_end(start, end, self.step);

            let context = StrokeContext {
                framework: context.framework,
                layer: context.image_editor.selected_layer(),
                editor: &context.image_editor,
                renderer: context.renderer,
            };

            self.last_mouse_position = new_pointer_position;
            self.last_pressure = pointer_motion.pressure;

            self.engine.borrow_mut().stroke(path, context)
        } else {
            None
        }
    }

    fn on_pointer_release(
        &mut self,
        _pointer_release: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = false;
        self.engine.borrow_mut().end_stroking(context)
    }
    fn name(&self) -> &'static str {
        "Brush tool"
    }
}
