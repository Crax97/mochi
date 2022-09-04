mod layer;

use std::{collections::HashMap, ops::Deref, rc::Rc};

use crate::{
    framework::{self, Framework, Mesh, Vertices},
    image_editor::layer::BitmapLayerConfiguration,
};
use cgmath::{prelude::*, *};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    ColorTargetState, CommandBuffer, CommandEncoderDescriptor, FragmentState, MultisampleState,
    PipelineLayout, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, TextureDescriptor, VertexBufferLayout, VertexState,
};

use self::layer::*;

pub struct Assets {
    pub quad_mesh: Mesh,
    pub simple_diffuse_pipeline: RenderPipeline,
    pub final_present_pipeline: RenderPipeline,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub(crate) struct LayerIndex(u16);
pub(crate) enum LayerTree {
    SingleLayer(LayerIndex),
    Group(Vec<LayerIndex>),
}
pub(crate) struct RootLayer(Vec<LayerTree>);

pub(crate) struct Document {
    layers: HashMap<LayerIndex, LayerType>,
    tree_root: RootLayer,
    final_layer: BitmapLayer,
}

pub struct ImageEditor {
    framework: Rc<Framework>,
    assets: Rc<Assets>,

    document: Document,
}

impl ImageEditor {
    pub fn new(framework: Rc<Framework>, assets: Rc<Assets>) -> Self {
        let final_layer = BitmapLayer::new(
            &framework,
            BitmapLayerConfiguration {
                label: "Final Rendering Layer".to_owned(),
                width: 800,
                height: 600,
                initial_background_color: [0.5, 0.5, 0.5, 1.0],
            },
        );
        let test_layer = BitmapLayer::new(
            &framework,
            BitmapLayerConfiguration {
                label: "Layer 0".to_owned(),
                width: 800,
                height: 600,
                initial_background_color: [1.0, 1.0, 1.0, 1.0],
            },
        );

        let test_document = Document {
            layers: HashMap::from_iter(std::iter::once((
                LayerIndex(123),
                LayerType::Bitmap(test_layer),
            ))),
            tree_root: RootLayer(vec![LayerTree::SingleLayer(LayerIndex(123))]),
            final_layer,
        };

        ImageEditor {
            framework,
            assets,
            document: test_document,
        }
    }

    pub fn redraw_full_image(&mut self) -> CommandBuffer {
        let command_encoder_description = CommandEncoderDescriptor {
            label: Some("Image render encoder"),
        };
        let render_pass_description = RenderPassDescriptor {
            label: Some("ImageEditor Redraw Image Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.document.final_layer.texture_view(),
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
        let mut command_encoder = self
            .framework
            .device
            .create_command_encoder(&command_encoder_description);

        {
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_description);
            render_pass.set_pipeline(&self.assets.simple_diffuse_pipeline);
            let mut draw_context = LayerDrawContext {
                assets: &self.assets,
                render_pass,
            };

            for layer_node in self.document.tree_root.0.iter() {
                match layer_node {
                    LayerTree::SingleLayer(index) => {
                        let layer = self.document.layers.get(index).expect("Nonexistent layer");
                        layer.draw(&mut draw_context);
                    }
                    LayerTree::Group(indices) => {
                        for index in indices {
                            let layer = self.document.layers.get(index).expect("Nonexistent layer");
                            layer.draw(&mut draw_context);
                        }
                    }
                };
            }
        }
        command_encoder.finish()
    }

    pub(crate) fn get_full_image_texture(&self) -> &BitmapLayer {
        &self.document.final_layer
    }
}
