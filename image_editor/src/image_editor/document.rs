use std::collections::HashMap;

use cgmath::Vector2;

use super::layers::{BitmapLayer, Layer, LayerIndex, RootLayer};

pub(crate) struct Document<'framework> {
    pub layers: HashMap<LayerIndex, Layer<'framework>>,
    pub tree_root: RootLayer,
    pub final_layer: BitmapLayer,
}

impl Document<'_> {
    pub fn outer_size(&self) -> Vector2<f32> {
        self.final_layer.size()
    }
}
