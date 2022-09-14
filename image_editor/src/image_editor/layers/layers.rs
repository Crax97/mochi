use std::borrow::Borrow;
use std::cell::Ref;

use cgmath::{Point2, Vector2};
use framework::render_pass::{PassBindble, RenderPass};
use framework::{Framework, MeshInstance2D, TypedBuffer, TypedBufferConfiguration};
use wgpu::BindGroup;

use framework::{asset_library::AssetsLibrary, mesh_names};

use super::layer_draw_pass::LayerDrawPass;
use super::{bitmap_layer, BitmapLayer};

pub struct Layer<'framework> {
    pub name: String,
    pub layer_type: LayerType,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,

    pub instance_buffer: TypedBuffer<'framework>,
    pub(crate) bind_group: BindGroup,
}

pub struct LayerCreationInfo<'framework> {
    pub name: String,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,
    pub camera_buffer: &'framework TypedBuffer<'framework>,
}

pub enum LayerType {
    Bitmap(bitmap_layer::BitmapLayer),
}

pub(crate) struct LayerDrawContext<'context, 'library, 'pass> {
    pub render_pass: wgpu::RenderPass<'pass>,
    pub draw_pass: &'context LayerDrawPass,
    pub assets: Ref<'library, AssetsLibrary>,
}

impl<'framework> Layer<'framework> {
    pub fn new_bitmap(
        bitmap_layer: BitmapLayer,
        creation_info: LayerCreationInfo<'_>,
        framework: &'framework Framework,
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
                label: Some("Layer render pass"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(bitmap_layer.texture_view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(bitmap_layer.sampler()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(
                            creation_info.camera_buffer.binding_resource(),
                        ),
                    },
                ],
            });

        let instance_buffer = framework.allocate_typed_buffer(TypedBufferConfiguration {
            initial_setup: framework::typed_buffer::BufferInitialSetup::Data(
                &Vec::<MeshInstance2D>::new(),
            ),
            buffer_type: framework::BufferType::Vertex,
            allow_write: true,
            allow_read: false,
        });
        Self {
            name: creation_info.name,
            layer_type: LayerType::Bitmap(bitmap_layer),
            position: creation_info.position,
            scale: creation_info.scale,
            instance_buffer,
            rotation_radians: creation_info.rotation_radians,
            bind_group,
        }
    }
    pub(crate) fn update(&mut self) {
        match &self.layer_type {
            LayerType::Bitmap(bitmap_layer) => {
                let real_scale = Vector2 {
                    x: self.scale.x * bitmap_layer.size().x * 0.5,
                    y: self.scale.y * bitmap_layer.size().y * 0.5,
                };
                self.instance_buffer.write_sync(&vec![MeshInstance2D::new(
                    self.position.clone(),
                    real_scale,
                    self.rotation_radians,
                )]);
            }
        }
    }
    pub(crate) fn draw<'context, 'library, 'pass, 'l>(
        &'l self,
        draw_context: LayerDrawContext<'context, 'library, 'pass>,
    ) where
        'framework: 'pass,
        'l: 'pass,
        'context: 'pass,
    {
        match &self.layer_type {
            LayerType::Bitmap(_) => {
                draw_context.draw_pass.execute_with_renderpass(
                    draw_context.render_pass,
                    &[(1, &self.instance_buffer), (0, &self.bind_group)],
                );
            }
        }
    }
    pub(crate) fn size(&self) -> Vector2<f32> {
        match self.layer_type {
            LayerType::Bitmap(ref bm) => bm.size(),
        }
    }
}
