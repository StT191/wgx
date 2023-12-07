
struct VertexData {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;

@fragment
fn fs_main(in: VertexData) -> @location(0) vec4<f32> {
    let color = textureSample(color_texture, color_sampler, in.tex_coord);
    return color;
}