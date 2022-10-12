// TODO: Add some kind of preprocessor for this stuff
// Something like @include relative_path

struct VertexOutput {
    @builtin(position) coordinates_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) tex_uv: vec2<f32>,
    @location(2) opacity: f32,
}

@group(2) @binding(0) var diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(diffuse, s_diffuse, in.tex_uv) * in.opacity;
}