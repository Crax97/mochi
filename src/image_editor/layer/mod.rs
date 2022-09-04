mod bitmap_layer;

pub use bitmap_layer::*;

pub enum LayerType {
    Bitmap(bitmap_layer::BitmapLayer),
    Group(Vec<Box<LayerType>>),
}

impl LayerType {
    pub fn update() {}
}
