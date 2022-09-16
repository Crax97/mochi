use super::layers::{Layer, LayerIndex, RootLayer};
use crate::{
    layers::{LayerCreationInfo, LayerDrawContext, LayerTree},
    LayerConstructionInfo,
};
use cgmath::{num_traits::Num, point2, vec2, Vector2};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::{collections::HashMap, iter::FromIterator};

pub struct Document {
    document_size: Vector2<u32>,
    canvas_size: Vector2<u32>,
    layers: HashMap<LayerIndex, Layer>,
    tree_root: RootLayer,
    final_render_result: ImageBuffer<Rgba<u8>, Vec<u8>>,

    current_layer_index: LayerIndex,
}

pub struct DocumentCreationInfo {
    pub width: u32,
    pub height: u32,
    pub first_layer_color: [f32; 4],
}

pub fn ceil_to<N: Num + Copy + PartialOrd + From<u32>>(n: N, align_to: N) -> N
where
    u32: From<N>,
{
    let d = {
        let m = n % align_to;
        if m > N::from(0) {
            (u32::from(n) / u32::from(align_to)) + 1
        } else {
            return n;
        }
    };

    align_to * N::from(d)
}

impl Document {
    pub fn new(config: DocumentCreationInfo) -> Self {
        let row_aligned_width = ceil_to(config.width, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let first_layer = Layer::new_bitmap(LayerCreationInfo {
            name: "Layer 0".to_owned(),
            width: config.width,
            height: config.height,
            initial_color: Rgba([0, 0, 0, 255]),
            position: point2(0.0, 0.0),
            scale: vec2(1.0, 1.0),
            rotation_radians: 0.0,
        });

        let first_layer_index = LayerIndex(0);
        let final_render_result = ImageBuffer::new(config.width, config.height);

        Self {
            document_size: vec2(config.width, config.height),
            canvas_size: vec2(row_aligned_width, config.height),
            current_layer_index: first_layer_index,
            final_render_result,
            layers: HashMap::from_iter(std::iter::once((first_layer_index, first_layer))),
            tree_root: RootLayer(vec![LayerTree::SingleLayer(first_layer_index)]),
        }
    }

    pub fn current_layer(&self) -> &Layer {
        self.get_layer(&self.current_layer_index)
    }

    pub fn select_layer(&mut self, new_current_layer: LayerIndex) {
        assert!(self.layers.contains_key(&new_current_layer));
        self.current_layer_index = new_current_layer;
    }

    pub fn get_layer(&self, layer_index: &LayerIndex) -> &Layer {
        self.layers
            .get(&layer_index)
            .expect("Invalid layer index passed to document!")
    }

    pub fn get_layer_mut(&mut self, layer_index: &LayerIndex) -> &mut Layer {
        self.layers
            .get_mut(&layer_index)
            .expect("Invalid layer index passed to document!")
    }
    pub(crate) fn delete_layer(&mut self, layer_idx: LayerIndex) {
        if self.layers.len() == 1 {
            return;
        }
        if self.current_layer_index == layer_idx {
            let new_layer = self
                .layers
                .keys()
                .find(|layer_id| **layer_id != layer_idx)
                .unwrap();
            self.select_layer(new_layer.clone());
        }
        self.layers.remove(&layer_idx);
        let mut erase_which = 0usize;
        for (i, layer) in self.tree_root.0.iter().enumerate() {
            match layer {
                &LayerTree::SingleLayer(idx) if idx == layer_idx => {
                    erase_which = i;
                }
                LayerTree::Group(_) => todo!(),
                _ => {}
            }
        }
        self.tree_root.0.remove(erase_which);
    }

    pub(crate) fn add_layer(
        &mut self,
        layer_name: String,
        layer_index: LayerIndex,
        config: LayerConstructionInfo,
    ) {
        let new_layer = Layer::new_bitmap(LayerCreationInfo {
            name: config.name,
            position: point2(0.0, 0.0),
            scale: vec2(1.0, 1.0),
            rotation_radians: 0.0,
            initial_color: Rgba(config.initial_color),
            width: self.document_size.x,
            height: self.document_size.y,
        });
        self.layers.insert(layer_index.clone(), new_layer);
        self.tree_root.0.push(LayerTree::SingleLayer(layer_index));
    }

    pub(crate) fn update_layers(&mut self) {
        for (_, layer) in self.layers.iter_mut() {
            layer.update();
        }
    }

    pub(crate) fn render(&mut self) {
        for layer_node in self.tree_root.0.iter() {
            match layer_node {
                LayerTree::SingleLayer(index) => {
                    let layer = self.layers.get(&index).expect("Nonexistent layer");
                    layer.draw(&mut LayerDrawContext {});
                }
                LayerTree::Group(indices) => {
                    for index in indices {
                        let layer = self.layers.get(index).expect("Nonexistent layer");
                        layer.draw(&mut LayerDrawContext {});
                    }
                }
            };
        }
    }

    pub fn document_size(&self) -> Vector2<u32> {
        self.document_size
    }

    pub fn current_layer_index(&self) -> LayerIndex {
        self.current_layer_index
    }

    pub(crate) fn canvas_size(&self) -> Vector2<u32> {
        self.canvas_size
    }

    pub fn image_bytes(&mut self) -> &ImageBuffer<Rgba<u8>, Vec<u8>> {
        &self.final_render_result
    }

    pub fn for_each_layer<F: FnMut(&Layer, &LayerIndex)>(&self, mut f: F) {
        for tree in self.tree_root.0.iter() {
            match tree {
                LayerTree::SingleLayer(idx) => {
                    let layer = self.get_layer(idx);
                    f(layer, idx);
                }
                LayerTree::Group(group_members) => {
                    for idx in group_members {
                        let layer = self.get_layer(idx);
                        f(layer, idx);
                    }
                }
            }
        }
    }
}
