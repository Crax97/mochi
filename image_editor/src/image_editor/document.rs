use std::collections::HashMap;

use cgmath::Vector2;

use super::layers::{BitmapLayer, Layer, LayerIndex, RootLayer};

pub(crate) struct Document<'framework> {
    pub layers: HashMap<LayerIndex, Layer<'framework>>,
    pub tree_root: RootLayer,
    pub final_layer: BitmapLayer,

    pub current_layer_index: LayerIndex,
}

impl Document<'_> {
    pub fn outer_size(&self) -> Vector2<f32> {
        self.final_layer.size()
    }

    pub(crate) fn current_layer(&self) -> &Layer {
        self.layers
            .get(&self.current_layer_index)
            .expect("Invalid layer index passed to document!")
    }
}
