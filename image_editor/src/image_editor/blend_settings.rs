#[derive(Clone, Copy, PartialEq, Eq, Default, Hash, Debug)]
pub enum BlendMode {
    #[default]
    Normal = 0,
    Multiply = 1,
    Screen = 2,
}

impl BlendMode {
    fn as_i32(&self) -> i32 {
        *self as i32
    }
}

pub struct BlendSettings {
    pub blend_mode: BlendMode,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BlendSettingsUniform {
    blend_mode: i32,
    padding: [f32; 3],
}

unsafe impl bytemuck::Zeroable for BlendSettingsUniform {}
unsafe impl bytemuck::Pod for BlendSettingsUniform {}

impl From<BlendSettings> for BlendSettingsUniform {
    fn from(settings: BlendSettings) -> Self {
        Self {
            blend_mode: settings.blend_mode.as_i32(),
            padding: [0.0; 3],
        }
    }
}
