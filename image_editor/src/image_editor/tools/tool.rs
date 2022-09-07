use cgmath::{Point2, Vector2};

use crate::ImageEditor;

pub struct EditorContext<'editor, 'framework> {
    pub image_editor: &'editor mut ImageEditor<'framework>,
}

#[derive(Debug, Clone, Copy)]
pub struct PointerClick {
    pub pointer_location: Point2<f32>,
}
#[derive(Debug, Clone, Copy)]
pub struct PointerMove {
    pub new_pointer_location: Point2<f32>,
    pub delta_normalized: Vector2<f32>,
}
#[derive(Debug, Clone, Copy)]
pub struct PointerRelease {}

pub trait Tool {
    fn on_selected(&mut self, context: EditorContext) {}
    fn on_deselected(&mut self, context: EditorContext) {}
    fn on_pointer_click(&mut self, pointer_click: PointerClick, context: EditorContext) {}
    fn on_pointer_move(&mut self, pointer_motion: PointerMove, context: EditorContext) {}
    fn on_pointer_release(&mut self, pointer_release: PointerRelease, context: EditorContext) {}
}
