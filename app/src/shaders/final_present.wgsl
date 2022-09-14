struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) coordinates_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) tex_uv: vec2<f32>,
}

struct FinalPresentData {
    canvas_size: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> data: FinalPresentData;

@vertex
fn vs(in: VertexInput) -> VertexOutput {
    var out : VertexOutput;
    let aspect = data.canvas_size.x / data.canvas_size.y;
    let pos = vec3<f32>(in.position.x, in.position.y, in.position.z);
    out.coordinates_position = vec4<f32>(pos, 1.0);
    out.position = in.position;
    out.tex_uv = in.tex_uv;
    return out;
}


@group(0) @binding(0) var diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(diffuse, s_diffuse, in.tex_uv);
}