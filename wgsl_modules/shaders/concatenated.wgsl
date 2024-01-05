
struct VertexData {
    @builtin(position) position: vec4f,
    @location(0) tex_coord: vec2f,
};

@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;

@fragment
fn fs_main(in: VertexData) -> @location(0) vec4f {
    let color = textureSample(color_texture, color_sampler, in.tex_coord);
    return color;
}

fn normal_2d(v:vec2f) -> vec2f {
    return vec2f(v.y, -v.x);
}

@vertex
fn vs_main(
    @location(0) position: vec3f,
    @location(1) tex_coord: vec2f,
) -> VertexData {
    var out: VertexData;

    out.position = vec4f(position, 1.0);
    out.tex_coord = tex_coord;

    return out;
}