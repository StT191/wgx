
// compute
// struct U32 { u: u32; };

struct Instance {
    x0: f32; y0: f32; z0: f32;
    x1: f32; y1: f32; z1: f32;
    x2: f32; y2: f32; z2: f32;
    color: u32;
};
struct InstanceArray { data: [[stride(40)]] array<Instance>; };

struct Vertex { P: vec4<f32>; color: vec4<f32>; };
struct VertexArray { data: [[stride(32)]] array<Vertex>; };

[[group(0), binding(3)]] var<storage> instances: InstanceArray;
[[group(0), binding(4)]] var<storage, write> vertices: VertexArray;
// [[group(0), binding(5)]] var<uniform> steps: U32;


// untility functions
fn from_vecs(O:vec3<f32>, X:vec3<f32>, Y:vec3<f32>, Z:vec3<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(X, 0.0), vec4<f32>(Y, 0.0), vec4<f32>(Z, 0.0), vec4<f32>(O, 1.0));
}
fn homogen_3d(v:vec4<f32>) -> vec3<f32> {
    return vec3<f32>(v.x/v.w, v.y/v.w, v.z/v.w);
}

let Z0 = vec3<f32>(0.0, 0.0, 0.0);
let pi0 = 1.5707963267948966;


[[stage(compute), workgroup_size(1)]]
fn cp_main(
    [[builtin(global_invocation_id)]] global_id: vec3<u32>,
    [[builtin(num_workgroups)]] workgroups: vec3<u32>,
) {
    let is = instances.data[global_id.x];

    var color = unpack4x8unorm(is.color);

    // if (global_id.y % 2u == 0u) {
    //     color.a = 0.0;
    // }

    let O = vec3<f32>(is.x1, is.y1, is.z1);
    let X = vec3<f32>(is.x0, is.y0, is.z0) - O;
    let Y = vec3<f32>(is.x2, is.y2, is.z2) - O;

    let prj = from_vecs(O, X, Y, Z0);

    let steps_f = f32(workgroups.y);

    let fi0 = f32(global_id.y) / steps_f * pi0;
    let fi1 = f32(global_id.y+1u) / steps_f * pi0;

    let i = 3u * (global_id.x * workgroups.y + global_id.y);

    vertices.data[i]    = Vertex(prj * vec4<f32>(cos(fi0), sin(fi0), 0.0, 1.0), color);
    vertices.data[i+1u] = Vertex(vec4<f32>(O, 1.0), color);
    vertices.data[i+2u] = Vertex(prj * vec4<f32>(cos(fi1), sin(fi1), 0.0, 1.0), color);
}


// vertex
struct Matrix { m: mat4x4<f32>; };
// struct Vec2 { v: vec2<f32>; };

// [[group(0), binding(0)]] var<uniform> world: Matrix;
[[group(0), binding(1)]] var<uniform> clip: Matrix;
// [[group(0), binding(2)]] var<uniform> viewport: Vec2;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(1), interpolate(flat)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[builtin(vertex_index)]] i: u32,
    [[location(0)]] P: vec4<f32>,
    [[location(1)]] color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = color;

    out.position = clip.m * P;

    return out;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    // if (in.color.a == 0.0) {
    //     discard;
    // }

    return in.color;
}