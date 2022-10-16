//@include :common_definitions
//@include :2d_definitions
//@include :blend_modes

struct BlendSettings {
    blend_mode: i32,
    // x, y, w, h
    source_rect: vec4<f32>,
    dest_rect: vec4<f32>
}

@group(2) @binding(0) var top: texture_2d<f32>;
@group(2) @binding(1) var s_top: sampler;

@group(3) @binding(0) var bottom: texture_2d<f32>;
@group(3) @binding(1) var s_bottom: sampler;

@group(4) @binding(0) var<uniform> blend_settings: BlendSettings;

fn compute_dest_uv(settings: BlendeSettings, source_uv: vec2<f32>) -> vec2<f32> {
    let source_location = vec2<f32>(settings.source_rect.x, settings.source_rect.y);
    let source_size = vec2<f32>(settings.source_rect.w, settings.source_rect.z);
    let dest_location = vec2<f32>(settings.dest_rect.x, settings.dest_rect.y);
    let dest_size = vec2<f32>(settings.dest_rect.w, settings.dest_rect.z);
    
    let source_uv_to_fragment_pos = source_location + vec2<f32>(source_size.x * source_uv.x, source_size.y * source_uv.y);
    let uv_into_dest_rect = vec2<f32>(source_uv_to_fragment_pos.x / (dest_size.x + dest_location.x), source_uv_to_fragment_pos.x / (dest_size.y + dest_location.y));
    return uv_into_dest_rect;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let bottom_uv = compute_dest_uv(blend_settings, in.tex_uv);
    let top_sample = textureSample(top, s_top, in.tex_uv);
    let bottom_sample = textureSample(bottom, s_bottom, bottom_uv);
    let color = select_blend_mode(BLEND_NORMAL, bottom_sample, top_sample);
    return color.a * in.multiply_color;
}