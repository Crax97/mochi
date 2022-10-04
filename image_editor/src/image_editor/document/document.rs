use super::RenderingStrategy;
use crate::layers::{Layer, LayerIndex, RootLayer};
use crate::{
    layers::{LayerCreationInfo, LayerTree},
    LayerConstructionInfo,
};
use cgmath::{num_traits::Num, point2, vec2, Vector2};
use framework::Box2d;
use framework::{texture2d::Texture2dConfiguration, Framework, Texture2d};
use image::{ImageBuffer, Rgba};
use std::{collections::HashMap, iter::FromIterator};
use uuid::Uuid;
use wgpu::{CommandEncoder, RenderPassColorAttachment, RenderPassDescriptor};

pub struct Document<T: RenderingStrategy> {
    uuid: Uuid,

    document_size: Vector2<u32>,
    layers: HashMap<LayerIndex, Layer>,
    tree_root: RootLayer,
    current_layer_index: LayerIndex,
    rendering_strategy: T,
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

impl<T: RenderingStrategy> std::hash::Hash for Document<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl<T: RenderingStrategy> Document<T> {
    pub fn new(
        config: DocumentCreationInfo,
        framework: &Framework,
        mut rendering_strategy: T,
    ) -> Self {
        let bg_layer = Layer::new_bitmap(LayerCreationInfo {
            name: "Background layer".to_owned(),
            width: config.width,
            height: config.height,
            initial_color: Rgba([255, 255, 255, 255]),
            position: point2(0.0, 0.0),
            scale: vec2(1.0, 1.0),
            rotation_radians: 0.0,
        });
        let first_layer = Layer::new_bitmap(LayerCreationInfo {
            name: "Layer 0".to_owned(),
            width: config.width,
            height: config.height,
            initial_color: Rgba([0, 0, 0, 0]),
            position: point2(0.0, 0.0),
            scale: vec2(1.0, 1.0),
            rotation_radians: 0.0,
        });

        let background_layer_index = LayerIndex(0);
        let first_layer_index = LayerIndex(1);

        rendering_strategy.on_new_layer(&bg_layer);
        rendering_strategy.on_new_layer(&first_layer);

        Self {
            uuid: Uuid::new_v4(),
            document_size: vec2(config.width, config.height),
            current_layer_index: first_layer_index,
            layers: HashMap::from_iter([
                (background_layer_index, bg_layer),
                (first_layer_index, first_layer),
            ]),
            tree_root: RootLayer(vec![
                LayerTree::SingleLayer(background_layer_index),
                LayerTree::SingleLayer(first_layer_index),
            ]),
            rendering_strategy,
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn current_layer(&self) -> &Layer {
        self.get_layer(&self.current_layer_index)
    }
    pub fn current_layer_mut(&mut self) -> &mut Layer {
        self.layers
            .get_mut(&self.current_layer_index)
            .expect("Invalid layer index passed to document!")
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

    pub fn mutate_current_layer<F: FnMut(&mut Layer) -> Box2d<u32>>(&mut self, mut fun: F) {
        let modified_rect = {
            let layer = self.current_layer_mut();
            fun(layer)
        };
        if modified_rect.area() == 0 {
            return;
        }
        {
            let layer = self
                .layers
                .get(&self.current_layer_index)
                .expect("This should not happen");
            self.rendering_strategy
                .on_layer_updated(layer, modified_rect);
        }
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

    pub fn texture_bind_group(&self) -> &wgpu::BindGroup {
        &self.rendering_strategy.get_result().bind_group()
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
        self.rendering_strategy.on_new_layer(&new_layer);
        self.layers.insert(layer_index.clone(), new_layer);
        self.tree_root.0.push(LayerTree::SingleLayer(layer_index));
    }

    pub(crate) fn render(&mut self, mut encoder: CommandEncoder) {
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

    pub(crate) fn update_gpu_data(&self, framework: &Framework) {}

    pub fn document_size(&self) -> Vector2<u32> {
        self.document_size
    }

    pub fn current_layer_index(&self) -> LayerIndex {
        self.current_layer_index
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
