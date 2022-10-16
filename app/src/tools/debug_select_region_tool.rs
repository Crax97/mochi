use crate::tools::{EditorContext, PointerEvent};
use cgmath::{ElementWise, Point2};
use image::{DynamicImage, RgbaImage};

use super::{tool::Tool, EditorCommand};

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
        pointer_event: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = false;

        let new_position = pointer_event.new_pointer_location_normalized;
        let new_position = context
            .image_editor
            .transform_point_into_pixel_position(new_position);
        match (self.begin_position, new_position) {
            (Some(begin), Some(end)) => {
                let layer = context.image_editor.document().current_layer();
                match layer.layer_type {
                    image_editor::layers::LayerType::Bitmap => {
                        let framework = context.image_editor.framework();
                        let (width, height) = framework.texture2d_dimensions(layer.bitmap.texture());
                        let half_dims = Point2 {
                            x: width / 2,
                            y: height / 2,
                        }
                        .cast::<f32>()
                        .unwrap();
                        // COPY THIS WHEN IMPROVING BRUSH UNDO REDO COMMAND
                        let begin = begin.add_element_wise(half_dims);
                        let end = end.add_element_wise(half_dims);

                        let begin_x = begin.x.min(end.x) as u32;
                        let begin_y = begin.y.min(end.y) as u32;
                        let end_x = begin.x.max(end.x) as u32;
                        let end_y = begin.y.max(end.y) as u32;
                        let region_width = end_x - begin_x;
                        let region_height = end_y - begin_y;

                        let new_subregion_texture = framework.texture2d_copy_subregion(
                            layer.bitmap.texture(),
                            begin_x,
                            begin_y,
                            region_width,
                            region_height,
                        );
                        let subregion = framework.texture2d_read_data(&new_subregion_texture);
                        let width = subregion.width;
                        let height = subregion.height;

                        let data = subregion.to_bytes(true);
                        let dyn_image = DynamicImage::ImageRgba8(
                            RgbaImage::from_vec(width, height, data).unwrap(),
                        );
                        dyn_image
                            .save("test_reg.png")
                            .unwrap_or_else(|err| println!("Error happened: {err}"));
                    }
                }
            }
            _ => {}
        }
        None
    }
    fn name(&self) -> &'static str {
        "Region test tool"
    }
}
