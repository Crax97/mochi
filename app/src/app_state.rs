use std::rc::Rc;

use cgmath::{point2, point3};
use framework::{Framework, Mesh, MeshConstructionDetails, Vertex};
use image_editor::{AssetsLibrary, ImageEditor};
use wgpu::{ColorTargetState, FragmentState, Surface, SurfaceConfiguration, VertexState};
use winit::{dpi::PhysicalSize, window::Window};

pub(crate) struct AppState<'framework> {
    pub(crate) framework: &'framework Framework,
    pub(crate) assets: Rc<AssetsLibrary>,
    pub(crate) window: Window,
    pub(crate) final_surface: Surface,
    pub(crate) final_surface_configuration: SurfaceConfiguration,
}
impl<'framework> AppState<'framework> {
    pub(crate) fn new(window: Window, framework: &'framework Framework) -> Self {
        let final_surface = unsafe { framework.instance.create_surface(&window) };
        let final_surface_configuration = SurfaceConfiguration {
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: final_surface.get_supported_formats(&framework.adapter)[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        final_surface.configure(&framework.device, &final_surface_configuration);

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
            .create_shader_module(wgpu::include_wgsl!("shaders/final_present.wgsl"));

        let bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Final render group layout"),
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
        let render_pipeline_layout =
            framework
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
        let final_present_pipeline =
            framework
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("final render shader"),
                    layout: Some(&render_pipeline_layout),
                    depth_stencil: None,
                    vertex: VertexState {
                        module: &module,
                        entry_point: "vs",
                        buffers: &[Mesh::layout()],
                    },
                    fragment: Some(FragmentState {
                        module: &module,
                        entry_point: "fs",
                        targets: &[Some(ColorTargetState {
                            format: final_surface_configuration.format,
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

        let assets = Rc::new(AssetsLibrary {
            quad_mesh,
            final_present_pipeline,
        });
        Self {
            window,
            assets: assets.clone(),
            framework,
            final_surface,
            final_surface_configuration,
        }
    }

    pub(crate) fn on_resized(
        &self,
        new_size: winit::dpi::PhysicalSize<u32>,
        image_editor: &mut ImageEditor,
    ) {
        let half_size = PhysicalSize {
            width: new_size.width as f32 * 0.5,
            height: new_size.height as f32 * 0.5,
        };
        let left_right_top_bottom = [
            -half_size.width,
            half_size.width,
            half_size.height,
            -half_size.height,
        ];
        let new_surface_configuration = SurfaceConfiguration {
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self
                .final_surface
                .get_supported_formats(&self.framework.adapter)[0],
            width: new_size.width,
            height: new_size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        self.final_surface
            .configure(&self.framework.device, &new_surface_configuration);
        image_editor.on_resize(left_right_top_bottom);
    }
}
