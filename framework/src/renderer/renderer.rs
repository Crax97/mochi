use cgmath::{point2, vec2};
use wgpu::{
    include_wgsl, Color, CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, TextureFormat, TextureView,
};

use crate::{
    buffer::BufferInitialSetup,
    framework::{BufferId, ShaderId},
    shader::{Shader, ShaderCreationInfo},
    AssetRef, Buffer, BufferConfiguration, BufferType, Camera2d, Camera2dUniformBlock, Framework,
    Mesh, MeshInstance2D, Texture2d,
};

use super::draw_command::{BindableResource, DrawCommand, DrawMode, PrimitiveType};

enum ResolvedResourceType<'a> {
    UniformBuffer(AssetRef<'a, Buffer>),
    Texture(AssetRef<'a, Texture2d>),
}

enum ResolvedDrawType<'a> {
    Instanced {
        buffer: AssetRef<'a, Buffer>,
        elements: u32,
    },
    Separate(Vec<AssetRef<'a, Buffer>>),
}

struct ResolvedDrawCommand<'a> {
    mesh: AssetRef<'a, Mesh>,
    draw_type: ResolvedDrawType<'a>,
    shader: AssetRef<'a, Shader>,
    vertex_buffers: Vec<AssetRef<'a, Buffer>>,
    bindable_resources: Vec<ResolvedResourceType<'a>>,
}

pub struct Renderer<'f> {
    framework: &'f Framework,

    draw_queue: Vec<DrawCommand>,
    camera_buffer_id: BufferId,
    clear_color: Option<Color>,

    texture2d_instanced_shader_id: ShaderId,
    texture2d_single_shader_id: ShaderId,
}

impl<'f> Renderer<'f> {
    pub fn new(framework: &'f Framework) -> Self {
        let camera_buffer_id =
            framework.allocate_typed_buffer(BufferConfiguration::<Camera2dUniformBlock> {
                initial_setup: BufferInitialSetup::Count(1),
                buffer_type: BufferType::Uniform,
                allow_write: true,
                allow_read: false,
            });
        let texture2d_instanced_shader_id =
            framework.create_shader(Renderer::texture2d_shader_creation_info(framework));
        let texture2d_single_shader_id =
            framework.create_shader(ShaderCreationInfo::using_default_vertex_fragment(framework));
        Self {
            framework,
            camera_buffer_id,
            draw_queue: vec![],
            clear_color: None,

            texture2d_instanced_shader_id,
            texture2d_single_shader_id,
        }
    }

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
                    draw_type: self.resolve_draw_type(&command),
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

            match &command.draw_type {
                ResolvedDrawType::Instanced { buffer, elements } => {
                    self.bind_vertex_buffer(Mesh::INDEX_BUFFER_SLOT, &buffer, &mut render_pass);
                    command.mesh.draw_instanced(&mut render_pass, *elements);
                }
                ResolvedDrawType::Separate(buffers) => {
                    for buffer in buffers {
                        self.bind_vertex_buffer(Mesh::INDEX_BUFFER_SLOT, &buffer, &mut render_pass);
                        command.mesh.draw(&mut render_pass);
                    }
                }
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
                PrimitiveType::Texture2D { .. } => match command.draw_mode {
                    DrawMode::Instanced(_) => &self.texture2d_instanced_shader_id,
                    DrawMode::Single => &self.texture2d_single_shader_id,
                }, // Pick quad mesh
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

    fn resolve_draw_type<'a>(&self, command: &DrawCommand) -> ResolvedDrawType {
        match command.draw_mode {
            DrawMode::Instanced(instances) => {
                self.build_instance_buffer_for_primitive_type(&command.primitives)
            }
            DrawMode::Single => self.build_uniform_buffers_for_primitive_type(&command.primitives),
        }
    }

    pub fn texture2d_shader_creation_info(framework: &Framework) -> ShaderCreationInfo {
        ShaderCreationInfo::using_default_vertex_fragment_instanced(framework)
    }

    fn build_instance_buffer_for_primitive_type(
        &self,
        primitives: &PrimitiveType,
    ) -> ResolvedDrawType {
        match primitives {
            PrimitiveType::Noop => unreachable!(),
            PrimitiveType::Texture2D { instances, .. } => {
                let mesh_instances_2d = instances
                    .iter()
                    .map(|inst| {
                        MeshInstance2D::new(
                            point2(inst.position.x, inst.position.y),
                            vec2(inst.scale.x, inst.scale.y),
                            inst.rotation_radians.0,
                            true,
                            1.0,
                        )
                    })
                    .collect();
                let buffer_id = self.framework.allocate_typed_buffer(BufferConfiguration {
                    initial_setup: BufferInitialSetup::Data(&mesh_instances_2d),
                    buffer_type: BufferType::Vertex,
                    allow_write: false,
                    allow_read: true,
                });
                ResolvedDrawType::Instanced {
                    buffer: self.framework.buffer(&buffer_id),
                    elements: instances.len() as u32,
                }
            }
        }
    }

    fn build_uniform_buffers_for_primitive_type(
        &self,
        primitives: &PrimitiveType,
    ) -> ResolvedDrawType {
        match primitives {
            PrimitiveType::Noop => unreachable!(),
            PrimitiveType::Texture2D { instances, .. } => {
                let instances = instances
                    .iter()
                    .map(|inst| {
                        MeshInstance2D::new(
                            point2(inst.position.x, inst.position.y),
                            vec2(inst.scale.x, inst.scale.y),
                            inst.rotation_radians.0,
                            true,
                            1.0,
                        )
                    })
                    .map(|instance| {
                        self.framework.allocate_typed_buffer(BufferConfiguration {
                            initial_setup: BufferInitialSetup::Data(&vec![instance]),
                            buffer_type: BufferType::Uniform,
                            allow_write: false,
                            allow_read: true,
                        })
                    })
                    .map(|buffer_id| self.framework.buffer(&buffer_id));
                ResolvedDrawType::Separate(instances.collect())
            }
        }
    }
}
