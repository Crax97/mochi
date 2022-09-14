use std::cell::RefCell;
use std::rc::Rc;

use cgmath::{point2, vec2, ElementWise, Point2};
use framework::render_pass::{PassBindble, RenderPass};
use framework::AssetsLibrary;
use framework::{
    asset_library::mesh_names, Framework, Mesh, MeshInstance2D, TypedBuffer,
    TypedBufferConfiguration,
};
use wgpu::{
    BindGroup, BindGroupLayout, BlendComponent, ColorTargetState, FragmentState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, VertexState,
};

use crate::{layers::BitmapLayer, StrokeContext, StrokePath};

use super::stamping_engine_pass::StampingEngineRenderPass;
use super::BrushEngine;

pub struct Stamp<'framework> {
    brush_texture: BitmapLayer<'framework>,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

pub struct StampCreationInfo<'framework> {
    pub camera_buffer: &'framework TypedBuffer<'framework>,
}

impl<'framework> Stamp<'framework> {
    pub fn new(
        brush_texture: BitmapLayer<'framework>,
        framework: &'framework Framework,
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
    stamps: Vec<Stamp<'framework>>,
    stamp_pass: StampingEngineRenderPass<'framework>,
}

impl<'framework> StrokingEngine<'framework> {
    pub fn new(
        initial_stamp: Stamp<'framework>,
        framework: &'framework Framework,
        assets: Rc<RefCell<AssetsLibrary>>,
    ) -> Self {
        let stamp_pass = StampingEngineRenderPass::new(framework, assets);
        Self {
            stamps: vec![initial_stamp],
            current_stamp: 0,
            stamp_pass,
        }
    }

    pub fn create_stamp(
        &self,
        brush_texture: BitmapLayer<'framework>,
        framework: &'framework Framework,
        info: StampCreationInfo,
    ) -> Stamp {
        Stamp::new(brush_texture, framework, info)
    }

    pub fn settings(&self) -> StampConfiguration {
        self.stamp_pass.get_stamp_settings()
    }

    pub fn set_new_settings(&mut self, settings: StampConfiguration) {
        self.stamp_pass.set_stamp_settings(settings);
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
                // 1. Update buffer
                let instances: Vec<MeshInstance2D> = path
                    .points
                    .iter()
                    .map(|pt| MeshInstance2D::new(pt.position, vec2(pt.size, pt.size), 0.0))
                    .collect();
                self.stamp_pass.update(instances);
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

                render_pass.set_viewport(
                    0.0,
                    0.0,
                    bitmap_layer.size().x,
                    bitmap_layer.size().y,
                    0.0,
                    1.0,
                );
                self.stamp_pass.execute_with_renderpass(
                    render_pass,
                    &[
                        (0, &self.current_stamp().bind_group),
                        (2, bitmap_layer.camera_bind_group()),
                    ],
                );
            }
        }
    }
}
