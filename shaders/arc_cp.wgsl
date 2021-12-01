
// defs
[[block]] struct Matrix { m: mat4x4<f32>; };
[[block]] struct Vec2 { v: vec2<f32>; };

struct Instance {
    P0: vec3<f32>; P1: vec3<f32>; P2: vec3<f32>;
    ty: f32; color: u32;
};
[[block]] struct InstanceArray { data: [[stride(64)]] array<Instance>; };

struct Vertex { P: vec3<f32>; color: u32; };
[[block]] struct VertexArray { data: [[stride(16)]] array<Vertex>; };


// bindings
[[group(0), binding(0)]] var<uniform> world: Matrix;
[[group(0), binding(1)]] var<uniform> clip: Matrix;
[[group(0), binding(2)]] var<uniform> viewport: Vec2;


[[group(0), binding(3)]] var<storage> instances: InstanceArray;
[[group(0), binding(4)]] var<storage, write> vertices: VertexArray;



[[stage(compute), workgroup_size(1)]]
fn cp_main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {

    let instance = instances.data[global_id.x];

    let i = global_id.x * 3u;

    vertices.data[i] = Vertex(instance.P0, instance.color);
    vertices.data[i+1u] = Vertex(instance.P1, instance.color);
    vertices.data[i+2u] = Vertex(instance.P2, instance.color);
}



struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(1), interpolate(flat)]] color: vec4<f32>;
};


[[stage(vertex)]]
fn vs_main(
    [[builtin(vertex_index)]] i: u32,
    [[location(0)]] P: vec3<f32>,
    [[location(1)]] color: u32,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = unpack4x8unorm(color);

    out.position = clip.m * world.m * vec4<f32>(P, 1.0);

    return out;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    return in.color;
}