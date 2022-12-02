//@include :common_definitions
//@include :2d_definitions

@group(2) @binding(0) var diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

fn stencil_sample(_uv: vec2<f32>) -> f32 {

    let uv = vec2(_uv.x, 1.0 - _uv.y);

    let stencil_dimensions = textureDimensions(diffuse);
    let one_over_dimensions = vec2(1.0, 1.0) / vec2(f32(stencil_dimensions.x), f32(stencil_dimensions.y));
    
    let up_sample = textureSample(diffuse, s_diffuse, uv + vec2<f32>(0.0, 1.0) * one_over_dimensions);
    let down_sample = textureSample(diffuse, s_diffuse,  uv + vec2<f32>(0.0, -1.0) * one_over_dimensions);
    let left_sample = textureSample(diffuse, s_diffuse,  uv + vec2<f32>(-1.0, 0.0) * one_over_dimensions);
    let right_sample = textureSample(diffuse, s_diffuse,  uv + vec2<f32>(1.0, 0.0) * one_over_dimensions);
    let up_down = down_sample + up_sample;
    let left_right = right_sample + left_sample;
    let sampled_component = up_down - left_right;
    
    return f32(sampled_component.a);
}

fn stripes(step: f32, size: f32) -> f32 {
    return f32(fract(step * size) < 0.5);
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let animation_speed = 5.0;
    let time_step = in.time * 0.03;
    
    let size = 35.0;
    let ss = stencil_sample(in.tex_uv);
    let st = stripes(in.tex_uv.x + in.tex_uv.y + time_step, size);
    let o = ss * st;
    return vec4(0.0, 0.0, 0.0, o);
}