use super::layers::{Layer, LayerIndex, RootLayer};
use crate::{
    blend_settings::{BlendSettings, BlendSettingsUniform},
    layers::{BitmapLayer, BitmapLayerConfiguration, LayerCreationInfo, LayerTree},
    LayerConstructionInfo,
};
use cgmath::{point2, vec2, Vector2};
use framework::{framework::BufferId, Framework};
use framework::{
    framework::TextureId, renderer::renderer::Renderer, scene::Camera2d, BufferConfiguration,
};
use image::{DynamicImage, ImageBuffer};

use framework::framework::ShaderId;
use std::collections::HashMap;
use uuid::Uuid;

enum BufferingStep {
    First,
    Second,
}

struct LayerDrawInfo {
    bitmap_canvas: BitmapLayer,
    layer_settings_buffer: BufferId,
}

pub struct Document<'framework> {
    framework: &'framework Framework,
    layers_created: u16,

    document_size: Vector2<u32>,
    layers: HashMap<LayerIndex, Layer>,
    layer_canvases: HashMap<Uuid, LayerDrawInfo>,
    tree_root: RootLayer,
    final_layer_1: BitmapLayer,
    final_layer_2: BitmapLayer,
    #[allow(dead_code)]
    buffer_layer: BitmapLayer, // Imma keep it here just in case, too many times i removed it just to need it later again
    buffering_step: BufferingStep,

    current_layer_index: LayerIndex,
}

pub struct DocumentCreationInfo {
    pub width: u32,
    pub height: u32,
    pub first_layer_color: [f32; 4],
}

impl<'l> Document<'l> {
    pub fn new(config: DocumentCreationInfo, framework: &'l Framework) -> Self {
        let final_layer_1 = BitmapLayer::new(
            framework,
            BitmapLayerConfiguration {
                label: "Double Buffering Layer 1".to_owned(),
                width: config.width,
                height: config.height,
                initial_background_color: [0.5, 0.5, 0.5, 1.0],
            },
        );
        let final_layer_2 = BitmapLayer::new(
            framework,
            BitmapLayerConfiguration {
                label: "Double Buffering Layer 2".to_owned(),
                width: config.width,
                height: config.height,
                initial_background_color: [0.5, 0.5, 0.5, 1.0],
            },
        );
        let buffer_layer = BitmapLayer::new(
            framework,
            BitmapLayerConfiguration {
                label: "Draw Buffer Layer".to_owned(),
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

            final_layer_1,
            final_layer_2,
            buffer_layer,

            layers: HashMap::new(),
            layer_canvases: HashMap::new(),
            tree_root: RootLayer(vec![]),
            buffering_step: BufferingStep::First,
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
        self.final_layer().size()
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
        let removed_layer = self.layers.remove(&layer_idx).unwrap();
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
        self.layer_canvases.remove(removed_layer.uuid()).unwrap();
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
                name: config.name.clone(),
                position: point2(0.0, 0.0),
                scale: vec2(1.0, 1.0),
                rotation_radians: 0.0,
            },
        );
        let bitmap_canvas = BitmapLayer::new(
            self.framework,
            BitmapLayerConfiguration {
                label: config.name,
                width: self.document_size.x,
                height: self.document_size.y,
                initial_background_color: [0.0; 4],
            },
        );
        let settings = BlendSettingsUniform::from(BlendSettings {
            blend_mode: new_layer.settings().blend_mode,
        });
        let layer_settings_buffer = self.framework.allocate_typed_buffer(BufferConfiguration::<
            BlendSettingsUniform,
        > {
            initial_setup: framework::buffer::BufferInitialSetup::Data(&vec![settings]),
            buffer_type: framework::BufferType::Uniform,
            allow_write: true,
            allow_read: false,
        });

        let layer_draw_info = LayerDrawInfo {
            bitmap_canvas,
            layer_settings_buffer,
        };

        self.layer_canvases
            .insert(new_layer.uuid().clone(), layer_draw_info);
        self.layers.insert(layer_index.clone(), new_layer);
        self.tree_root.0.push(LayerTree::SingleLayer(layer_index));
    }

