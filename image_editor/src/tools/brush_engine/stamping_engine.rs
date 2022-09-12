use cgmath::{point2, vec2, ElementWise, Point2};
use framework::{
    asset_library::mesh_names, Framework, Mesh, MeshInstance2D, TypedBuffer,
    TypedBufferConfiguration,
};
use wgpu::{
    BindGroup, BindGroupLayout, BlendComponent, ColorTargetState, FragmentState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, VertexState,
};

use crate::{layers::BitmapLayer, StrokeContext, StrokePath};

use super::BrushEngine;

pub struct Stamp {
    brush_texture: BitmapLayer,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

pub struct StampCreationInfo<'framework> {
    pub camera_buffer: &'framework TypedBuffer<'framework>,
}

impl<'framework> Stamp {
    pub fn new(
        brush_texture: BitmapLayer,
        framework: &Framework,
        creation_info: StampCreationInfo,
    ) -> Self {
        let bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Layer render pass bind layout"),
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
        let bind_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Stamp render bind group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(brush_texture.texture_view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(brush_texture.sampler()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(
                            creation_info.camera_buffer.binding_resource(),
                        ),
                    },
                ],
            });
        Self {
            brush_texture,
            bind_group,
            bind_group_layout,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StampConfiguration {
    pub color: [f32; 4],
    pub flow: f32,
    pub softness: f32,
    pub padding: [f32; 3],
    pub is_eraser: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StampUniformData {
    pub color: [f32; 4],
    pub flow: f32,
    pub softness: f32,
    pub padding: [f32; 3],
}

impl From<StampConfiguration> for StampUniformData {
    fn from(cfg: StampConfiguration) -> Self {
        Self {
            color: cfg.color,
            flow: cfg.flow,
            softness: cfg.softness,
            padding: cfg.padding,
        }
    }
}

pub struct StrokingEngine<'framework> {
    current_stamp: usize,
    instance_buffer: TypedBuffer<'framework>,
    stamp_pipeline: RenderPipeline,
    eraser_pipeline: RenderPipeline,
    stamps: Vec<Stamp>,
    stamp_data_buffer: TypedBuffer<'framework>,
    configuration: StampConfiguration,
    pub brush_bind_group: BindGroup,
}

impl<'framework, 'stamp> StrokingEngine<'framework> {
    pub fn new(initial_stamp: Stamp, framework: &'framework Framework) -> Self {
        let instance_buffer = TypedBuffer::new(
            framework,
            TypedBufferConfiguration::<MeshInstance2D> {
                initial_data: vec![],
                buffer_type: framework::BufferType::Vertex,
                allow_write: true,
                allow_read: false,
            },
        );
        let initial_setup = StampConfiguration {
            color: [0.0, 1.0, 0.0, 1.0],
            flow: 1.0,
            softness: 0.5,
            padding: [0.0, 0.0, 0.0],
            is_eraser: false,
        };
        let stamp_pipeline = StrokingEngine::create_stamp_pipeline(framework, false);
        let eraser_pipeline = StrokingEngine::create_stamp_pipeline(framework, true);
        let stamp_uniform_buffer = TypedBuffer::new(
            framework,
            TypedBufferConfiguration::<StampUniformData> {
                initial_data: vec![initial_setup.into()],
                buffer_type: framework::BufferType::Uniform,
                allow_write: true,
                allow_read: false,
            },
        );
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
            stamps: vec![initial_stamp],
            current_stamp: 0,
            brush_bind_group,
            stamp_data_buffer: stamp_uniform_buffer,
            configuration: initial_setup,
            instance_buffer,
            stamp_pipeline,
            eraser_pipeline,
        }
    }

    fn create_stamp_pipeline(framework: &Framework, is_eraser: bool) -> RenderPipeline {
        let texture_bind_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("PaintBrush BindGroupLayout"),
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
        let brush_bind_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Brush bind layout"),
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
                    label: Some("StampingEngine Layout"),
                    bind_group_layouts: &[&texture_bind_layout, &brush_bind_layout],
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

        let simple_colored_pipeline =
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
                });
        simple_colored_pipeline
    }

    pub fn create_stamp(
        &self,
        brush_texture: BitmapLayer,
        framework: &Framework,
        info: StampCreationInfo,
    ) -> Stamp {
        Stamp::new(brush_texture, framework, info)
    }

    pub fn settings(&self) -> StampConfiguration {
        self.configuration.clone()
    }

    pub fn set_new_settings(&mut self, settings: StampConfiguration) {
        let unif_data: StampUniformData = settings.into();
        self.stamp_data_buffer.write_sync(&[unif_data]);
        self.configuration = settings;
    }

    fn current_stamp(&self) -> &Stamp {
        self.stamps
            .get(self.current_stamp)
            .expect("Could not find the given index in stamp array")
    }
}

impl<'framework> BrushEngine for StrokingEngine<'framework> {
    fn stroke(&mut self, path: StrokePath, context: StrokeContext) {
        match context.layer.layer_type {
            crate::layers::LayerType::Bitmap(ref bitmap_layer) => {
                let one_over_scale = 1.0 / context.editor.camera().current_scale();
                let top_left = context.editor.camera().ndc_into_world(point2(-1.0, 1.0));
                let bottom_right = context.editor.camera().ndc_into_world(point2(1.0, -1.0));
                let width = (bottom_right.x - top_left.x).abs() * one_over_scale;
                let height = (top_left.y - bottom_right.y).abs() * one_over_scale;
                let x_ratio = bitmap_layer.size().x / width;
                let y_ratio = bitmap_layer.size().y / height;

                let actual_layer_scale =
                    bitmap_layer.size().mul_element_wise(context.layer.scale) * one_over_scale;
                let layer_ratio = actual_layer_scale.div_element_wise(bitmap_layer.size());
                let lrp = point2(layer_ratio.x * x_ratio, layer_ratio.y * y_ratio);

                let correct_point = |point: Point2<f32>| {
                    let point = point.div_element_wise(lrp);
                    let camera_displace = context.editor.camera().position().mul_element_wise(-1.0);
                    let pt = point.add_element_wise(camera_displace);
                    context.debug.borrow_mut().draw_debug_point(
                        pt,
                        vec2(3.0, 3.0),
                        [0.0, 1.0, 0.0, 1.0],
                    );
                    pt
                };

                // 1. Update buffer
                let instances: Vec<MeshInstance2D> = path
                    .points
                    .iter()
                    .map(|pt| {
                        MeshInstance2D::new(
                            correct_point(pt.position),
                            vec2(pt.size, pt.size) * context.editor.camera().current_scale(),
                            0.0,
                        )
                    })
                    .collect();

                self.instance_buffer.write_sync(&instances.as_slice());
                // 2. Do draw
                let stroking_engine_render_pass = RenderPassDescriptor {
                    label: Some("Stamping Engine render pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: bitmap_layer.texture_view(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                };
                let mut render_pass = context
                    .command_encoder
                    .begin_render_pass(&stroking_engine_render_pass);

                if self.configuration.is_eraser {
                    render_pass.set_pipeline(&self.eraser_pipeline);
                } else {
                    render_pass.set_pipeline(&self.stamp_pipeline);
                }
                render_pass.set_bind_group(0, &self.current_stamp().bind_group, &[]);
                render_pass.set_bind_group(1, &self.brush_bind_group, &[]);
                self.instance_buffer.bind(1, &mut render_pass);
                context
                    .assets
                    .mesh(mesh_names::QUAD)
                    .draw(&mut render_pass, instances.len() as u32);
            }
        }
    }
}
