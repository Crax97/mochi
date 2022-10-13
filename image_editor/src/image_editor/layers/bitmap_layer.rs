use cgmath::{num_traits::ToPrimitive, point2, point3, Point3, Rad, Vector2};
use framework::{
    framework::TextureId,
    renderer::{
        draw_command::{DrawCommand, DrawMode, OptionalDrawData, PrimitiveType},
        renderer::Renderer,
    },
    Camera2d, Framework, MeshInstance2D, Texture2dConfiguration, Transform2d,
};

pub struct BitmapLayerConfiguration {
    pub label: String,
    pub width: u32,
    pub initial_background_color: [f32; 4],
    pub height: u32,
}
pub struct BitmapLayer {
    texture: TextureId,
    configuration: BitmapLayerConfiguration,
}

impl BitmapLayer {
    pub fn new(framework: &Framework, configuration: BitmapLayerConfiguration) -> Self {
        let bytes: Vec<u32> = (1..(configuration.width * configuration.height) + 1)
            .map(|_| {
                let bg = configuration.initial_background_color;
                let r = (bg[0].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                let g = (bg[1].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                let b = (bg[2].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                let a = (bg[3].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                u32::from_le_bytes([r, g, b, a])
            })
            .collect();
        let bytes = bytemuck::cast_slice(&bytes);
        Self::new_from_bytes(framework, &bytes, configuration)
    }

    pub fn new_from_bytes(
        framework: &Framework,
        bytes: &[u8],
        configuration: BitmapLayerConfiguration,
    ) -> Self {
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let texture = framework.allocate_texture2d(
            Texture2dConfiguration {
                debug_name: Some(configuration.label.clone() + " Texture"),
                width: configuration.width,
                height: configuration.height,
                format,
                allow_cpu_write: true,
                allow_cpu_read: true,
                allow_use_as_render_target: true,
            },
            Some(bytes),
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

    pub fn replace_texture(&mut self, new_texture: TextureId) {
        self.texture = new_texture
    }

    pub fn draw(
        &self,
        renderer: &mut Renderer,
        position: cgmath::Point2<f32>,
        scale: Vector2<f32>,
        rotation_radians: f32,
        opacity: f32,
        output: &TextureId,
    ) {
        let real_scale = Vector2 {
            x: scale.x * self.size().x * 0.5,
            y: scale.y * self.size().y * 0.5,
        };
        renderer.begin(&self.camera(), None);
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
                flip_uv_y: false,
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData::default(),
        });
        renderer.end_on_texture(output);
    }

    pub fn camera(&self) -> Camera2d {
        let half_w = self.size().x * 0.5;
        let half_h = self.size().y * 0.5;
        Camera2d::new(-0.01, 1000.0, [-half_w, half_w, half_h, -half_h])
    }
}
