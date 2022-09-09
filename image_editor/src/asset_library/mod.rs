use std::{collections::HashMap, iter::FromIterator};

use cgmath::{point2, point3};
use framework::{Framework, Mesh, MeshConstructionDetails, MeshInstance2D, Vertex};
use wgpu::{ColorTargetState, FragmentState, RenderPipeline, VertexState};

pub struct AssetsLibrary {
    pipelines: HashMap<String, RenderPipeline>,
    meshes: HashMap<String, Mesh>,
}

impl AssetsLibrary {
    pub fn new(framework: &'_ Framework) -> Self {
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
        let quad_mesh = Mesh::new(
            &framework,
            MeshConstructionDetails {
                vertices: quad_mesh_vertices,
                indices,
                allow_editing: false,
            },
        );
        let module = framework
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/simple_shader.wgsl"));

        let bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Simple shader layout"),
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
                    label: Some("Simple render pipeline"),
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

        let module = framework
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/simple_colored.wgsl"));

        let simple_colored_pipeline =
            framework
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Simple render pipeline"),
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
            pipelines: HashMap::from_iter(
                [
                    (
                        PipelineNames::SIMPLE_TEXTURED.to_owned(),
                        simple_diffuse_pipeline,
                    ),
                    (
                        PipelineNames::SIMPLE_COLORED.to_owned(),
                        simple_colored_pipeline,
                    ),
                ]
                .into_iter(),
            ),
            meshes: HashMap::from_iter(std::iter::once((MeshNames::QUAD.to_owned(), quad_mesh))),
        }
    }
    pub fn add_pipeline(&mut self, name: &str, pipeline: RenderPipeline) {
        self.pipelines.insert(name.to_owned(), pipeline);
    }
    pub fn add_mesh(&mut self, name: &str, mesh: Mesh) {
        self.meshes.insert(name.to_owned(), mesh);
    }
}

impl<'assetlib> AssetsLibrary {
    pub fn pipeline(&'assetlib self, name: &str) -> &'assetlib RenderPipeline {
        self.pipelines
            .get(name)
            .expect("This pipeline doesn't exist")
    }
    pub fn mesh(&'assetlib self, name: &str) -> &'assetlib Mesh {
        self.meshes.get(name).expect("This mesh doesn't exist")
    }
}

pub mod PipelineNames {
    pub const SIMPLE_TEXTURED: &'static str = "SIMPLE_TEXTURED";
    pub const SIMPLE_COLORED: &'static str = "SIMPLE_COLORED";
}

pub mod MeshNames {
    pub const QUAD: &'static str = "QUAD";
}
