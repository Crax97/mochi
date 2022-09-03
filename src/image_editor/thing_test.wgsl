struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) coordinates_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) tex_uv: vec2<f32>,
}

@vertex
fn vs(in: VertexInput) -> VertexOutput {
    var out = VertexOutput();
    out.coordinates_position = vec4<f32>(in.position, 1.0);
    out.position = in.position;
    out.tex_uv = in.tex_uv;
    return out;
}


@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.6, 0.2, 0.8, 1.0);
}