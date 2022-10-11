use cgmath::{point2, vec2};
use wgpu::{
    Color, CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, TextureView,
};

use crate::{
    buffer::BufferInitialSetup,
    framework::{BufferId, ShaderId},
    shader::Shader,
    AssetRef, Buffer, BufferConfiguration, BufferType, Camera2d, Camera2dUniformBlock, Framework,
    Mesh, MeshInstance2D, Texture2d,
};

use super::draw_command::{BindableResource, DrawCommand, PrimitiveType};

pub struct Renderer<'f> {
    framework: &'f Framework,

    draw_queue: Vec<DrawCommand>,
    camera_buffer_id: BufferId,
    clear_color: Option<Color>,

    texture2d_default_shader_id: ShaderId,
}

enum ResolvedResourceType<'a> {
    UniformBuffer(AssetRef<'a, Buffer>),
    Texture(AssetRef<'a, Texture2d>),
}

struct ResolvedDrawCommand<'a> {
    mesh: AssetRef<'a, Mesh>,
    mesh_instances: u32,
    instance_buffer: Option<AssetRef<'a, Buffer>>,
    shader: AssetRef<'a, Shader>,
    vertex_buffers: Vec<AssetRef<'a, Buffer>>,
    bindable_resources: Vec<ResolvedResourceType<'a>>,
}

impl<'f> Renderer<'f> {
    pub fn begin(&mut self, camera: &Camera2d, clear_color: Option<Color>) {
        self.clear_color = clear_color;
        self.framework
            .buffer_write_sync::<Camera2dUniformBlock>(&self.camera_buffer_id, vec![camera.into()]);
    }
    pub fn draw(&mut self, draw_command: DrawCommand) {
        self.draw_queue.push(draw_command)
    }

    pub fn end(&mut self, output: &TextureView) {
        let mut command_encoder = self.create_command_encoder();
        self.execute_draw_queue(&mut command_encoder, output);
        self.submit_frame(command_encoder);
    }

    fn create_command_encoder(&self) -> CommandEncoder {
        let command_encoder_description = CommandEncoderDescriptor {
            label: Some("Framework Renderer command descriptor"),
        };
        self.framework
            .device
            .create_command_encoder(&command_encoder_description)
    }

