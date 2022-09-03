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
    pub final_present_pipeline: RenderPipeline,
}

pub struct ImageEditor {
    framework: Rc<Framework>,
    assets: Rc<Assets>,

    // TODO: Put into document struct
    layers: Vec<Layer>,
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
                initial_background_color: [0.5, 0.5, 0.5, 1.0],
            },
        );
        let test_layer = Layer::new(
            &framework,
            LayerConfiguration {
                label: "Layer 0".to_owned(),
                width: 800,
                height: 600,
                initial_background_color: [1.0, 1.0, 1.0, 1.0],
            },
        );

        ImageEditor {
            framework,
            assets,
            layers: vec![test_layer],
            final_layer,
        }
    }

    pub fn redraw_full_image(&mut self) -> CommandBuffer {
        let command_encoder_description = CommandEncoderDescriptor {
            label: Some("Image render encoder"),
        };
        let render_pass_description = RenderPassDescriptor {
            label: Some("ImageEditor Redraw Image Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.final_layer.texture_view(),
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

            for layer in self.layers.iter() {
                render_pass.set_bind_group(0, layer.binding_group(), &[]);
                self.assets.quad_mesh.draw(&mut render_pass, 1);
            }
        }
        command_encoder.finish()
    }

    pub(crate) fn get_full_image_texture(&self) -> &Layer {
        &self.final_layer
    }
}
