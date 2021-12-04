
// vertex
[[block]] struct Matrix { m: mat4x4<f32>; };
// [[block]] struct Vec2 { v: vec2<f32>; };
[[block]] struct U32 { u: u32; };

// [[group(0), binding(0)]] var<uniform> world: Matrix;
[[group(0), binding(1)]] var<uniform> clip: Matrix;
[[group(0), binding(2)]] var<uniform> steps: U32;
// [[group(0), binding(2)]] var<uniform> viewport: Vec2;


struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(1), interpolate(flat)]] color: vec4<f32>;
};


// untility functions
fn from_vecs(O:vec3<f32>, X:vec3<f32>, Y:vec3<f32>, Z:vec3<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(X, 0.0), vec4<f32>(Y, 0.0), vec4<f32>(Z, 0.0), vec4<f32>(O, 1.0));
}
fn homogen_3d(vec:vec4<f32>) -> vec3<f32> {
    return vec3<f32>(vec.x/vec.w, vec.y/vec.w, vec.z/vec.w);
}

let Z0 = vec3<f32>(0.0, 0.0, 0.0);
let pi0 = 1.5707963267948966;


[[stage(vertex)]]
fn vs_main(
    [[builtin(vertex_index)]] v_i: u32,
    [[location(0)]] X: vec3<f32>,
    [[location(1)]] O: vec3<f32>,
    [[location(2)]] Y: vec3<f32>,
    [[location(3)]] color: u32,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = unpack4x8unorm(color);

    let i = v_i % 3u; // which vertex

    if (i == 1u) {
        out.position = clip.m * vec4<f32>(O, 1.0);
    }
    else {
        var j = f32(v_i / 3u);

        if (i == 2u) {
            j = j + 1.0;
        }

        let fi = j / f32(steps.u) * pi0;

        out.position = clip.m * from_vecs(O, X-O, Y-O, Z0) * vec4<f32>(cos(fi), sin(fi), 0.0, 1.0);
    }

    return out;
}



[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    // if (in.color.a == 0.0) {
    //     discard;
    // }

    return in.color;
}