
@group(0) @binding(1) var<uniform> clip: mat4x4<f32>;
@group(0) @binding(2) var<uniform> steps: u32;

struct VertexData {
    @builtin(position) position: vec4<f32>,
    @location(1) @interpolate(flat) color: vec4<f32>,
};


/* &import from_vecs from "util.wgsl" */

const Z0 = vec3<f32>(0.0, 0.0, 0.0);
const pi0 = 1.5707963267948966;


@vertex
fn vs_main(
    @builtin(vertex_index) v_i: u32,
    @location(0) X: vec3<f32>,
    @location(1) O: vec3<f32>,
    @location(2) Y: vec3<f32>,
    @location(3) color: u32,
) -> VertexData {
    var out: VertexData;

    out.color = unpack4x8unorm(color);

    let i = v_i % 3u; // which vertex

    if (i == 1u) {
        out.position = clip * vec4<f32>(O, 1.0);
    }
    else {
        var j = f32(v_i / 3u);

        if (i == 2u) {
            j = j + 1.0;
        }

        let fi = j / f32(steps) * pi0;

        out.position = clip * from_vecs(O, X-O, Y-O, Z0) * vec4<f32>(cos(fi), sin(fi), 0.0, 1.0);
    }

    return out;
}


@fragment
fn fs_main(in: VertexData) -> @location(0) vec4<f32> {
    return in.color;
}