use crate::tools::{EditorContext, PointerEvent};
use cgmath::{EuclideanSpace, Point2};
use framework::{
    renderer::draw_command::{DrawCommand, DrawMode, OptionalDrawData, PrimitiveType},
    Box2d,
};

use super::{tool::Tool, EditorCommand};

pub struct RectSelectionTool {
    is_active: bool,
    first_click_position: Option<Point2<f32>>,
    last_click_position: Point2<f32>,
}

impl RectSelectionTool {
    pub fn new() -> Self {
        Self {
            is_active: false,
            first_click_position: None,
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
            .transform_point_into_pixel_position(event.new_pointer_location_normalized);
        if let Some(pos) = self.first_click_position {
            self.last_click_position = pos.clone();
        }
        None
    }

    fn on_pointer_move(
        &mut self,
        pointer_event: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        let new_position = pointer_event.new_pointer_location_normalized;
        let new_position = context
            .image_editor
            .transform_point_into_pixel_position(new_position);
        match new_position {
            Some(new_pos) => {
                self.last_click_position = new_pos;
            }
            _ => {}
        }
        None
    }

    fn on_pointer_release(
        &mut self,
        _pointer_event: PointerEvent,
        _context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = false;
        self.first_click_position = None;

        // 1. Draw the selection rect on the stencil buffer
        // 2. Draw layer using the rect stencil buffer, this is the selection. Store it into a new texture
        // 3. Invert the stencil buffer
        // 4. Draw the layer using the  inverted stencil buffer: this is the remaining part of the texture

        None
    }

    fn draw(&self, renderer: &mut framework::renderer::renderer::Renderer) {
        match self.first_click_position {
            Some(pos) => {
                let rect = Box2d::from_points(pos, self.last_click_position);
                renderer.draw(DrawCommand {
                    primitives: PrimitiveType::Rect {
                        rects: vec![rect],
                        multiply_color: wgpu::Color::RED,
                    },
                    draw_mode: DrawMode::Single,
                    additional_data: OptionalDrawData::default(),
                });
            }
            _ => {}
        }
    }
    fn name(&self) -> &'static str {
        "Rect Selection tool"
    }
}
