
@group(0) @binding(1) var<uniform> clip: mat4x4<f32>;

struct VertexData {
    @builtin(position) position: vec4<f32>,
    @location(1) @interpolate(flat) color: vec4<f32>,
};


/* &import from_vecs from "util.wgsl" */

let Z0 = vec3<f32>(0.0, 0.0, 0.0);


@vertex
fn vs_main(
    @location(0) X: vec3<f32>,
    @location(1) O: vec3<f32>,
    @location(2) Y: vec3<f32>,
    @location(3) color: vec4<f32>,
    @location(4) R: vec4<f32>,
) -> VertexData {
    var out: VertexData;

    out.color = color;

    out.position = clip * from_vecs(O, X-O, Y-O, Z0) * R;

    return out;
}


@fragment
fn fs_main(in: VertexData) -> @location(0) vec4<f32> {
    return in.color;
}