
struct VertexData {
    @builtin(position) position: vec4f,
    @location(9) tex_coord: vec2f,
};

@vertex
fn vs_main(
    @location(0) position: vec3f,
    @location(5) tex_coord: mat4x4f,
) -> VertexData {
    var out: VertexData;

    out.position = vec4f(position, 1.0);
    out.tex_coord = tex_coord;

    return out;
}