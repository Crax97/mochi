use strum_macros::EnumIter;

#[derive(Clone, Copy, PartialEq, Eq, Default, Hash, Debug, EnumIter)]
pub enum BlendMode {
    #[default]
    Normal = 0,
    Multiply = 1,
    Screen = 2,
    Overlay = 3,
    SoftLight = 4,
    ColorDodge = 5,
    ColorBurn = 6,
    Add = 7,
    Divide = 8,
    Subtract = 9,
    Difference = 10,
    Darken = 11,
    Lighten = 12,
}

impl std::fmt::Display for BlendMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pretty_text = match self {
            BlendMode::Normal => "Normal",
            BlendMode::Multiply => "Multiply",
            BlendMode::Screen => "Screen",
            BlendMode::Overlay => "Overlay",
            BlendMode::SoftLight => "Soft Light",
            BlendMode::ColorDodge => "Color Dodge",
            BlendMode::ColorBurn => "Color Burn",
            BlendMode::Add => "Add",
            BlendMode::Divide => "Divide",
            BlendMode::Subtract => "Subtract",
            BlendMode::Difference => "Difference",
            BlendMode::Darken => "Darken",
            BlendMode::Lighten => "Lighten",
        };
        f.write_str(pretty_text)
    }
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
