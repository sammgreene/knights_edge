struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> Time: f32;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Height factor (assumes grass base at y=0, tip at y=1)
    let height = in.position.y;

    // World-space-ish variation using XZ
    let wave = sin(Time * 1.8 + in.position.x * 2.0)
             + 0.4 * sin(Time * 4.1 + in.position.z * 3.3);

    let sway_strength = 0.12;

    // Bend more at the top
    let offset_x = wave * sway_strength * height;

    let new_position = in.position + vec3<f32>(offset_x, 0.0, 0.0);

    out.clip_position = vec4<f32>(new_position, 1.0);
    out.uv = in.uv;

    return out;
}