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

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let top_sample = textureSample(top, s_top, in.tex_uv);
    let bottom_sample = textureSample(bottom, s_bottom, in.tex_uv);
    let blend = select_blend_mode(blend_settings.blend_mode, bottom_sample, top_sample);
    let color = mix(bottom_sample, blend, top_sample.a);
    return color * in.multiply_color;
}