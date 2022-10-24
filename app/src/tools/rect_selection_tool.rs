use crate::tools::{EditorContext, PointerEvent};
use cgmath::{EuclideanSpace, Point2};

use super::{tool::Tool, EditorCommand};

pub struct RectSelectionTool {
    is_active: bool,
    min_point: Option<Point2<f32>>,
    max_point: Option<Point2<f32>>,
}

impl RectSelectionTool {
    pub fn new() -> Self {
        Self {
            is_active: false,
            min_point: None,
            max_point: None,
        }
    }
}

impl Tool for RectSelectionTool {
    fn on_pointer_click(
        &mut self,
        event: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = true;
        self.min_point = context
            .image_editor
            .transform_point_into_pixel_position(event.new_pointer_location_normalized);
        self.max_point = self.min_point.clone();
        None
    }

    fn on_pointer_release(
        &mut self,
        pointer_event: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = false;

        let new_position = pointer_event.new_pointer_location_normalized;
        let new_position = context
            .image_editor
            .transform_point_into_pixel_position(new_position);
        match new_position {
            Some(new_pos) => {
                self.min_point = self.min_point.map(|mut pt| {
                    if new_pos.x < pt.x {
                        pt.x = new_pos.x;
                    }
                    if new_pos.y < pt.y {
                        pt.y = new_pos.y
                    }

                    pt
                });
                self.max_point = self.max_point.map(|mut pt| {
                    if new_pos.x > pt.x {
                        pt.x = new_pos.x;
                    }
                    if new_pos.y > pt.y {
                        pt.y = new_pos.y
                    }

                    pt
                });
            }
            _ => {}
        }
        match (self.min_point, self.max_point) {
            (Some(min), Some(max)) => {
                let center = (max + min.to_vec()) * 0.5;
                let extents = (max - min) * 0.5;

                println!("Center {:?},  extents {:?}", center, extents);
            }
            _ => {}
        }
        None
    }
    fn name(&self) -> &'static str {
        "Rect Selection tool"
    }
}
