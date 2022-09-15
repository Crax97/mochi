use std::{cell::RefCell, rc::Rc};

use crate::tools::{EditorContext, PointerRelease};

use cgmath::num_traits::Pow;
use image::GenericImageView;

use super::{brush_engine::stamping_engine::StrokingEngine, tool::Tool, BrushTool};

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
            let pixel = context
                .image_editor
                .get_full_image_bytes()
                .get_pixel(valid_position.x, valid_position.y);
            let mut engine = self.stamping_engine.borrow_mut();
            let mut settings = engine.settings();
            settings.color_srgb = [pixel.0[0], pixel.0[1], pixel.0[2]];
            settings.opacity = pixel.0[3];
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