    pub(crate) fn update_layers(&mut self, renderer: &mut Renderer) {
        for (_, layer) in self.layers.iter_mut() {
            let layer_info = self.layer_canvases.get(layer.uuid()).unwrap();
            if layer.needs_settings_update() {
                Self::update_layer_settings(
                    self.framework,
                    layer,
                    &layer_info.layer_settings_buffer,
                );
            }

            if layer.needs_bitmap_update() {
                Self::update_layer_bitmap(renderer, layer, &layer_info.bitmap_canvas);
            }
        }
    }

    pub(crate) fn render<'tex>(&mut self, renderer: &mut Renderer, shader_to_use: ShaderId)
    where
        'l: 'tex,
    {
        let draw_sequence = self.generate_draw_sequence();

        self.execute_draw_sequence_double_buffered(renderer, draw_sequence, shader_to_use)
    }

    fn execute_draw_sequence_double_buffered(
        &mut self,
        renderer: &mut Renderer,
        draw_sequence: Vec<LayerIndex>,
        shader_to_use: ShaderId,
    ) {
        // Clear first layer
        let final_layer = self.final_layer().texture().clone();
        Self::clear_texture(renderer, &final_layer, wgpu::Color::TRANSPARENT);

        // Actually draw shit
        let mut draw_layer = |index| {
            let final_layer = self.advance_final_layer().texture().clone();
            let previous_layer = self.previous_buffer_layer().texture().clone();

            // 1. Draw current layer onto buffer layer
            let layer = self.get_layer(&index);
            let layer_draw_info = self.layer_canvases.get(layer.uuid()).unwrap();
            // 2. Blend buffer layer with final layer

            layer_draw_info.bitmap_canvas.draw_blended(
                renderer,
                shader_to_use.clone(),
                previous_layer.clone(),
                layer_draw_info.layer_settings_buffer.clone(),
                &final_layer,
            );
            Self::clear_texture(renderer, &previous_layer, wgpu::Color::TRANSPARENT);
        };
        for layer_index in draw_sequence {
            draw_layer(layer_index);
        }
    }

    pub fn clear_texture(renderer: &mut Renderer, texture: &TextureId, color: wgpu::Color) {
        renderer.begin(&Camera2d::default(), Some(color));
        renderer.end_on_texture(texture);
    }

    fn generate_draw_sequence(&self) -> Vec<LayerIndex> {
        let mut draw_sequence = Vec::new();
        for layer_node in self.tree_root.0.iter() {
            match layer_node {
                LayerTree::SingleLayer(index) => {
                    let layer = self.get_layer(&index);
                    if !layer.settings().is_enabled {
                        continue;
                    }
                    draw_sequence.push(index.clone());
                }
                LayerTree::Group(indices) => {
                    for index in indices {
                        let layer = self.get_layer(&index);
                        if !layer.settings().is_enabled {
                            continue;
                        }
                        draw_sequence.push(index.clone());
                    }
                }
            };
        }
        draw_sequence
    }

    pub fn final_layer(&self) -> &BitmapLayer {
        match self.buffering_step {
            BufferingStep::First => &self.final_layer_2,
            BufferingStep::Second => &self.final_layer_1,
        }
    }

    pub fn previous_buffer_layer(&self) -> &BitmapLayer {
        match self.buffering_step {
            BufferingStep::First => &self.final_layer_1,
            BufferingStep::Second => &self.final_layer_2,
        }
    }

    fn advance_final_layer(&mut self) -> &BitmapLayer {
        match self.buffering_step {
            BufferingStep::First => {
                self.buffering_step = BufferingStep::Second;
            }
            BufferingStep::Second => {
                self.buffering_step = BufferingStep::First;
            }
        };
        self.final_layer()
    }

    pub fn document_size(&self) -> Vector2<u32> {
        self.document_size
    }

    pub fn current_layer_index(&self) -> LayerIndex {
        self.current_layer_index
    }

    pub fn final_image_bytes(&self) -> DynamicImage {
        let bytes = self
            .framework
            .texture2d_read_data(self.final_layer().texture());
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

    fn update_layer_settings(framework: &Framework, layer: &mut Layer, target: &BufferId) {
        framework.buffer_write_sync(
            target,
            vec![BlendSettingsUniform::from(BlendSettings {
                blend_mode: layer.settings().blend_mode,
            })],
        )
    }

    fn update_layer_bitmap(renderer: &mut Renderer, layer: &mut Layer, target: &BitmapLayer) {
        Self::clear_texture(renderer, target.texture(), wgpu::Color::TRANSPARENT);
        layer.lay_on_canvas(renderer, &target);
    }
}
