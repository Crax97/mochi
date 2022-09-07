pub mod document;
pub mod image_editor;
pub mod image_editor_event;
pub mod layers;
pub mod tools;

pub use image_editor::ImageEditor;
pub use image_editor_event::ImageEditorEvent;
pub use tools::*;

pub struct DummyTool;

impl Tool for DummyTool {}
