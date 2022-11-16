use cgmath::{Point3, Rad, Vector2};
use framework::framework::{BufferId, ShaderId};
use framework::renderer::draw_command::BindableResource;
use framework::{
    framework::TextureId,
    renderer::{
        draw_command::{DrawCommand, DrawMode, OptionalDrawData, PrimitiveType},
        renderer::Renderer,
    },
    Camera2d, Transform2d,
};
use framework::{Framework, RgbaTexture2D, Texture, TextureConfiguration, TextureUsage};

pub struct BitmapLayerConfiguration {
    pub width: u32,
    pub height: u32,
}
pub struct BitmapLayer {
    texture: TextureId,
    configuration: BitmapLayerConfiguration,
}

impl BitmapLayer {
    pub fn new(
        label: &str,
        background: [u8; 4],
        configuration: BitmapLayerConfiguration,
        framework: &mut Framework,
    ) -> Self {
        let bytes: Vec<u8> = (0..(configuration.width * configuration.height) * 4)
            .enumerate()
            .map(|(i, _)| background[i % 4])
            .collect();
        Self::new_from_bytes(label, &bytes, configuration, framework)
    }

    pub fn new_from_texture(label: &str, texture_id: TextureId, framework: &Framework) -> Self {
        let (width, height) = framework.texture2d_dimensions(&texture_id);
        Self {
            texture: texture_id,
            configuration: BitmapLayerConfiguration { width, height },
        }
    }

    pub fn new_from_bytes(
        label: &str,
        bytes: &[u8],
        configuration: BitmapLayerConfiguration,
        framework: &mut Framework,
    ) -> Self {
        let texture = framework.allocate_texture2d(
            RgbaTexture2D::from_bytes(bytes, (configuration.width, configuration.height)).unwrap(),
            TextureConfiguration {
                label: Some(label),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );

        Self {
            texture,
            configuration,
        }
    }

    pub fn texture(&self) -> &TextureId {
        &self.texture
    }

    pub fn size(&self) -> Vector2<f32> {
        Vector2 {
            x: self.configuration.width as f32,
            y: self.configuration.height as f32,
        }
    }

    pub(crate) fn replace_texture(&mut self, new_texture: TextureId) {
        self.texture = new_texture;
    }

    pub fn draw(
        &self,
        renderer: &mut Renderer,
        position: cgmath::Point2<f32>,
        scale: Vector2<f32>,
        rotation_radians: f32,
        opacity: f32,
    ) {
        let real_scale = Vector2 {
            x: scale.x * self.size().x * 0.5,
            y: scale.y * self.size().y * 0.5,
        };
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: self.texture().clone(),
                instances: vec![Transform2d {
                    position: Point3 {
                        x: position.x,
                        y: position.y,
                        z: 0.0,
                    },
                    scale: real_scale,
                    rotation_radians: Rad(rotation_radians),
                }],
                flip_uv_y: true,
                multiply_color: wgpu::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: opacity as f64,
                },
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData::default(),
        });
    }

    pub fn draw_blended(
        &self,
        renderer: &mut Renderer,
        shader_to_use: ShaderId,
        bottom_layer: TextureId,
        blend_settings_buffer: BufferId,
        output: &TextureId,
        framework: &mut Framework,
    ) {
        let real_scale = Vector2 {
            x: self.size().x * 0.5,
            y: self.size().y * 0.5,
        };
        renderer.begin(&self.camera(), None, framework);
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: self.texture().clone(),
                instances: vec![Transform2d {
                    position: Point3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    scale: real_scale,
                    rotation_radians: Rad(0.0),
                }],
                flip_uv_y: true,
                multiply_color: wgpu::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData {
                additional_vertex_buffers: vec![],
                additional_bindable_resource: vec![
                    BindableResource::Texture(bottom_layer),
                    BindableResource::UniformBuffer(blend_settings_buffer),
                ],
                shader: Some(shader_to_use),
            },
        });
        renderer.end(output, None, framework);
    }

    pub fn camera(&self) -> Camera2d {
        let half_w = self.size().x * 0.5;
        let half_h = self.size().y * 0.5;
        Camera2d::new(-0.01, 1000.0, [-half_w, half_w, half_h, -half_h])
    }
}
