use cgmath::{point2, point3, vec2};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor, Color, CommandEncoder,
    CommandEncoderDescriptor, LoadOp, Operations, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureView,
};

use crate::{
    buffer::BufferInitialSetup,
    framework::{BufferId, DepthStencilTextureId, MeshId, ShaderId, TextureId},
    shader::{Shader, ShaderCreationInfo},
    Buffer, BufferConfiguration, BufferType, Camera2d, Camera2dUniformBlock, Framework,
    GpuDepthStencilTexture2D, GpuRgbaTexture2D, Mesh, MeshConstructionDetails, MeshInstance2D,
    RgbaTexture2D, Texture, Vertex,
};

use super::draw_command::{BindableResource, DrawCommand, DrawMode, PrimitiveType};

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum DepthStencilUsage {
    Depth,
    Stencil,
}

enum ResolvedResourceType<'a> {
    UniformBuffer(&'a Buffer),
    EmptyBindGroup,
    Texture(&'a GpuRgbaTexture2D),
}

enum DrawType {
    Instanced { buffer: BufferId, elements: u32 },
    Separate(Vec<BufferId>),
}

enum ResolvedDrawType<'a> {
    Instanced { buffer: &'a Buffer, elements: u32 },
    Separate(Vec<ResolvedResourceType<'a>>),
}

struct ResolvedDrawCommand<'a> {
    mesh: &'a Mesh,
    draw_type: ResolvedDrawType<'a>,
    shader: &'a Shader,
    vertex_buffers: Vec<&'a Buffer>,
    bindable_resources: Vec<(u32, ResolvedResourceType<'a>)>,
}

pub struct Renderer {
    draw_queue: Vec<DrawCommand>,
    camera_buffer_id: BufferId,
    clear_color: Option<Color>,
    clear_depth: Option<f32>,
    clear_stencil: Option<u32>,
    viewport: Option<(f32, f32, f32, f32)>,
    empty_bind_group: BindGroup,

    texture2d_instanced_shader_id: ShaderId,
    texture2d_single_shader_id: ShaderId,

    render_pass_debug_name: Option<String>,
    depth_stencil_target: Option<TextureId>,
    stencil_value: Option<u32>,

    white_texture_id: TextureId,

    quad_mesh_id: MeshId,
}

impl Renderer {
    const DIFFUSE_BIND_GROUP_LOCATION: u32 = 2;

    fn construct_initial_quad(framework: &mut Framework) -> MeshId {
        let quad_mesh_vertices = [
            Vertex {
                position: point3(-1.0, 1.0, 0.0),
                tex_coords: point2(0.0, 1.0),
            },
            Vertex {
                position: point3(1.0, 1.0, 0.0),
                tex_coords: point2(1.0, 1.0),
            },
            Vertex {
                position: point3(-1.0, -1.0, 0.0),
                tex_coords: point2(0.0, 0.0),
            },
            Vertex {
                position: point3(1.0, -1.0, 0.0),
                tex_coords: point2(1.0, 0.0),
            },
        ]
        .into();

        let indices = [0u16, 1, 2, 2, 1, 3].into();
        let construction_info = MeshConstructionDetails {
            vertices: quad_mesh_vertices,
            indices,
            allow_editing: false,
            primitives: 6,
        };
        framework.allocate_mesh(construction_info)
    }

    fn empty_bind_group(framework: &Framework) -> BindGroup {
        let layout = framework
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[],
            });
        framework.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[],
        })
    }

    pub fn new(framework: &mut Framework) -> Self {
        let camera_buffer_id =
            framework.allocate_typed_buffer(BufferConfiguration::<Camera2dUniformBlock> {
                initial_setup: BufferInitialSetup::Count(1),
                buffer_type: BufferType::Uniform,
                allow_write: true,
                allow_read: false,
            });
        let texture2d_instanced_shader_id = framework.create_shader(
            ShaderCreationInfo::using_default_vertex_fragment_instanced(&framework),
        );
        let texture2d_single_shader_id = framework.create_shader(
            ShaderCreationInfo::using_default_vertex_fragment(&framework),
        );

        let quad_mesh_id = Renderer::construct_initial_quad(framework);
        let empty_bind_group = Renderer::empty_bind_group(framework);

        let white_cpu_texture = RgbaTexture2D::from_bytes(&[255, 255, 255, 255], (1, 1)).unwrap();
        let white_texture_id = framework.allocate_texture2d(
            white_cpu_texture,
            crate::TextureConfiguration {
                label: Some("White texture"),
                usage: crate::texture::TextureUsage::READ_WRITE,
                mip_count: None,
            },
        );

        Self {
            camera_buffer_id,
            draw_queue: vec![],
            clear_color: None,
            clear_depth: None,
            clear_stencil: None,
            viewport: None,
            empty_bind_group,
            render_pass_debug_name: None,
            depth_stencil_target: None,
            stencil_value: None,

            texture2d_instanced_shader_id,
            texture2d_single_shader_id,
            white_texture_id,
            quad_mesh_id,
        }
    }

    pub fn begin(
        &mut self,
        camera: &Camera2d,
        clear_color: Option<Color>,
        framework: &mut Framework,
    ) {
        self.clear_color = clear_color;
        framework
            .buffer_write_sync::<Camera2dUniformBlock>(&self.camera_buffer_id, vec![camera.into()]);
        self.draw_queue.clear();
    }

    pub fn set_viewport(&mut self, viewport: Option<(f32, f32, f32, f32)>) {
        self.viewport = viewport;
    }

    pub fn set_draw_debug_name(&mut self, name: &str) {
        self.render_pass_debug_name = Some(name.to_owned());
    }

    pub fn set_depth_stencil_target(&mut self, new_target: Option<TextureId>) {
        self.depth_stencil_target = new_target;
    }

    pub fn set_stencil_reference(&mut self, new_value: u32) {
        self.stencil_value = Some(new_value);
    }
    pub fn set_depth_clear(&mut self, new_depth: Option<f32>) {
        self.clear_depth = new_depth;
    }

    pub fn set_stencil_clear(&mut self, new_value: Option<u32>) {
        self.clear_stencil = new_value;
    }

    pub fn draw(&mut self, draw_command: DrawCommand) {
        self.draw_queue.push(draw_command)
    }

    pub fn end(
        &mut self,
        output: &TextureId,
        depth_stencil_output: Option<(&DepthStencilTextureId, DepthStencilUsage)>,
        framework: &mut Framework,
    ) {
        // let texture = framework.allocated_textures.map.get(&output.index).unwrap();
        // self.end(&texture.value.texture_view, None, framework);
        let command_encoder_description = CommandEncoderDescriptor {
            label: Some("Framework Renderer command descriptor"),
        };
        let mut command_encoder = framework
            .device
            .create_command_encoder(&command_encoder_description);

        let draw_commands_with_buffers = self.generate_partial_draws(framework);
        let commands = self.resolve_draw_commands(framework, draw_commands_with_buffers);

        let texture = &framework.texture2d(output).texture_view(0);
        let depth_texture_view = depth_stencil_output
            .map(|tex_id| (framework.depth_stencil_texture(tex_id.0), tex_id.1));
        self.execute_draw_queue(
            &mut command_encoder,
            texture,
            depth_texture_view,
            commands,
            framework,
        );
        self.submit_frame(command_encoder, framework);
    }
    pub fn end_on_external_texture(&mut self, output: &TextureView, framework: &mut Framework) {
        // let texture = framework.allocated_textures.map.get(&output.index).unwrap();
        // self.end(&texture.value.texture_view, None, framework);
        let command_encoder_description = CommandEncoderDescriptor {
            label: Some("Framework Renderer command descriptor"),
        };
        let mut command_encoder = framework
            .device
            .create_command_encoder(&command_encoder_description);

        let draw_commands_with_buffers = self.generate_partial_draws(framework);
        let commands = self.resolve_draw_commands(framework, draw_commands_with_buffers);

        self.execute_draw_queue(&mut command_encoder, output, None, commands, framework);
        self.submit_frame(command_encoder, framework);
    }

    fn generate_partial_draws(
        &mut self,
        framework: &mut Framework,
    ) -> Vec<(DrawType, DrawCommand)> {
        let mut partial_draws: Vec<(DrawType, DrawCommand)> = vec![];
        for draw in self.draw_queue.iter() {
            let draw_type = self.generate_draw_type(&draw, framework);
            partial_draws.push((draw_type, draw.clone()))
        }
        partial_draws
    }

    fn generate_draw_type(&self, command: &DrawCommand, framework: &mut Framework) -> DrawType {
        match command.draw_mode {
            DrawMode::Instanced => {
                self.build_instance_buffer_for_primitive_type(&command, framework)
            }
            DrawMode::Single => self.build_uniform_buffers_for_primitive_type(&command, framework),
        }
    }

    fn resolve_draw_commands<'f>(
        &self,
        framework: &'f Framework,
        partial_draws: Vec<(DrawType, DrawCommand)>,
    ) -> Vec<ResolvedDrawCommand<'f>> {
        let mut commands: Vec<ResolvedDrawCommand> = vec![];
        for (draw, command) in partial_draws.into_iter() {
            commands.push(ResolvedDrawCommand {
                mesh: self.pick_mesh_from_draw_type(&command.primitives, framework),
                draw_type: self.resolve_draw_type(draw, framework),
                shader: self.pick_shader_from_command(&command, framework),
                vertex_buffers: self.resolve_vertex_buffers(&command, framework),
                bindable_resources: self.resolve_bindable_resources(&command, framework),
            });
        }
        commands
    }

    fn execute_draw_queue(
        &mut self,
        command_encoder: &mut CommandEncoder,
        output: &TextureView,
        depth_output: Option<(&GpuDepthStencilTexture2D, DepthStencilUsage)>,
        commands: Vec<ResolvedDrawCommand>,
        framework: &Framework,
    ) {
        let depth_load = Operations {
            load: if let Some(depth) = self.clear_depth {
                LoadOp::Clear(depth)
            } else {
                LoadOp::Load
            },
            store: true,
        };
        let stencil_load = Operations {
            load: if let Some(stencil) = self.clear_stencil {
                LoadOp::Clear(stencil)
            } else {
                LoadOp::Load
            },
            store: true,
        };
        let load = match self.clear_color.take() {
            Some(color) => LoadOp::Clear(color),
            None => LoadOp::Load,
        };
        let pass_name = self.render_pass_debug_name.take();
        let render_pass_description = RenderPassDescriptor {
            label: pass_name
                .as_deref()
                .or(Some("Renderer pass with clear color")),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                ops: Operations { load, store: true },
            })],
            depth_stencil_attachment: if let Some((texture, usage)) = depth_output {
                Some(RenderPassDepthStencilAttachment {
                    view: if usage == DepthStencilUsage::Depth {
                        texture.depth_view()
                    } else {
                        texture.stencil_view()
                    },
                    depth_ops: Some(depth_load),
                    stencil_ops: Some(stencil_load),
                })
            } else {
                None
            },
        };

        let mut render_pass = command_encoder.begin_render_pass(&render_pass_description);

        if let Some(stencil_reference) = self.stencil_value.take() {
            render_pass.set_stencil_reference(stencil_reference);
        }

        if let Some(viewport) = self.viewport.take() {
            render_pass.set_viewport(viewport.0, viewport.1, viewport.2, viewport.3, 0.0, 1.0);
        }
        let camera_buffer =
            ResolvedResourceType::UniformBuffer(framework.buffer(&self.camera_buffer_id));
        self.execute_commands(render_pass, &camera_buffer, &commands);
    }

    fn execute_commands<'a>(
        &'a self,
        mut render_pass: RenderPass<'a>,
        camera_buffer: &'a ResolvedResourceType<'a>,
        commands: &'a Vec<ResolvedDrawCommand<'a>>,
    ) {
        for command in commands.iter() {
            render_pass.set_pipeline(&command.shader.render_pipeline);

            self.bind_resource(0, camera_buffer, &mut render_pass);

            for (idx, buffer) in command.vertex_buffers.iter().enumerate() {
                self.bind_vertex_buffer(
                    idx as u32 + Mesh::reserved_buffer_count(),
                    &buffer,
                    &mut render_pass,
                );
            }

            for (idx, resource) in command.bindable_resources.iter() {
                self.bind_resource(*idx, resource, &mut render_pass);
            }

            match &command.draw_type {
                ResolvedDrawType::Instanced { buffer, elements } => {
                    self.bind_vertex_buffer(Mesh::INDEX_BUFFER_SLOT, &buffer, &mut render_pass);
                    command.mesh.draw_instanced(&mut render_pass, *elements);
                }
                ResolvedDrawType::Separate(buffers) => {
                    for buffer in buffers {
                        self.bind_resource(Mesh::MESH_INFO_UNIFORM_SLOT, buffer, &mut render_pass);
                    }
                    command.mesh.draw(&mut render_pass);
                }
            }
        }
    }
    fn submit_frame(&mut self, command_encoder: CommandEncoder, framework: &Framework) {
        framework
            .queue
            .submit(std::iter::once(command_encoder.finish()));
        self.draw_queue.clear();
    }

    fn bind_vertex_buffer<'a>(
        &self,
        idx: u32,
        buffer: &'a Buffer,
        render_pass: &mut RenderPass<'a>,
    ) {
        debug_assert!(buffer.config.buffer_type == BufferType::Vertex);
        render_pass.set_vertex_buffer(idx, buffer.entire_slice());
    }
    fn bind_resource<'a>(
        &'a self,
        idx: u32,
        resource: &'a ResolvedResourceType<'a>,
        render_pass: &mut RenderPass<'a>,
    ) {
        let bind_group = match resource {
            ResolvedResourceType::UniformBuffer(buffer) => buffer.bind_group.as_ref().unwrap(),
            ResolvedResourceType::Texture(texture) => texture.bind_group(0),
            ResolvedResourceType::EmptyBindGroup => &self.empty_bind_group,
        };
        render_pass.set_bind_group(idx, bind_group, &[])
    }

    fn pick_mesh_from_draw_type<'f, 'b>(
        &self,
        draw_type: &PrimitiveType,
        framework: &'f Framework,
    ) -> &'b Mesh
    where
        'f: 'b,
    {
        let mesh_id = match draw_type {
            PrimitiveType::Noop => unreachable!(),
            PrimitiveType::Texture2D { .. } | PrimitiveType::Rect { .. } => &self.quad_mesh_id, // Pick quad mesh
        };
        framework.mesh(&mesh_id)
    }

    fn pick_shader_from_command<'f, 'b>(
        &self,
        command: &DrawCommand,
        framework: &'f Framework,
    ) -> &'b Shader
    where
        'f: 'b,
    {
        let shader_id = if let Some(shader_id) = command.additional_data.shader.as_ref() {
            &shader_id
        } else {
            match command.primitives {
                PrimitiveType::Noop => unreachable!(),
                PrimitiveType::Texture2D { .. } | PrimitiveType::Rect { .. } => {
                    match command.draw_mode {
                        DrawMode::Instanced => &self.texture2d_instanced_shader_id,
                        DrawMode::Single => &self.texture2d_single_shader_id,
                    }
                } // Pick quad mesh
            }
        };

        framework.shader(shader_id)
    }

    fn resolve_vertex_buffers<'f, 'b>(
        &self,
        command: &DrawCommand,
        framework: &'f Framework,
    ) -> Vec<&'b Buffer>
    where
        'f: 'b,
    {
        command
            .additional_data
            .additional_vertex_buffers
            .iter()
            .map(|buf_id| {
                let buffer = framework.buffer(buf_id);
                debug_assert!(buffer.config.buffer_type == BufferType::Vertex);
                buffer
            })
            .collect()
    }

    fn resolve_bindable_resources<'f, 'b>(
        &self,
        command: &DrawCommand,
        framework: &'f Framework,
    ) -> Vec<(u32, ResolvedResourceType<'b>)>
    where
        'f: 'b,
    {
        let mut specific_draw_resources = self.resolve_draw_type_resources(command, framework);
        let mut additional_draw_resources = command
            .additional_data
            .additional_bindable_resource
            .iter()
            .enumerate()
            .map(|(idx, resource)| {
                (
                    idx as u32 + Shader::reserved_buffer_count(),
                    match &resource {
                        BindableResource::UniformBuffer(buf_id) => {
                            ResolvedResourceType::UniformBuffer({
                                let buffer = framework.buffer(buf_id);
                                debug_assert!(buffer.config.buffer_type == BufferType::Uniform);
                                buffer
                            })
                        }
                        BindableResource::Texture(tex_id) => {
                            ResolvedResourceType::Texture(framework.texture2d(tex_id))
                        }
                    },
                )
            })
            .collect();
        specific_draw_resources.append(&mut additional_draw_resources);
        specific_draw_resources
    }

    fn resolve_draw_type_resources<'a>(
        &self,
        command: &DrawCommand,
        framework: &'a Framework,
    ) -> Vec<(u32, ResolvedResourceType<'a>)> {
        match &command.primitives {
            PrimitiveType::Noop => unreachable!(),
            PrimitiveType::Texture2D { texture_id, .. } => {
                vec![
                    (1, ResolvedResourceType::EmptyBindGroup),
                    (
                        Renderer::DIFFUSE_BIND_GROUP_LOCATION,
                        ResolvedResourceType::Texture(framework.texture2d(texture_id)),
                    ),
                ]
            }
            PrimitiveType::Rect { .. } => {
                vec![
                    (1, ResolvedResourceType::EmptyBindGroup),
                    (
                        Renderer::DIFFUSE_BIND_GROUP_LOCATION,
                        ResolvedResourceType::Texture(framework.texture2d(&self.white_texture_id)),
                    ),
                ]
            }
        }
    }

    fn resolve_draw_type<'f, 'b>(
        &self,
        draw: DrawType,
        framework: &'f Framework,
    ) -> ResolvedDrawType<'b>
    where
        'f: 'b,
    {
        match draw {
            DrawType::Instanced { buffer, elements } => ResolvedDrawType::Instanced {
                buffer: framework.buffer(&buffer),
                elements,
            },
            DrawType::Separate(draws) => {
                let mut buffers: Vec<ResolvedResourceType> = vec![];
                buffers.reserve(draws.len());
                for b in draws {
                    buffers.push(ResolvedResourceType::UniformBuffer(framework.buffer(&b)));
                }
                ResolvedDrawType::Separate(buffers)
            }
        }
    }

    fn build_instance_buffer_for_primitive_type<'f>(
        &self,
        command: &DrawCommand,
        framework: &'f mut Framework,
    ) -> DrawType {
        match &command.primitives {
            PrimitiveType::Noop => unreachable!(),
            PrimitiveType::Texture2D {
                instances,
                flip_uv_y,
                multiply_color,
                ..
            } => {
                let mesh_instances_2d = instances
                    .iter()
                    .map(|inst| {
                        MeshInstance2D::new(
                            point2(inst.position.x, inst.position.y),
                            vec2(inst.scale.x, inst.scale.y),
                            inst.rotation_radians.0,
                            *flip_uv_y,
                            *multiply_color,
                        )
                    })
                    .collect();
                let buffer_id = framework.allocate_typed_buffer(BufferConfiguration {
                    initial_setup: BufferInitialSetup::Data(&mesh_instances_2d),
                    buffer_type: BufferType::Vertex,
                    allow_write: false,
                    allow_read: true,
                });
                DrawType::Instanced {
                    buffer: buffer_id,
                    elements: instances.len() as u32,
                }
            }
            PrimitiveType::Rect {
                rects,
                multiply_color,
            } => {
                let mesh_instances_2d = rects
                    .iter()
                    .map(|rect| {
                        MeshInstance2D::new(
                            rect.center(),
                            rect.extents,
                            0.0,
                            false,
                            multiply_color.clone(),
                        )
                    })
                    .collect();
                let buffer_id = framework.allocate_typed_buffer(BufferConfiguration {
                    initial_setup: BufferInitialSetup::Data(&mesh_instances_2d),
                    buffer_type: BufferType::Vertex,
                    allow_write: false,
                    allow_read: true,
                });
                DrawType::Instanced {
                    buffer: buffer_id,
                    elements: rects.len() as u32,
                }
            }
        }
    }

    fn build_uniform_buffers_for_primitive_type<'a>(
        &self,
        command: &DrawCommand,
        framework: &'a mut Framework,
    ) -> DrawType {
        match &command.primitives {
            PrimitiveType::Noop => unreachable!(),
            PrimitiveType::Texture2D {
                instances,
                flip_uv_y,
                multiply_color,
                ..
            } => {
                let instances = instances.iter().map(|inst| {
                    MeshInstance2D::new(
                        point2(inst.position.x, inst.position.y),
                        vec2(inst.scale.x, inst.scale.y),
                        inst.rotation_radians.0,
                        *flip_uv_y,
                        *multiply_color,
                    )
                });
                let mut buffer_ids: Vec<BufferId> = vec![];
                for instance in instances {
                    let buffer_id = framework.allocate_typed_buffer(BufferConfiguration {
                        initial_setup: BufferInitialSetup::Data(&vec![instance]),
                        buffer_type: BufferType::Uniform,
                        allow_write: false,
                        allow_read: true,
                    });
                    buffer_ids.push(buffer_id);
                }
                DrawType::Separate(buffer_ids)
            }
            PrimitiveType::Rect {
                rects,
                multiply_color,
            } => {
                let instances = rects.iter().map(|rect| {
                    MeshInstance2D::new(
                        rect.center(),
                        rect.extents,
                        0.0,
                        false,
                        multiply_color.clone(),
                    )
                });
                let mut buffer_ids: Vec<BufferId> = vec![];
                for instance in instances {
                    let buffer_id = framework.allocate_typed_buffer(BufferConfiguration {
                        initial_setup: BufferInitialSetup::Data(&vec![instance]),
                        buffer_type: BufferType::Uniform,
                        allow_write: false,
                        allow_read: true,
                    });
                    buffer_ids.push(buffer_id);
                }
                DrawType::Separate(buffer_ids)
            }
        }
    }
}
