use std::{cell::RefCell, rc::Rc};

use cgmath::{Point2, Vector2};

use framework::Debug;
use image_editor::ImageEditor;

pub struct EditorContext<'editor, 'framework> {
    pub image_editor: &'editor mut ImageEditor<'framework>,
    pub debug: Rc<RefCell<Debug>>,
}

#[derive(Debug, Clone, Copy)]
pub struct PointerClick {
    pub pointer_location_normalized: Point2<f32>,
    pub pressure: f32,
}
#[derive(Debug, Clone, Copy)]
pub struct PointerMove {
    pub new_pointer_location_normalized: Point2<f32>,
    pub new_pointer_location: Point2<f32>,
    pub delta: Vector2<f32>,
    pub delta_normalized: Vector2<f32>,
    pub pressure: f32,
}
#[derive(Debug, Clone, Copy)]
pub struct PointerRelease {}

pub trait Tool {
    fn on_selected(&mut self, _context: EditorContext) {}
    fn on_deselected(&mut self, _context: EditorContext) {}
    fn on_pointer_click(&mut self, _pointer_click: PointerClick, _context: EditorContext) {}
    fn on_pointer_move(&mut self, _pointer_motion: PointerMove, _context: EditorContext) {}
    fn on_pointer_release(&mut self, _pointer_release: PointerRelease, _context: EditorContext) {}
}
