
/* &import * from "../shaders/circular.wgsl" */

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexData {
    var out: VertexData;

    out.position = vec4<f32>(position, 1.0);
    out.tex_coord = tex_coord;

    return out;
}