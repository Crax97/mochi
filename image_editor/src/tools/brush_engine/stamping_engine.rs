use cgmath::{
    point2, point3, vec2, vec3, ElementWise, EuclideanSpace, Matrix4, Point2, Rad, Transform,
    Vector3,
};
use framework::{
    asset_library::{MeshNames, PipelineNames},
    Framework, MeshInstance2D, TypedBuffer, TypedBufferConfiguration,
};
use wgpu::{
    BindGroup, CommandEncoder, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    Texture, TextureView,
};

use crate::{
    layers::{BitmapLayer, Layer},
    ImageEditor, StrokeContext, StrokePath,
};

use super::BrushEngine;

pub struct Stamp {
    brush_texture: BitmapLayer,
    bind_group: BindGroup,
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
                label: Some("Layer render pass"),
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
        }
    }
}

pub struct StrokingEngine<'framework> {
    current_stamp: Stamp,
    instance_buffer: TypedBuffer<'framework>,
}

impl<'framework> StrokingEngine<'framework> {
    pub fn new(initial_stamp: Stamp, framework: &'framework Framework) -> Self {
        let instance_buffer = TypedBuffer::new(
            framework,
            TypedBufferConfiguration::<MeshInstance2D> {
                initial_data: vec![],
                buffer_type: framework::BufferType::Vertex,
                allow_write: true,
                allow_read: false,
            },
        );
        Self {
            current_stamp: initial_stamp,
            instance_buffer,
        }
    }
}

fn correct_point_for_stroke(
    point: Point2<f32>,
    editor: &ImageEditor,
    layer: &Layer,
) -> Point2<f32> {
    // When the user clicks on the canvas, the canvas is probably zoomed
    // but when the stroke points are rendered, the canvas is loaded without the zoom
    // so we have to "correct" the points position based on the zoom level.
    // This might change if we stroke by blitting the image

    match layer.layer_type {
        crate::layers::LayerType::Bitmap(ref bitmap_layer) => {
            let one_over_scale = 1.0 / editor.camera().current_scale();
            let actual_layer_scale =
                bitmap_layer.size().mul_element_wise(layer.scale) * one_over_scale;
            let layer_ratio = actual_layer_scale.div_element_wise(bitmap_layer.size());
            let lrp = point2(layer_ratio.x, layer_ratio.y);
            // println!("Actual layer scale {:?}, ratio is {:?}", actual_layer_scale, lrp);
            let point = point.div_element_wise(lrp);
            let camera_displace = editor
                .camera()
                .position()
                .mul_element_wise(point2(-1.0, 1.0 / lrp.y));
            point.add_element_wise(camera_displace)
        }
        _ => point,
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
                    .map(|pt| {
                        MeshInstance2D::new(
                            correct_point_for_stroke(*pt, context.editor, context.layer),
                            vec2(5.0, 5.0),
                            0.0,
                        )
                    })
                    .collect();

                self.instance_buffer.write_sync(&instances.as_slice());
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
                render_pass.set_pipeline(&context.assets.pipeline(PipelineNames::SIMPLE_TEXTURED));
                render_pass.set_bind_group(0, &self.current_stamp.bind_group, &[]);
                self.instance_buffer.bind(1, &mut render_pass);
                context
                    .assets
                    .mesh(MeshNames::QUAD)
                    .draw(&mut render_pass, instances.len() as u32);
            }
            _ => {}
        }
    }
}
