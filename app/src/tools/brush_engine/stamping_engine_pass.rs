use framework::{
    asset_library::mesh_names, AssetsLibrary, Framework, Mesh, MeshInstance2D, Texture2d,
    TypedBuffer, TypedBufferConfiguration,
};
use wgpu::{
    BindGroup, BlendComponent, ColorTargetState, FragmentState, RenderPipeline, VertexState,
};

use crate::stamping_engine::{StampConfiguration, StampUniformData};

pub struct StampingEngineRenderPass<'framework> {
    instance_buffer: TypedBuffer<'framework>,
    stamp_pipeline: RenderPipeline,
    eraser_pipeline: RenderPipeline,
    stamp_data_buffer: TypedBuffer<'framework>,
    brush_bind_group: BindGroup,
    stamp_settings: StampConfiguration,
}
impl<'framework> StampingEngineRenderPass<'framework> {
    pub fn new(framework: &'framework Framework) -> Self {
        let instance_buffer =
            framework.allocate_typed_buffer(TypedBufferConfiguration::<MeshInstance2D> {
                initial_setup: framework::typed_buffer::BufferInitialSetup::Data(&vec![]),
                buffer_type: framework::BufferType::Vertex,
                allow_write: true,
                allow_read: false,
            });
        let initial_setup = StampConfiguration {
            color_srgb: [0, 0, 0],
            opacity: 255,
            flow: 1.0,
            softness: 0.2,
            padding: [1.0, 1.0, 1.0],
            is_eraser: false,
        };
        let texture_bind_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("PaintBrush Stamp Bind Layout"),
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
        let brush_bind_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("PaintBrush Brush Settings Bind layout"),
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
        let camera_bind_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("BitmapLayer camera bind layout"),
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
        let render_pipeline_layout =
            framework
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("PaintBrush StampingEngine Layout"),
                    bind_group_layouts: &[
                        &texture_bind_layout,
                        &brush_bind_layout,
                        &camera_bind_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let module = framework
            .device
            .create_shader_module(wgpu::include_wgsl!("./stamp_brush.wgsl"));

        let eraser_blend_state = wgpu::BlendState {
            color: BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let brush_blend_state = wgpu::BlendState {
            color: BlendComponent::OVER,
            alpha: BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Max,
            },
        };

        let make_pipeline = |is_eraser: bool| {
            framework
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("StampingEngine pipeline"),
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
                            blend: Some(if is_eraser {
                                eraser_blend_state
                            } else {
                                brush_blend_state
                            }),
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
                })
        };
        let stamp_pipeline = make_pipeline(false);
        let eraser_pipeline = make_pipeline(true);
        let stamp_uniform_buffer =
            framework.allocate_typed_buffer(TypedBufferConfiguration::<StampUniformData> {
                initial_setup: framework::typed_buffer::BufferInitialSetup::Data(&vec![
                    initial_setup.into(),
                ]),
                buffer_type: framework::BufferType::Uniform,
                allow_write: true,
                allow_read: false,
            });
        let texture_bind_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("PaintBrush BindGroupLayout"),
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
        let brush_bind_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Stamp data bind group"),
                layout: &texture_bind_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        stamp_uniform_buffer.binding_resource(),
                    ),
                }],
            });

        Self {
            brush_bind_group,
            stamp_data_buffer: stamp_uniform_buffer,
            instance_buffer,
            stamp_pipeline,
            eraser_pipeline,
            stamp_settings: initial_setup,
        }
    }

    pub(crate) fn update(&mut self, instances: Vec<MeshInstance2D>) {
        self.instance_buffer.write_sync(&instances);
    }

    pub(crate) fn set_stamp_settings(&mut self, settings: StampConfiguration) {
        let unif_data: StampUniformData = settings.into();
        self.stamp_data_buffer.write_sync(&vec![unif_data]);
        self.stamp_settings = settings;
    }

    pub(crate) fn get_stamp_settings(&self) -> StampConfiguration {
        self.stamp_settings
    }

    pub fn execute<'s, 'call, 'pass>(
        &'s self,
        mut pass: wgpu::RenderPass<'pass>,
        stamp: &'pass Texture2d,
        asset_library: &'pass AssetsLibrary,
        camera_bind_group: &'pass BindGroup,
    ) where
        'pass: 'call,
        's: 'pass,
    {
        if self.stamp_settings.is_eraser {
            pass.set_pipeline(&self.eraser_pipeline);
        } else {
            pass.set_pipeline(&self.stamp_pipeline);
        }

        pass.set_bind_group(0, stamp.bind_group(), &[]);
        pass.set_bind_group(1, &self.brush_bind_group, &[]);
        pass.set_bind_group(2, &camera_bind_group, &[]);
        pass.set_vertex_buffer(1, self.instance_buffer.entire_slice());
        asset_library
            .mesh(mesh_names::QUAD)
            .draw(&mut pass, self.instance_buffer.elem_count() as u32)
    }
}
