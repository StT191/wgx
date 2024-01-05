
// depends on view from "uniform_const.wgsl"

struct VertexData {
    @builtin(position) position: vec4f,
    @location(0) P: vec2f,
};

@vertex
fn vs_main(
    @location(0) P: vec2f,
) -> VertexData {
    return VertexData(vec4f(P, 0.0, 1.0), P * view.w);
}