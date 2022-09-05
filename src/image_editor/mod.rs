mod layer;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    framework::{Framework, InstanceBuffer, InstanceBufferConfiguration, Mesh, MeshInstance2D},
    image_editor::layer::BitmapLayerConfiguration,
};
use cgmath::{point2, vec2};
use wgpu::{
    ColorTargetState, CommandBuffer, CommandEncoderDescriptor, FragmentState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, VertexState,
};

use self::layer::*;

pub struct Assets {
    pub quad_mesh: Mesh,
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
    layers: HashMap<LayerIndex, Layer>,
    tree_root: RootLayer,
    final_layer: BitmapLayer,
}

pub struct ImageEditor {
    framework: Rc<Framework>,
    assets: Rc<Assets>,

    document: Document,
    simple_diffuse_pipeline: RenderPipeline,
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

        let instance_buffer = InstanceBuffer::new(
            &framework,
            InstanceBufferConfiguration {
                initial_data: Vec::<MeshInstance2D>::new(),
                allow_write: false,
            },
        );
        let test_document = Document {
            layers: HashMap::from_iter(std::iter::once((
                LayerIndex(123),
                Layer {
                    layer_type: LayerType::Bitmap(test_layer),
                    position: point2(0.5, 0.0),
                    scale: vec2(1.0, 1.0),
                    rotation_radians: 0.0,
                    instance_buffer,
                },
            ))),
            tree_root: RootLayer(vec![LayerTree::SingleLayer(LayerIndex(123))]),
            final_layer,
        };

        let module = framework
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/simple_shader.wgsl"));

        let bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Simple shader layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let render_pipeline_layout =
            framework
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Simple Render Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let simple_diffuse_pipeline =
            framework
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Simple render pipeline"),
                    layout: Some(&render_pipeline_layout),
                    depth_stencil: None,
                    vertex: VertexState {
                        module: &module,
                        entry_point: "vs",
                        buffers: &[Mesh::layout(), MeshInstance2D::layout()],
                    },
                    fragment: Some(FragmentState {
                        module: &module,
                        entry_point: "fs",
                        targets: &[Some(ColorTargetState {
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Cw,
                        conservative: false,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                    },
                });

        ImageEditor {
            framework,
            assets,
            document: test_document,
            simple_diffuse_pipeline,
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
            render_pass.set_pipeline(&self.simple_diffuse_pipeline);

            for (_, layer) in self.document.layers.iter_mut() {
                layer.update(&self.framework);
            }

            let mut draw_context = LayerDrawContext {
                render_pass: &mut render_pass,
                assets: &self.assets,
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
