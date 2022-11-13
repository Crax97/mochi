//@include :common_definitions
//@include :2d_definitions

@group(2) @binding(0) var diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

@group(3) @binding(0) var stencil: texture_2d<f32>;
@group(3) @binding(1) var s_stencil: sampler;

fn stencil_sample(uv: vec2<f32>) -> f32 {

    let stencil_dimensions = textureDimensions(stencil);
    let texel_size = vec2<f32>(1.0 / f32(stencil_dimensions.x), 1.0 / f32(stencil_dimensions.y));

    let up_sample = textureSample(stencil, s_stencil, uv + vec2<f32>(0.0, 1.0) * texel_size);
    let down_sample = textureSample(stencil, s_stencil, uv + vec2<f32>(0.0, -1.0) * texel_size);
    let left_sample = textureSample(stencil, s_stencil, uv + vec2<f32>(-1.0, 0.0) * texel_size);
    let right_sample = textureSample(stencil, s_stencil, uv + vec2<f32>(1.0, 0.0) * texel_size);
    let up_down = down_sample + up_sample;
    let left_right = right_sample + left_sample;
    let sampled_component = up_down - left_right;
    return textureSample(stencil, s_stencil, uv).r;
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
    let point_inside_border = 
        f32((midpoint.x < span)  || (midpoint.x >= scale.x * 2.0 - span))+
        f32((midpoint.y < span)  || (midpoint.y >= scale.y * 2.0 - span))
    ;
    let ss = stencil_sample(in.tex_uv);

    // return mix(vec4(0.0), vec4(1.0, 0.0, 0.0, 1.0), stencil_sample(in.tex_uv));
    return vec4<f32>(ss, ss, ss, ss);
}