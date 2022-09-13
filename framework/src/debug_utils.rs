use cgmath::{vec4, Point2, Vector2, Vector4};
use wgpu::{
    CommandBuffer, CommandEncoderDescriptor, RenderPassColorAttachment, RenderPassDescriptor,
    TextureView, VertexAttribute, VertexBufferLayout,
};

use crate::{
    mesh_names, pipeline_names, typed_buffer::BufferInitialSetup, AssetsLibrary, Framework,
    TypedBuffer, TypedBufferConfiguration,
};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DebugInstance2D {
    position_and_scale: Vector4<f32>,
    color: [f32; 4],
}

impl DebugInstance2D {
    pub fn new(position: Point2<f32>, scale: Vector2<f32>, color: [f32; 4]) -> Self {
        Self {
            position_and_scale: vec4(position.x, position.y, scale.x, scale.y),
            color: color,
        }
    }
}

impl<'a> DebugInstance2D {
    pub fn layout() -> VertexBufferLayout<'a> {
        const LAYOUT: &'static [VertexAttribute] =
            &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<DebugInstance2D>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: LAYOUT,
        }
    }
}

unsafe impl bytemuck::Pod for DebugInstance2D {}
unsafe impl bytemuck::Zeroable for DebugInstance2D {}

pub struct Debug {
    debug_items: Vec<DebugInstance2D>,
}

impl Debug {
    pub fn new() -> Self {
        Self {
            debug_items: vec![],
        }
    }

    pub fn begin_debug(&mut self) {
        self.debug_items.clear();
    }

    pub fn draw_debug_point(
        &mut self,
        position: Point2<f32>,
        scale: Vector2<f32>,
        color: [f32; 4],
    ) {
        self.debug_items
            .push(DebugInstance2D::new(position, scale, color));
    }

    pub fn end_debug(
        &mut self,
        final_texture: &TextureView,
        asset_library: &AssetsLibrary,
        camera: &TypedBuffer,
        framework: &'_ Framework,
    ) -> CommandBuffer {
        let bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Debug bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let bind_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Debug bind group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(camera.binding_resource()),
                }],
            });
        let command_encoder_description = CommandEncoderDescriptor {
            label: Some("Debug draw render encoder"),
        };
        let render_pass_description = RenderPassDescriptor {
            label: Some("ImageEditor Debug Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &final_texture,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };
        let mut debug_points_buffer = TypedBuffer::new(
            framework,
            TypedBufferConfiguration::<DebugInstance2D> {
                initial_setup: BufferInitialSetup::Size(0),
                buffer_type: crate::BufferType::Vertex,
                allow_write: true,
                allow_read: false,
            },
        );
        debug_points_buffer.write_sync(&self.debug_items);
        let mut command_encoder = framework
            .device
            .create_command_encoder(&command_encoder_description);

        {
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_description);

            let pipeline = asset_library.pipeline(pipeline_names::SIMPLE_COLORED);
            pipeline.execute_with_renderpass(
                render_pass,
                &[(0, &bind_group), (1, &debug_points_buffer)],
            );
        }
        command_encoder.finish()
    }
}
