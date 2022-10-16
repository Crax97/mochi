//@include :common_definitions
//@include :2d_definitions

@group(2) @binding(0) var diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    return textureSample(diffuse, s_diffuse, in.tex_uv) * in.multiply_color;
}