use cgmath::{point2, vec2, vec3, Point2, Point3};
use framework::{TypedBuffer, TypedBufferConfiguration};
use wgpu::{
    BindGroup, ColorTargetState, FragmentState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, TextureView, VertexState,
};

use framework::asset_library::mesh_names;
use framework::framework::{Framework, TextureId};
use framework::mesh::{Mesh, MeshInstance2D};
use scene::{Camera2d, Camera2dUniformBlock, Transform2d};

struct TextureDrawInfo {
    texture: TextureId,
    instance_data: MeshInstance2D,
}

pub struct Texture2dDrawPass<'framework> {
    pipeline: RenderPipeline,
    textures: Vec<TextureDrawInfo>,
    clear_color: wgpu::Color,
    camera: Camera2d,
    camera_buffer: TypedBuffer<'framework>,
    camera_bind_group: BindGroup,
}

impl<'tex, 'framework> Texture2dDrawPass<'framework> {
    pub fn new(framework: &'framework Framework, output_format: wgpu::TextureFormat) -> Self {
        let module = framework
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/draw_texture2d.wgsl"));

        let camera_buffer =
            framework.allocate_typed_buffer(TypedBufferConfiguration::<Camera2dUniformBlock> {
                initial_setup: framework::typed_buffer::BufferInitialSetup::Size(
                    std::mem::size_of::<Camera2dUniformBlock>() as u64,
                ),
                buffer_type: framework::BufferType::Uniform,
                allow_write: true,
                allow_read: false,
            });
        let bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Layer Pipeline LayerTextures Bind layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let camera_bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture2D Camera Layout"),
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
        let camera_bind_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Texture2D Camera"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        camera_buffer.inner_buffer().as_entire_buffer_binding(),
                    ),
                }],
            });

        let render_pipeline_layout =
            framework
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Texture2D Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let simple_diffuse_pipeline =
            framework
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Texture2D Pipeline"),
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
                            format: output_format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
        Self {
            pipeline: simple_diffuse_pipeline,
            textures: vec![],
            clear_color: wgpu::Color::BLACK,
            camera_buffer,
            camera_bind_group,
            camera: Camera2d::new(0.01, 1000.0, [-1.0, 1.0, 1.0, -1.0]),
        }
    }
    pub fn draw_texture(&mut self, texture: &TextureId, instance_data: MeshInstance2D) {
        self.textures.push(TextureDrawInfo {
            texture: texture.clone(),
            instance_data,
        })
    }

    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.clear_color = color;
    }

    pub fn begin(&mut self, camera: &Camera2d) {
        let mut new_camera = camera.clone();
        new_camera.set_position(point2(camera.position().x, -camera.position().y));
        self.camera = new_camera;
        self.camera_buffer
            .write_sync::<Camera2dUniformBlock>(&vec![(&self.camera).into()]);
    }
    pub fn execute(&mut self, framework: &Framework, output_texture: &TextureView, clear: bool) {
        let render_pass_description = RenderPassDescriptor {
            label: Some("Texture2D Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output_texture,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if clear {
                        wgpu::LoadOp::Clear(self.clear_color)
                    } else {
                        wgpu::LoadOp::Load
                    },
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };

        {
            let quad_mesh = framework.asset_library.mesh(mesh_names::QUAD);
            for texture in self.textures.iter() {
                let mut encoder =
                    framework
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Texture2D Render Pass Encoder"),
                        });
                let instance_buffer = framework.allocate_typed_buffer(TypedBufferConfiguration {
                    initial_setup: framework::typed_buffer::BufferInitialSetup::Data(&vec![
                        texture.instance_data,
                    ]),
                    buffer_type: framework::BufferType::Vertex,
                    allow_write: false,
                    allow_read: false,
                });
                {
                    let framework_texture = framework.texture2d(&texture.texture);
                    let mut pass = encoder.begin_render_pass(&render_pass_description);
                    pass.set_pipeline(&self.pipeline);

                    pass.set_bind_group(0, framework_texture.bind_group(), &[]);
                    pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    pass.set_vertex_buffer(1, instance_buffer.inner_buffer().slice(..));

                    quad_mesh.draw(&mut pass, 1);
                }
                framework.queue.submit(std::iter::once(encoder.finish()));
            }
        }
        self.textures.clear();
    }
}
