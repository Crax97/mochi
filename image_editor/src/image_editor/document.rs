use super::layers::{Layer, LayerIndex, RootLayer};
use crate::{
    layers::{BitmapLayer, BitmapLayerConfiguration, LayerCreationInfo, LayerTree},
    LayerConstructionInfo,
};
use cgmath::{point2, vec2, Vector2};
use framework::Framework;
use image::{DynamicImage, GenericImage, ImageBuffer};
use renderer::render_pass::texture2d_draw_pass::Texture2dDrawPass;
use scene::Camera2d;

use std::collections::HashMap;

pub struct Document<'framework> {
    framework: &'framework Framework,
    layers_created: u16,

    document_size: Vector2<u32>,
    layers: HashMap<LayerIndex, Layer<'framework>>,
    tree_root: RootLayer,
    final_layer: BitmapLayer,

    current_layer_index: LayerIndex,
}

pub struct DocumentCreationInfo {
    pub width: u32,
    pub height: u32,
    pub first_layer_color: [f32; 4],
}

impl<'l> Document<'l> {
    pub fn new(config: DocumentCreationInfo, framework: &'l Framework) -> Self {
        let final_layer = BitmapLayer::new(
            framework,
            BitmapLayerConfiguration {
                label: "Final Rendering Layer".to_owned(),
                width: config.width,
                height: config.height,
                initial_background_color: [0.5, 0.5, 0.5, 1.0],
            },
        );

        let first_layer_index = LayerIndex(1);

        let mut document = Self {
            framework,
            layers_created: 0,
            document_size: vec2(config.width, config.height),
            current_layer_index: first_layer_index,
            final_layer,
            layers: HashMap::new(),
            tree_root: RootLayer(vec![]),
        };

        document.add_layer(
            framework,
            LayerConstructionInfo {
                initial_color: [1.0, 1.0, 1.0, 1.0],
                name: "Background Layer".into(),
                width: document.document_size.x,
                height: document.document_size.y,
            },
        );
        document.add_layer(
            framework,
            LayerConstructionInfo {
                initial_color: [0.0, 0.0, 0.0, 0.0],
                name: "Layer 0".into(),
                width: document.document_size.x,
                height: document.document_size.y,
            },
        );

        document
    }

    pub fn outer_size(&self) -> Vector2<f32> {
        self.final_layer.size()
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

    pub fn mutate_layer<F: FnMut(&mut Layer)>(
        &mut self,
        layer_index: &LayerIndex,
        mut mutate_fn: F,
    ) {
        let layer = self
            .layers
            .get_mut(&layer_index)
            .expect("Invalid layer index passed to document!");

        mutate_fn(layer);
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

    pub(crate) fn add_layer(&mut self, framework: &'l Framework, config: LayerConstructionInfo) {
        let layer_index = LayerIndex(self.layers_created);
        self.layers_created += 1;
        let new_layer = BitmapLayer::new(
            framework,
            BitmapLayerConfiguration {
                label: config.name.clone(),
                width: config.width,
                height: config.height,
                initial_background_color: config.initial_color,
            },
        );
        let new_layer = Layer::new_bitmap(
            new_layer,
            LayerCreationInfo {
                name: config.name,
                position: point2(0.0, 0.0),
                scale: vec2(1.0, 1.0),
                rotation_radians: 0.0,
            },
            framework,
        );
        self.layers.insert(layer_index.clone(), new_layer);
        self.tree_root.0.push(LayerTree::SingleLayer(layer_index));
    }

    pub(crate) fn update_layers(&mut self) {
        for (_, layer) in self.layers.iter_mut() {
            layer.update();
        }
    }

    pub(crate) fn render<'tex>(
        &mut self,
        mut pass: &mut Texture2dDrawPass<'l>, // layer_draw_pass: &crate::layers::Texture2dDrawPass,
    ) where
        'l: 'tex,
    {
        let final_layer = self.final_layer.texture();
        let final_texture = self.framework.texture2d(&final_layer);

        pass.finish(final_texture.texture_view(), true);

        pass.begin(&Camera2d::new(
            -0.1,
            1000.0,
            [
                -(self.document_size().x as f32 * 0.5),
                self.document_size().x as f32 * 0.5,
                self.document_size().y as f32 * 0.5,
                -(self.document_size().y as f32 * 0.5),
            ],
        ));
        for layer_node in self.tree_root.0.iter() {
            match layer_node {
                LayerTree::SingleLayer(index) => {
                    let layer = self.layers.get(&index).expect("Nonexistent layer");
                    layer.draw(self.framework, &mut pass, final_texture.texture_view());
                }
                LayerTree::Group(indices) => {
                    for index in indices {
                        let layer = self.layers.get(index).expect("Nonexistent layer");
                        layer.draw(self.framework, &mut pass, final_texture.texture_view());
                    }
                }
            };
        }
    }

    pub fn final_layer(&self) -> &BitmapLayer {
        &self.final_layer
    }

    pub fn document_size(&self) -> Vector2<u32> {
        self.document_size
    }

    pub fn current_layer_index(&self) -> LayerIndex {
        self.current_layer_index
    }

    pub fn final_image_bytes(&self) -> DynamicImage {
        let final_layer_texture = self.framework.texture2d(self.final_layer.texture());
        let bytes = final_layer_texture.read_data(&self.framework);
        let width = bytes.width;
        let height = bytes.height;
        let data = bytes.to_bytes(true);
        let raw_image = ImageBuffer::from_raw(width, height, data).unwrap();
        DynamicImage::ImageRgba8(raw_image)
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
