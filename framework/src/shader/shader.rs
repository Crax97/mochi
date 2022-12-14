use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BlendState, ColorTargetState, DepthStencilState,
    FragmentState, RenderPipeline, ShaderModule, ShaderModuleDescriptor, TextureFormat,
    VertexBufferLayout, VertexState,
};

use crate::{Buffer, Framework, Mesh, MeshInstance2D};

pub trait ShaderLayout {
    fn layout() -> VertexBufferLayout<'static>;
}

pub enum BindElement {
    UniformBuffer,
    Texture,
    DepthTexture,
    StencilTexture,
    None,
}

pub struct ShaderCreationInfo<'a> {
    vertex_module: ShaderModule,
    fragment_module: ShaderModule,
    output_format: Option<TextureFormat>,
    bind_elements: Vec<BindElement>,
    blend_state: Option<BlendState>,
    depth_state: Option<DepthStencilState>,
    layouts: Vec<VertexBufferLayout<'a>>,
}

impl<'a> ShaderCreationInfo<'a> {
    pub fn using_default_vertex_instanced(
        fragment: ShaderModuleDescriptor,
        framework: &Framework,
    ) -> Self {
        let default_vertex_instanced = framework.shader_compiler.compile_into_shader_description(
            "Default Instanced Vertex Shader",
            include_str!("default_shaders/default_vertex_instanced.wgsl"),
        );
        let default_vertex_instanced = framework
            .device
            .create_shader_module(default_vertex_instanced);

        let fragment_module = framework.device.create_shader_module(fragment);
        Self {
            vertex_module: default_vertex_instanced,
            fragment_module,
            output_format: None,
            bind_elements: vec![],
            blend_state: None,
            depth_state: None,
            layouts: vec![],
        }
        .with_layout::<Mesh>()
        .with_layout::<MeshInstance2D>()
        .with_bind_element(BindElement::UniformBuffer) // 0 camera info buffer
        .with_bind_element(BindElement::None) // 1 is unused, for compat with default fragment shader
    }

    pub fn using_default_vertex(fragment: ShaderModuleDescriptor, framework: &Framework) -> Self {
        let default_vertex = framework.shader_compiler.compile_into_shader_description(
            "Default Vertex Shader",
            include_str!("default_shaders/default_vertex.wgsl"),
        );
        let default_vertex = framework.device.create_shader_module(default_vertex);
        let fragment_module = framework.device.create_shader_module(fragment);
        Self {
            vertex_module: default_vertex,
            fragment_module,
            output_format: None,
            bind_elements: vec![],
            blend_state: None,
            depth_state: None,
            layouts: vec![],
        }
        .with_layout::<Mesh>()
        .with_bind_element(BindElement::UniformBuffer) // 0 mesh info buffer
        .with_bind_element(BindElement::UniformBuffer) // 1 camera info buffer
    }
    pub fn using_default_vertex_fragment(framework: &Framework) -> Self {
        let default_fragment = framework.shader_compiler.compile_into_shader_description(
            "Default Fragment Shader",
            include_str!("default_shaders/default_fragment.wgsl"),
        );
        ShaderCreationInfo::using_default_vertex(default_fragment, framework)
            .with_bind_element(BindElement::Texture) // 2: diffuse texture + sampler
    }
    pub fn using_default_vertex_fragment_instanced(framework: &Framework) -> Self {
        let default_fragment = framework.shader_compiler.compile_into_shader_description(
            "Default Fragment Shader",
            include_str!("default_shaders/default_fragment.wgsl"),
        );
        ShaderCreationInfo::using_default_vertex_instanced(default_fragment, framework)
            .with_bind_element(BindElement::Texture) // 2: texture + sampler
    }

    pub fn with_bind_element(mut self, element: BindElement) -> Self {
        self.bind_elements.push(element);
        self
    }
    pub fn with_layout<T: ShaderLayout>(mut self) -> Self {
        self.layouts.push(T::layout());
        self
    }
    pub fn with_blend_state(mut self, blend_state: BlendState) -> Self {
        self.blend_state = Some(blend_state);
        self
    }
    pub fn with_output_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.output_format = Some(format);
        self
    }
    pub fn with_depth_state(mut self, depth_state: Option<DepthStencilState>) -> Self {
        self.depth_state = depth_state;
        self
    }
}

pub struct Shader {
    pub(crate) render_pipeline: RenderPipeline,
}

impl Shader {
    pub fn reserved_buffer_count() -> u32 {
        3
    }
    pub(crate) fn new(framework: &Framework, info: ShaderCreationInfo) -> Self {
        let bind_group_layouts =
            Shader::bind_group_layouts_from_bind_elements(framework, &info.bind_elements);
        let bind_group_layouts: Vec<&BindGroupLayout> =
            bind_group_layouts.iter().map(|g| g).collect();
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
                    label: Some("Shader pipeline"),
                    layout: Some(&render_pipeline_layout),
                    depth_stencil: info.depth_state,
                    vertex: VertexState {
                        module: &info.vertex_module,
                        entry_point: "vertex",
                        buffers: &info.layouts.as_slice(),
                    },
                    fragment: Some(FragmentState {
                        module: &info.fragment_module,
                        entry_point: "fragment",
                        targets: &[Some(ColorTargetState {
                            format: info.output_format.unwrap_or(TextureFormat::Rgba8UnormSrgb),
                            blend: Some(
                                info.blend_state
                                    .unwrap_or(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                            ),
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

    fn bind_group_layouts_from_bind_elements(
        framework: &Framework,
        elements: &Vec<BindElement>,
    ) -> Vec<BindGroupLayout> {
        elements
            .iter()
            .map(|e| match e {
                BindElement::UniformBuffer => Buffer::bind_group_layout(framework),
                BindElement::Texture => crate::texture2d_bind_group_layout(framework),
                BindElement::None => {
                    framework
                        .device
                        .create_bind_group_layout(&BindGroupLayoutDescriptor {
                            label: None,
                            entries: &[],
                        })
                }
                BindElement::DepthTexture => {
                    framework
                        .device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: Some("Depth Texture Bindg layout"),
                            entries: &[
                                wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                    ty: wgpu::BindingType::Texture {
                                        sample_type: wgpu::TextureSampleType::Depth,
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                        multisampled: false,
                                    },
                                    count: None,
                                },
                                wgpu::BindGroupLayoutEntry {
                                    binding: 1,
                                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                    ty: wgpu::BindingType::Sampler(
                                        wgpu::SamplerBindingType::Filtering,
                                    ),
                                    count: None,
                                },
                            ],
                        })
                }
                BindElement::StencilTexture => {
                    framework
                        .device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: Some("Stencil Texture Bindg layout"),
                            entries: &[wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Uint,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            }],
                        })
                }
            })
            .collect()
    }
}
