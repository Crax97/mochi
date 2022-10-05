use std::{cell::RefCell, rc::Rc};

use crate::tools::{EditorContext, PointerRelease};

use super::{brush_engine::stamping_engine::StrokingEngine, tool::Tool};

pub struct ColorPicker<'b> {
    is_active: bool,
    stamping_engine: Rc<RefCell<StrokingEngine<'b>>>,
}

impl<'b> ColorPicker<'b> {
    pub fn new(stamping_engine: Rc<RefCell<StrokingEngine<'b>>>) -> Self {
        Self {
            stamping_engine,
            is_active: false,
        }
    }
}

impl<'b> Tool for ColorPicker<'b> {
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
        let position_into_canvas = context
            .image_editor
            .camera()
            .ndc_into_world(pointer_motion.new_pointer_location_normalized);
        let position_into_canvas = position_into_canvas.cast::<i32>().unwrap();
        let half_document_size = (context.image_editor.document().document_size() / 2)
            .cast::<i32>()
            .unwrap();
        let pixel_position = (position_into_canvas + half_document_size).cast::<u32>();
        if let Some(valid_position) = pixel_position {
            if valid_position.x >= context.image_editor.document().document_size().x
                || valid_position.y >= context.image_editor.document().document_size().y
            {
                return;
            }

            //TODO, FIXME: Final layer should not be flipped.
            let (x, y) = (
                valid_position.x,
                context.image_editor.document().document_size().y - valid_position.y,
            );

            let final_texture_id = context.image_editor.document().final_layer();
            let final_texture = context.image_editor.framework().texture2d(final_texture_id);
            let pixel = final_texture.sample_pixel(x, y, context.image_editor.framework());
            let mut engine = self.stamping_engine.borrow_mut();
            let mut settings = engine.settings();
            settings.color_srgb = [
                (pixel.r * 255.0) as u8,
                (pixel.g * 255.0) as u8,
                (pixel.b * 255.0) as u8,
            ];
            settings.opacity = (pixel.a * 255.0) as u8;
            engine.set_new_settings(settings);
        }
    }

    fn on_pointer_release(&mut self, _pointer_release: PointerRelease, _context: EditorContext) {
        self.is_active = false;
    }
    fn name(&self) -> &'static str {
        "Color picker"
    }
}
