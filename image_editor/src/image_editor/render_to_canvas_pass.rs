use std::{cell::RefCell, rc::Rc};

use framework::{
    mesh_names,
    render_pass::{self, PassBindble},
    AssetsLibrary, Framework, Mesh, MeshInstance2D, TypedBuffer, TypedBufferConfiguration,
};
use wgpu::{
    BindGroup, ColorTargetState, FragmentState, RenderPipeline, SurfaceConfiguration, VertexState,
};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FinalPassUniform {
    size: [f32; 2],
}

pub struct RenderToCanvasPass {
    pipeline: RenderPipeline,
    assets: Rc<RefCell<AssetsLibrary>>,
}

impl RenderToCanvasPass {
    pub fn new(
        framework: &Framework,
        target_format: wgpu::TextureFormat,
        assets: Rc<RefCell<AssetsLibrary>>,
    ) -> Self {
        let module = framework
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/render_to_canvas.wgsl"));

        let bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Final render group layout"),
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
        let camera_bind_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Final render group layout"),
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
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout, &camera_bind_layout],
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
                        buffers: &[Mesh::layout(), MeshInstance2D::layout()],
                    },
                    fragment: Some(FragmentState {
                        module: &module,
                        entry_point: "fs",
                        targets: &[Some(ColorTargetState {
                            format: target_format,
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
            pipeline: final_present_pipeline,

            assets,
        }
    }
}

impl render_pass::RenderPass for RenderToCanvasPass {
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
        self.assets.borrow().mesh(mesh_names::QUAD).draw(pass, 1);
    }
}
