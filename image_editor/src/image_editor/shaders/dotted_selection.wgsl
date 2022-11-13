//@include :common_definitions
//@include :2d_definitions

@group(2) @binding(0) var diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

@group(3) @binding(0) var stencil: texture_2d<u32>;
@group(3) @binding(1) var s_stencil: sampler;

fn stencil_sample(uv: vec2<f32>) -> f32 {

    let stencil_dimensions = textureDimensions(stencil);
    let x = uv.x * f32(stencil_dimensions.x);
    let y = (1.0 - uv.y) * f32(stencil_dimensions.y);
    let texel_position = vec2(i32(x), i32(y));

    let up_sample = textureLoad(stencil, texel_position + vec2<i32>(0, 1), 0);
    let down_sample = textureLoad(stencil,  texel_position + vec2<i32>(0, -1), 0);
    let left_sample = textureLoad(stencil,  texel_position + vec2<i32>(-1, 0), 0);
    let right_sample = textureLoad(stencil,  texel_position + vec2<i32>(1, 0), 0);
    let up_down = down_sample + up_sample;
    let left_right = right_sample + left_sample;
    let sampled_component = up_down - left_right;
    
    return f32(sampled_component.r);
}

fn checkerboard(el: i32, span: i32) -> f32 {
    let el = abs(el);
    return f32((el % (span * 2)) < span);
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let position = vec2<f32>(in.position.x, in.position.y);
    let scale = vec2<f32>(in.scale.x, in.scale.y);
    let top_left = position - scale;
    let bottom_right = position + scale;
    let x = in.tex_uv.x;
    let y = in.tex_uv.y;
    let midpoint = scale * 2.0 * vec2<f32>(x, y);

    let span = 2.0;
    let i_x_span = 15;
    let i_y_span = 15;
    let ss = stencil_sample(in.tex_uv);
    let checker = checkerboard(i32(midpoint.x + midpoint.y), i_x_span);
    return mix(vec4(0.0), vec4(0.0, 0.0, 0.0, 1.0), checker * ss);
}