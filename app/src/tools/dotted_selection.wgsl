//@include :common_definitions
//@include :2d_definitions

@group(2) @binding(0) var diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

// todo pass rect size

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let x = in.tex_uv.x;
    let y = in.tex_uv.y;
    let span = 0.01;
    if ( (x < span)  || (y < span) || (x > 1.0 - span) || (y > 1.0 - span)) {
        if (i32(x * 101.0) % 2 == 0 && i32(y * 101.0) % 2 == 0) {
            return vec4(0.0, 0.0, 0.0, 1.0);
        } else {
            return vec4(0.0, 0.0, 0.0, 0.0);
        }
    } else {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }
}