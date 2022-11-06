//@include :common_definitions
//@include :2d_definitions

@group(2) @binding(0) var diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

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
    let checkerboard = checkerboard(i32(midpoint.x + midpoint.y), i_x_span);

    return mix(vec4(0.0), vec4(0.0, 0.0, 0.0, 1.0), checkerboard * point_inside_border);
}