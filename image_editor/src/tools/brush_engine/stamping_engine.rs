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
    stamps: Vec<Stamp>,
    stamp_pass: StampingEngineRenderPass<'framework>,
}

impl<'framework> StrokingEngine<'framework> {
    pub fn new(
        initial_stamp: Stamp,
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
        brush_texture: BitmapLayer,
        framework: &Framework,
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
                let instance_len = instances.len();
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
                let render_pass = context
                    .command_encoder
                    .begin_render_pass(&stroking_engine_render_pass);

                self.stamp_pass
                    .execute_with_renderpass(render_pass, &[(0, &self.current_stamp().bind_group)]);
            }
        }
    }
}
