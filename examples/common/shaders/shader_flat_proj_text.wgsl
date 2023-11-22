
@group(0) @binding(0) var<uniform> projection: mat4x4<f32>;

/* &import * from "frag_flat_text.wgsl" */

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexData {
    var out: VertexData;

    out.position = projection * vec4<f32>(position, 1.0);
    out.tex_coord = tex_coord;

    return out;
}