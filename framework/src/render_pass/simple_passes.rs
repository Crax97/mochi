use std::{cell::RefCell, rc::Rc};

use wgpu::{ColorTargetState, CommandBuffer, FragmentState, RenderPipeline, VertexState};

use crate::{asset_library, mesh_names, AssetsLibrary, DebugInstance2D, Mesh, MeshInstance2D};

use super::{render_pass, PassBindble, RenderPass};

pub struct SimpleTexturedPass {
    pipeline: RenderPipeline,
    asset_library: Rc<RefCell<AssetsLibrary>>,
}

impl SimpleTexturedPass {
    pub fn new(framework: &crate::Framework, asset_library: Rc<RefCell<AssetsLibrary>>) -> Self {
        let module = framework
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/simple_shader.wgsl"));

        let bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Simple textured bind group layout"),
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

        let camera_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Simple textured bind group layout"),
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
                    label: Some("Simple textured Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout, &camera_layout],
                    push_constant_ranges: &[],
                });

        let simple_diffuse_pipeline =
            framework
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Simple textured pipeline"),
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
            asset_library,
        }
    }
}

impl RenderPass for SimpleTexturedPass {
    fn execute_with_renderpass<'s, 'call, 'pass>(
        &'s self,
        mut pass: wgpu::RenderPass<'pass>,
        items: &'call [(u32, &'pass dyn PassBindble)],
    ) where
        'pass: 'call,
        's: 'pass,
    {
        pass.set_pipeline(&self.pipeline);
        self.bind_all(&mut pass, items);
        self.asset_library
            .borrow()
            .mesh(mesh_names::QUAD)
            .draw(pass, 1);
    }
}

pub struct SimpleColoredPass {
    pipeline: RenderPipeline,
    asset_library: Rc<RefCell<AssetsLibrary>>,
}

impl SimpleColoredPass {
    pub fn new(framework: &crate::Framework, asset_library: Rc<RefCell<AssetsLibrary>>) -> Self {
        let module = framework
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/simple_colored.wgsl"));

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
                    label: Some("Simple colored pipeline"),
                    layout: Some(&render_pipeline_layout),
                    depth_stencil: None,
                    vertex: VertexState {
                        module: &module,
                        entry_point: "vs",
                        buffers: &[Mesh::layout(), DebugInstance2D::layout()],
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
        Self {
            pipeline: simple_diffuse_pipeline,
            asset_library,
        }
    }
}

impl RenderPass for SimpleColoredPass {
    fn execute_with_renderpass<'s, 'call, 'pass>(
        &'s self,
        mut pass: wgpu::RenderPass<'pass>,
        items: &'call [(u32, &'pass dyn PassBindble)],
    ) where
        'pass: 'call,
        's: 'pass,
    {
        pass.set_pipeline(&self.pipeline);
        self.bind_all(&mut pass, items);
    }
}
