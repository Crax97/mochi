//@include :common_definitions
//@include :2d_definitions
//@include :blend_modes

struct BlendSettings {
    blend_mode: i32,
}

@group(2) @binding(0) var top: texture_2d<f32>;
@group(2) @binding(1) var s_top: sampler;

@group(3) @binding(0) var bottom: texture_2d<f32>;
@group(3) @binding(1) var s_bottom: sampler;

@group(4) @binding(0) var<uniform> blend_settings: BlendSettings;

fn over(a: vec3<f32>, b: vec3<f32>, alpha_a: f32, alpha_b: f32) -> vec4<f32> {
    // a over b: a is on top
    let a_o = alpha_a + alpha_b * (1.0 - alpha_a);
    let color = (a * alpha_a + b * alpha_b * (1.0 - alpha_a)) / a_o;
    return vec4<f32>(color.r, color.g, color.b, a_o);
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let top_sample = textureSample(top, s_top, in.tex_uv);
    let bottom_sample = textureSample(bottom, s_bottom, in.tex_uv);

    let top_rgb = vec3<f32>(top_sample.r, top_sample.g, top_sample.b);
    let bottom_rgb = vec3<f32>(bottom_sample.r, bottom_sample.g, bottom_sample.b);
    let blend = select_blend_mode(blend_settings.blend_mode, bottom_rgb, top_rgb);
    return over(blend, bottom_rgb, top_sample.a, bottom_sample.a);
}