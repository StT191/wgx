
// depends on view from "uniform_const.wgsl"

struct VertexData {
    @builtin(position) position: vec4<f32>,
    @location(0) P: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) P: vec2<f32>,
) -> VertexData {
    return VertexData(vec4<f32>(P, 0.0, 1.0), P * view.w);
}