    fn execute_draw_queue(&mut self, command_encoder: &mut CommandEncoder, output: &TextureView) {
        let load = match self.clear_color.take() {
            Some(color) => LoadOp::Clear(color),
            None => LoadOp::Load,
        };
        let render_pass_description = RenderPassDescriptor {
            label: Some("Renderer pass with clear color"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                ops: Operations { load, store: true },
            })],
            depth_stencil_attachment: None,
        };

        let render_pass = command_encoder.begin_render_pass(&render_pass_description);
        let commands = self.resolve_draw_commands();
        self.execute_commands(render_pass, &commands);
    }

    fn submit_frame(&mut self, command_encoder: CommandEncoder) {
        self.framework
            .queue
            .submit(std::iter::once(command_encoder.finish()));
        self.draw_queue.clear();
    }

    fn resolve_draw_commands(&'f self) -> Vec<ResolvedDrawCommand<'f>> {
        self.draw_queue
            .iter()
            .map(|command| -> ResolvedDrawCommand {
                ResolvedDrawCommand {
                    mesh: self.pick_mesh_from_draw_type(&command.primitives),
                    mesh_instances: command.primitive_count,
                    instance_buffer: self.resolve_instance_buffer(&command),
                    shader: self.pick_shader_from_command(&command),
                    vertex_buffers: self.resolve_vertex_buffers(&command),
                    bindable_resources: self.resolve_bindable_resources(&command),
                }
            })
            .collect()
    }

    fn execute_commands<'a>(
        &self,
        mut render_pass: RenderPass<'a>,
        commands: &'a Vec<ResolvedDrawCommand<'a>>,
    ) {
        for command in commands.iter() {
            render_pass.set_pipeline(&command.shader.render_pipeline);

            for (idx, buffer) in command.vertex_buffers.iter().enumerate() {
                self.bind_vertex_buffer(
                    idx as u32 + Mesh::reserved_buffer_count(),
                    &buffer,
                    &mut render_pass,
                );
            }

            for (idx, resource) in command.bindable_resources.iter().enumerate() {
                self.bind_resource(idx as u32, resource, &mut render_pass);
            }
            if let Some(instance_buffer) = command.instance_buffer.as_ref() {
                self.bind_vertex_buffer(
                    Mesh::INDEX_BUFFER_SLOT,
                    &instance_buffer,
                    &mut render_pass,
                );
                command
                    .mesh
                    .draw_instanced(&mut render_pass, command.mesh_instances);
            } else {
                command.mesh.draw(&mut render_pass);
            }
        }
    }

    fn bind_vertex_buffer<'a>(
        &self,
        idx: u32,
        buffer: &'a AssetRef<'a, Buffer>,
        render_pass: &mut RenderPass<'a>,
    ) {
        debug_assert!(buffer.config.buffer_type == BufferType::Vertex);
        render_pass.set_vertex_buffer(idx, buffer.entire_slice());
    }
    fn bind_resource<'a>(
        &self,
        idx: u32,
        resource: &'a ResolvedResourceType<'a>,
        render_pass: &mut RenderPass<'a>,
    ) {
        let bind_group = match resource {
            ResolvedResourceType::UniformBuffer(buffer) => buffer.bind_group.as_ref().unwrap(),
            ResolvedResourceType::Texture(texture) => &texture.bind_group,
        };
        render_pass.set_bind_group(idx, bind_group, &[])
    }

    fn pick_mesh_from_draw_type<'a>(&self, draw_type: &PrimitiveType) -> AssetRef<'a, Mesh> {
        match draw_type {
            PrimitiveType::Texture2D { .. } => todo!(), // Pick quad mesh
            _ => unreachable!(),
        }
    }

    fn pick_shader_from_command(&self, command: &DrawCommand) -> AssetRef<'f, Shader> {
        let shader_id = if let Some(shader_id) = command.additional_data.shader.as_ref() {
            &shader_id
        } else {
            match command.primitives {
                PrimitiveType::Texture2D { .. } => &self.texture2d_default_shader_id, // Pick quad mesh
                _ => unreachable!(),
            }
        };

        self.framework.shader(shader_id)
    }

    fn resolve_vertex_buffers(&self, command: &DrawCommand) -> Vec<AssetRef<'f, Buffer>> {
        command
            .additional_data
            .additional_vertex_buffers
            .iter()
            .map(|buf_id| {
                let buffer = self.framework.buffer(buf_id);
                debug_assert!(buffer.config.buffer_type == BufferType::Vertex);
                buffer
            })
            .collect()
    }

    fn resolve_bindable_resources<'a>(
        &'a self,
        command: &DrawCommand,
    ) -> Vec<ResolvedResourceType<'a>> {
        let mut specific_draw_resources = self.resolve_draw_type_resources(command);
        let mut additional_draw_resources = command
            .additional_data
            .additional_bindable_resource
            .iter()
            .map(|resource| match &resource {
                BindableResource::UniformBuffer(buf_id) => ResolvedResourceType::UniformBuffer({
                    let buffer = self.framework.buffer(buf_id);
                    debug_assert!(buffer.config.buffer_type == BufferType::Uniform);
                    buffer
                }),
                BindableResource::Texture(tex_id) => {
                    ResolvedResourceType::Texture(self.framework.texture2d(tex_id))
                }
            })
            .collect();
        specific_draw_resources.append(&mut additional_draw_resources);
        specific_draw_resources
    }

    fn resolve_draw_type_resources<'a>(
        &'a self,
        command: &DrawCommand,
    ) -> Vec<ResolvedResourceType<'a>> {
        match &command.primitives {
            PrimitiveType::Texture2D { texture_id, .. } => {
                vec![ResolvedResourceType::Texture(
                    self.framework.texture2d(texture_id),
                )]
            }
            _ => unreachable!(),
        }
    }

    fn resolve_instance_buffer<'a>(
        &'a self,
        command: &DrawCommand,
    ) -> Option<AssetRef<'a, Buffer>> {
        if let Some(buffer_id) = &command.instance_buffer_id {
            let buffer = self.framework.buffer(buffer_id);

            Some(buffer)
        } else {
            None
        }
    }
}
