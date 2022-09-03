mod layer;

use std::{ops::Deref, rc::Rc};

use crate::{
    framework::{self, Framework, Mesh, Vertices},
    image_editor::layer::LayerConfiguration,
};
use cgmath::{prelude::*, *};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    ColorTargetState, CommandBuffer, CommandEncoderDescriptor, FragmentState, MultisampleState,
    PipelineLayout, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, TextureDescriptor, VertexBufferLayout, VertexState,
};

use self::layer::Layer;

pub struct Assets {
    pub quad_mesh: Mesh,
    pub simple_diffuse_pipeline: RenderPipeline,
}

pub struct ImageEditor {
    framework: Rc<Framework>,
    assets: Rc<Assets>,
    render_pipeline_test: wgpu::RenderPipeline,

    final_layer: Layer,
}

impl ImageEditor {
    pub fn new(framework: Rc<Framework>, assets: Rc<Assets>) -> Self {
        let final_layer = Layer::new(
            &framework,
            LayerConfiguration {
                label: "Final Rendering Layer".to_owned(),
                width: 800,
                height: 600,
                texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            },
        );
        let module = framework
            .device
            .create_shader_module(wgpu::include_wgsl!("thing_test.wgsl"));

        let render_pipeline_test =
            framework
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("simple shader"),
                    layout: None,
                    depth_stencil: None,
                    vertex: VertexState {
                        module: &module,
                        entry_point: "vs",
                        buffers: &[Mesh::layout()],
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
                        front_face: wgpu::FrontFace::Ccw,
                        conservative: false,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                    },
                });

        ImageEditor {
            framework,
            final_layer,
            render_pipeline_test,
            assets,
        }
    }

    pub fn redraw_full_image(&mut self) -> CommandBuffer {
        let command_encoder_description = CommandEncoderDescriptor {
            label: Some("Image render encoder"),
        };
        let render_pass_description = RenderPassDescriptor {
            label: Some("ImageEditor Final Layer Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.final_layer.texture_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.3,
                        b: 0.3,
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
            self.assets.quad_mesh.bind_to_render_pass(&mut render_pass);
            render_pass.set_pipeline(&self.render_pipeline_test);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }
        command_encoder.finish()
    }

    pub(crate) fn get_full_image_texture(&self) -> &Layer {
        &self.final_layer
    }
}
