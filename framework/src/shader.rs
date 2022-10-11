use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, FragmentState, RenderPipeline, ShaderModule,
    TextureFormat, VertexBufferLayout, VertexState,
};

use crate::Framework;

pub trait ShaderLayout {
    fn layout() -> VertexBufferLayout<'static>;
}

pub enum BindElement {
    UniformBuffer,
    Texture,
}

pub struct ShaderCreationInfo<'a> {
    vertex_module: ShaderModule,
    fragment_module: ShaderModule,
    output_format: TextureFormat,
    bind_elements: Vec<BindElement>,
    blend_state: BlendState,
    layouts: Vec<VertexBufferLayout<'a>>,
}

pub struct Shader {
    pub(crate) render_pipeline: RenderPipeline,
}

impl Shader {
    pub(crate) fn new(framework: &Framework, info: ShaderCreationInfo) -> Self {
        let bind_group_layouts = Shader::bind_group_layouts_from_bind_elements(&info.bind_elements);
        let render_pipeline_layout =
            framework
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Shader pipeline layout"),
                    bind_group_layouts: bind_group_layouts.as_slice(),
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            framework
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("StampingEngine pipeline"),
                    layout: Some(&render_pipeline_layout),
                    depth_stencil: None,
                    vertex: VertexState {
                        module: &info.vertex_module,
                        entry_point: "vertex",
                        buffers: &info.layouts.as_slice(),
                    },
                    fragment: Some(FragmentState {
                        module: &info.fragment_module,
                        entry_point: "fragment",
                        targets: &[Some(ColorTargetState {
                            format: info.output_format,
                            blend: Some(info.blend_state),
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
        Self { render_pipeline }
    }

    fn bind_group_layouts_from_bind_elements(elements: &Vec<BindElement>) -> Vec<&BindGroupLayout> {
        todo!()
    }
}
