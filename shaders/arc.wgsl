
// locals
[[block]] struct Matrix { matrix: mat4x4<f32>; };

[[group(0), binding(0)]] var<uniform> clip: Matrix;
[[group(0), binding(1)]] var<uniform> pix: Matrix;


struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0), interpolate(flat)]] color: vec4<f32>;
    [[location(1), interpolate(perspective)]] E: vec2<f32>;
    [[location(2), interpolate(flat)]] prj: mat4x4<f32>;
};


fn translation(x:f32, y:f32, z:f32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, x), vec4<f32>(0.0, 1.0, 0.0, y), vec4<f32>(0.0, 0.0, 1.0, z), vec4<f32>(0.0, 0.0, 0.0, 1.0));
}

fn normal_2d(vec:vec2<f32>) -> vec2<f32> {
    return vec2<f32>(vec.y, -vec.x);
}

fn homogen_2d(vec:vec4<f32>) -> vec2<f32> {
    return vec2<f32>(vec.x/vec.w, vec.y/vec.w);
}


[[stage(vertex)]]
fn vs_main(
    [[location(0)]] E: vec2<f32>, // normalized corner
    [[location(1)]] color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = clip.matrix * vec4<f32>(E, 0.0, 1.0);
    out.color = color;
    out.E = E;

    let O = (pix.matrix * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xy; // origin

    out.prj = translation(-O.x, -O.y, 0.0) * pix.matrix;;

    return out;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {


    let e = length(in.E);

    if (e > 1.0) {
        discard;
        // return vec4<f32>(1.0, 1.0, 0.0, 1.0);
    }

    // get pixel distance to rim
    let R = homogen_2d(in.prj * vec4<f32>(in.E, 0.0, 1.0)); // R in pixel space
    let r = length(R); // length of R in pixel space

    // R at rim
    let Ep = in.E * 1.0/e; // R at rim in unit circle
    let Rp = homogen_2d(in.prj * vec4<f32>(Ep, 0.0, 1.0)); // R at rim in pixel space
    let rp = length(Rp);

    let T = Rp - homogen_2d(in.prj * vec4<f32>(Ep + normal_2d(Ep), 0.0, 1.0)); // tangent in pixel space


    let ds = (rp - r) * sin(acos(dot(Rp, T) / (rp*length(T)))); // skew corrected pixel distance to edge


    var color = in.color;

    let g = 1.0 * sqrt(2.0); // fade threshold

    if (ds < g) {
        color.a = color.a * (ds / g);
        // color.g = 1.0 - (ds / g);
        // color.g = (ds / g);
        // return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    }

    return color;
}