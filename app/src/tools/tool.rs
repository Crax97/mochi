use cgmath::{Point2, Vector2};

use image_editor::ImageEditor;
use renderer::render_pass::texture2d_draw_pass::Texture2dDrawPass;

pub struct EditorContext<'editor, 'framework> {
    pub image_editor: &'editor mut ImageEditor<'framework>,
}

#[derive(Debug, Clone, Copy)]
pub struct PointerEvent {
    pub new_pointer_location_normalized: Point2<f32>,
    pub new_pointer_location: Point2<f32>,
    pub pressure: f32,
    pub window_width: Vector2<u32>,
}

pub trait Tool {
    fn name(&self) -> &'static str;
    fn on_selected(&mut self, _context: &mut EditorContext) {}
    fn on_deselected(&mut self, _context: &mut EditorContext) {}
    fn on_pointer_click(&mut self, _pointer_click: PointerEvent, _context: &mut EditorContext) {}
    fn on_pointer_move(&mut self, _pointer_motion: PointerEvent, _context: &mut EditorContext) {}
    fn on_pointer_release(&mut self, _pointer_release: PointerEvent, _context: &mut EditorContext) {
    }
}
