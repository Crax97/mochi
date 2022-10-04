use std::{cell::RefCell, rc::Rc};

use framework::{
    mesh_names,
    render_pass::{self, PassBindble},
    AssetsLibrary, Framework, Mesh, TypedBuffer, TypedBufferConfiguration,
};
use image_editor::layers::BitmapLayer;
use wgpu::{
    BindGroup, ColorTargetState, FragmentState, RenderPipeline, SurfaceConfiguration, VertexState,
};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FinalPassUniform {
    size: [f32; 2],
}

pub struct FinalRenderPass<'framework> {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    size_group: BindGroup,
    assets: Rc<RefCell<AssetsLibrary>>,
    final_pass_uniform_buffer: TypedBuffer<'framework>,
}

impl<'framework> FinalRenderPass<'framework> {
    pub fn new(
        framework: &'framework Framework,
        final_surface_configuration: SurfaceConfiguration,
        final_render: &BitmapLayer,
        assets: Rc<RefCell<AssetsLibrary>>,
    ) -> Self {
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
        let size_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Final render group layout"),
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
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout, &size_group_layout],
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

        let bind_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Final Draw render pass"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            final_render.texture().texture_view(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(final_render.texture().sampler()),
                    },
                ],
            });

        let uniform_buffer = TypedBuffer::new(
            framework,
            TypedBufferConfiguration::<FinalPassUniform> {
                initial_setup: framework::typed_buffer::BufferInitialSetup::Data(&vec![
                    FinalPassUniform { size: [1.0, 1.0] },
                ]),
                buffer_type: framework::BufferType::Uniform,
                allow_write: true,
                allow_read: false,
            },
        );

        let size_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Final Draw render pass"),
                layout: &&size_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.binding_resource()),
                }],
            });

        Self {
            pipeline: final_present_pipeline,
            bind_group,
            size_group,
            assets,
            final_pass_uniform_buffer: uniform_buffer,
        }
    }

    pub fn update_size(&mut self, new_size: [f32; 2]) {
        self.final_pass_uniform_buffer
            .write_sync(&vec![FinalPassUniform { size: new_size }])
    }
}

impl<'f> render_pass::RenderPass for FinalRenderPass<'f> {
    fn execute_with_renderpass<'s, 'call, 'pass>(
        &'s self,
        mut pass: wgpu::RenderPass<'pass>,
        _items: &'call [(u32, &'pass dyn PassBindble)],
    ) where
        'pass: 'call,
        's: 'pass,
    {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_bind_group(1, &self.size_group, &[]);
        self.assets.borrow().mesh(mesh_names::QUAD).draw(pass, 1);
    }
}
