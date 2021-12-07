
[[block]] struct Matrix { m: mat4x4<f32>; };

[[group(0), binding(0)]] var<uniform> clip: Matrix;
[[group(0), binding(1)]] var<uniform> light: Matrix;


struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_coord: vec2<f32>;
    [[location(1)]] fl: f32;
};


[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] tex_coord: vec3<f32>,
    [[location(2)]] normal: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = clip.m * vec4<f32>(position, 1.0);
    out.tex_coord = tex_coord.xy;
    out.fl = -(light.m * vec4<f32>(normal, 1.0)).z;

    return out;
}


[[group(0), binding(2)]] var color_texture: texture_2d<f32>;
[[group(0), binding(3)]] var color_sampler: sampler;


let shade = vec2<f32>(0.1, 0.5);


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    let color = textureSample(color_texture, color_sampler, in.tex_coord);

    var f = shade.x;

    if (in.fl > 0.0) {
        f = f + (in.fl + (1.0 - in.fl) * shade.y) * (1.0 - shade.x);
    } else {
        f = f + (1.0 + in.fl) * shade.y * (1.0 - shade.x);
    }

    return vec4<f32>(color.xyz * f, 1.0);
}