use std::collections::HashMap;

use super::layers::{BitmapLayer, Layer, LayerIndex, RootLayer};

pub(crate) struct Document<'framework> {
    pub layers: HashMap<LayerIndex, Layer<'framework>>,
    pub tree_root: RootLayer,
    pub final_layer: BitmapLayer,
}
