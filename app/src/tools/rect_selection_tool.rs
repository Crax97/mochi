use crate::tools::{EditorContext, PointerEvent};
use cgmath::{EuclideanSpace, Point2};

use framework::{renderer::draw_command::DrawCommand, Box2d};
use image_editor::selection::SelectionShape;

use super::{tool::Tool, EditorCommand};

pub struct RectSelectionTool {
    is_active: bool,
    first_click_position: Point2<f32>,
    last_click_position: Point2<f32>,
}

impl RectSelectionTool {
    pub fn new() -> Self {
        Self {
            is_active: false,
            first_click_position: Point2::origin(),
            last_click_position: Point2::origin(),
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

        self.first_click_position = context
            .image_editor
            .transform_point_into_pixel_position(event.new_pointer_location_normalized)
            .unwrap();
        self.last_click_position = self.first_click_position.clone();
        None
    }

    fn on_pointer_move(
        &mut self,
        pointer_event: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        if !self.is_active {
            return None;
        }
        let new_position = pointer_event.new_pointer_location_normalized;
        let new_position = context
            .image_editor
            .transform_point_into_pixel_position(new_position);
        match new_position {
            Some(new_pos) => {
                self.last_click_position = new_pos;
            }
            _ => {}
        };
        let rect = Box2d::from_points(self.first_click_position, self.last_click_position);
        context.image_editor.mutate_document(|doc| {
            doc.mutate_partial_selection(|selection| {
                selection.set(SelectionShape::Rectangle(rect))
            });
        });
        None
    }

    fn on_pointer_release(
        &mut self,
        _pointer_event: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = false;

        let rect = Box2d::from_points(self.first_click_position, self.last_click_position);

        context.image_editor.mutate_document(|doc| {
            doc.mutate_selection(|selection| selection.extend(SelectionShape::Rectangle(rect)));
        });
        None
    }
    fn name(&self) -> &'static str {
        "Rect Selection tool"
    }
}
