use super::layers::{Layer, LayerIndex, RootLayer};
use crate::{
    layers::{
        BitmapLayer, BitmapLayerConfiguration, LayerCreationInfo, LayerDrawContext, LayerTree,
        ShaderLayerSettings,
    },
    LayerConstructionInfo,
};
use cgmath::{point2, vec2, Vector2};
use framework::{Framework, Texture2d, TypedBufferConfiguration};
use image::{DynamicImage, ImageBuffer};
use std::{collections::HashMap, iter::FromIterator};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, CommandEncoder, RenderPassColorAttachment,
    RenderPassDescriptor,
};

pub struct Document<'framework> {
    framework: &'framework Framework,
    document_size: Vector2<u32>,
    layers: HashMap<LayerIndex, Layer<'framework>>,
    tree_root: RootLayer,
    final_layer: BitmapLayer<'framework>,

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
            &framework,
            BitmapLayerConfiguration {
                label: "Final Rendering Layer".to_owned(),
                width: config.width,
                height: config.height,
                initial_background_color: [0.5, 0.5, 0.5, 1.0],
            },
        );
        let background_layer = BitmapLayer::new(
            &framework,
            BitmapLayerConfiguration {
                label: "Background Layer".to_owned(),
                width: config.width,
                height: config.height,
                initial_background_color: [1.0, 1.0, 1.0, 1.0],
            },
        );
        let background_layer = Layer::new_bitmap(
            background_layer,
            LayerCreationInfo {
                name: "Background Layer".to_owned(),
                position: point2(0.0, 0.0),
                scale: vec2(1.0, 1.0),
                rotation_radians: 0.0,
            },
            &framework,
        );
        let background_layer_index = LayerIndex(0);
        let first_layer = BitmapLayer::new(
            &framework,
            BitmapLayerConfiguration {
                label: "Layer 0".to_owned(),
                width: config.width,
                height: config.height,
                initial_background_color: [0.0, 0.0, 0.0, 0.0],
            },
        );
        let first_layer = Layer::new_bitmap(
            first_layer,
            LayerCreationInfo {
                name: "Layer 0".to_owned(),
                position: point2(0.0, 0.0),
                scale: vec2(1.0, 1.0),
                rotation_radians: 0.0,
            },
            &framework,
        );

        let first_layer_index = LayerIndex(1);
        let settings_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Layer Draw Settings bind layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        Self {
            framework,
            document_size: vec2(config.width, config.height),
            current_layer_index: first_layer_index,
            final_layer,
            layers: HashMap::from_iter([
                (background_layer_index, background_layer),
                (first_layer_index, first_layer),
            ]),
            tree_root: RootLayer(vec![
                LayerTree::SingleLayer(background_layer_index),
                LayerTree::SingleLayer(first_layer_index),
            ]),
        }
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

    pub(crate) fn final_layer_texture_view(&self) -> &wgpu::TextureView {
        self.final_layer.texture().texture_view()
    }

    pub(crate) fn final_texture(&self) -> &Texture2d {
        self.final_layer.texture()
    }

    pub(crate) fn add_layer(
        &mut self,
        framework: &'l Framework,
        layer_name: String,
        layer_index: LayerIndex,
        config: LayerConstructionInfo,
    ) {
        let new_layer = BitmapLayer::new(
            framework,
            BitmapLayerConfiguration {
                label: layer_name.clone(),
                width: self.document_size.x,
                height: self.document_size.y,
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

    pub(crate) fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        layer_draw_pass: &crate::layers::Texture2dDrawPass,
    ) {
        {
            {
                let render_pass_description = RenderPassDescriptor {
                    label: Some("ImageEditor Clear Final Image"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: self.final_layer_texture_view(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                };
                let _ = encoder.begin_render_pass(&render_pass_description);
            }
        }
        let render_pass_description = RenderPassDescriptor {
            label: Some("ImageEditor Redraw Image Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.final_layer_texture_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };
        for layer_node in self.tree_root.0.iter() {
            match layer_node {
                LayerTree::SingleLayer(index) => {
                    let layer = self.layers.get(&index).expect("Nonexistent layer");
                    let mut render_pass = encoder.begin_render_pass(&render_pass_description);
                    layer_draw_pass.prepare(&mut render_pass);
                    layer.draw(LayerDrawContext {
                        render_pass,
                        draw_pass: &layer_draw_pass,
                    });
                }
                LayerTree::Group(indices) => {
                    for index in indices {
                        let layer = self.layers.get(index).expect("Nonexistent layer");
                        let render_pass = encoder.begin_render_pass(&render_pass_description);
                        layer.draw(LayerDrawContext {
                            render_pass,
                            draw_pass: &layer_draw_pass,
                        });
                    }
                }
            };
        }
    }

    pub fn final_layer(&self) -> &Texture2d {
        &self.final_layer.texture()
    }

    pub fn document_size(&self) -> Vector2<u32> {
        self.document_size
    }

    pub fn current_layer_index(&self) -> LayerIndex {
        self.current_layer_index
    }

    pub fn final_image_bytes(&self) -> DynamicImage {
        let bytes = self.final_texture().read_data(&self.framework);
        DynamicImage::ImageRgba8(
            ImageBuffer::from_vec(self.document_size.x, self.document_size.y, bytes).unwrap(),
        )
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
