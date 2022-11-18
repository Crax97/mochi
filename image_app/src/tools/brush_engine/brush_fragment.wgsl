//@include :common_definitions
//@include :2d_definitions
//@include :blend_modes

@group(2) @binding(0) var diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

struct BrushSettings {
    smoothness: f32,
}

@group(3) @binding(0) var<uniform> brush_settings: BrushSettings;

fn smoothness(uv: vec2<f32>, theta: f32) -> f32 {
    let x = distance(vec2<f32>(0.5), uv);

    let r = 0.5 - 0.5 * x;
    let b = 1.0 - ((x * (2.0 - 2.0 * r) + x * x * (2.0 * r - 1.0)));
    return pow(b, theta);
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let s = smoothness(in.tex_uv, brush_settings.smoothness);
    return textureSample(diffuse, s_diffuse, in.tex_uv).a * in.multiply_color * s;
}