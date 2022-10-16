use cgmath::Vector4;

pub struct BlendSettings {
    blend_mode: i32,
    // x, y, w, h
    source_rect: Vector4<f32>,
    dest_rect: Vector4<f32>
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BlendSettingsUniform {
    blend_mode: i32, padding: [f32; 3],
    // x, y, w, h
    source_rect: Vector4<f32>,
    dest_rect: Vector4<f32>
}

unsafe impl bytemuck::Zeroable for BlendSettingsUniform {}
unsafe impl bytemuck::Pod for BlendSettingsUniform { }

impl From<BlendSettings> for BlendSettingsUniform {
    fn from(settings: BlendSettings) -> Self {
        Self {
            blend_mode: settings.blend_mode,
            padding: [0.0; 3],
            source_rect: settings.source_rect,
            dest_rect: settings.dest_rect,
        }
    }
}