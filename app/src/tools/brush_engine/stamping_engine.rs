use cgmath::vec2;
use framework::TypedBufferConfiguration;
use framework::{Framework, MeshInstance2D, TypedBuffer};
use image_editor::layers::{BitmapLayer, LayerType};
use scene::{Camera2d, Camera2dUniformBlock};
use wgpu::{BindGroup, RenderPassColorAttachment, RenderPassDescriptor};

use crate::{StrokeContext, StrokePath};

use super::stamping_engine_pass::StampingEngineRenderPass;
use super::BrushEngine;

pub struct Stamp {
    pub(crate) brush_texture: BitmapLayer,
}

pub struct StampCreationInfo<'framework> {
    pub camera_buffer: &'framework TypedBuffer<'framework>,
}

impl Stamp {
    pub fn new(brush_texture: BitmapLayer) -> Self {
        Self { brush_texture }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StampConfiguration {
    pub color_srgb: [u8; 3],
    pub opacity: u8,
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
        let color = [
            cfg.color_srgb[0] as f32 / 255.0,
            cfg.color_srgb[1] as f32 / 255.0,
            cfg.color_srgb[2] as f32 / 255.0,
            cfg.opacity as f32 / 255.0,
        ];
        Self {
            color,
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
    camera_buffer: TypedBuffer<'framework>,
    camera_bind_group: BindGroup,
}

impl<'framework> StrokingEngine<'framework> {
    pub fn new(initial_stamp: Stamp, framework: &'framework Framework) -> Self {
        let stamp_pass = StampingEngineRenderPass::new(framework);

        let camera_buffer =
            framework.allocate_typed_buffer(TypedBufferConfiguration::<Camera2dUniformBlock> {
                initial_setup: framework::typed_buffer::BufferInitialSetup::Size(
                    std::mem::size_of::<Camera2dUniformBlock>() as u64,
                ),
                buffer_type: framework::BufferType::Uniform,
                allow_write: true,
                allow_read: false,
            });
        let camera_bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture2D Camera Layout"),
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
        let camera_bind_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Texture2D Camera"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        camera_buffer.inner_buffer().as_entire_buffer_binding(),
                    ),
                }],
            });

        Self {
            stamps: vec![initial_stamp],
            current_stamp: 0,
            stamp_pass,
            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn create_stamp(&self, brush_texture: BitmapLayer) -> Stamp {
        Stamp::new(brush_texture)
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
        match context.editor.document().current_layer().layer_type {
            // TODO: Deal with difference between current_layer and buffer_layer size
            LayerType::Bitmap(_) => {
                let buffer_layer = context.editor.document().buffer_layer();

                // 1. Update buffer
                let instances: Vec<MeshInstance2D> = path
                    .points
                    .iter()
                    .map(|pt| {
                        MeshInstance2D::new(pt.position, vec2(pt.size, pt.size), 0.0, true, 1.0)
                    })
                    .collect();

                let bm_camera = Camera2d::new(
                    -0.1,
                    1000.0,
                    [
                        -buffer_layer.size().x as f32 * 0.5,
                        buffer_layer.size().x as f32 * 0.5,
                        buffer_layer.size().y as f32 * 0.5,
                        -buffer_layer.size().y as f32 * 0.5,
                    ],
                );
                self.camera_buffer
                    .write_sync::<Camera2dUniformBlock>(&vec![(&bm_camera).into()]);
                self.stamp_pass.update(instances);
                // 2. Do draw
                let bitmap_texture = context.editor.framework().texture2d(buffer_layer.texture());
                let stroking_engine_render_pass = RenderPassDescriptor {
                    label: Some("Stamping Engine render pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: bitmap_texture.texture_view(),
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
                    buffer_layer.size().x,
                    buffer_layer.size().y,
                    0.0,
                    1.0,
                );
                let stamp = self.current_stamp().brush_texture.texture();
                let stamp = context.editor.framework().texture2d(stamp);
                self.stamp_pass.execute(
                    render_pass,
                    &stamp,
                    &context.editor.framework().asset_library,
                    &self.camera_bind_group,
                );
            }
        }
    }
}
