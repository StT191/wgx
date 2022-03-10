
// locals
struct Projection { matrix: mat4x4<f32>; };
struct Dim { v: vec2<f32>; flat: vec2<f32>; };

[[group(0), binding(0)]] var<uniform> projection: Projection;
[[group(0), binding(1)]] var<uniform> dim: Dim;


struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0), interpolate(flat)]] color: vec4<f32>;
    [[location(1)]] R: vec2<f32>;
};


[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec2<f32>,
    [[location(1)]] color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = projection.matrix * vec4<f32>(position, 0.0, 1.0);
    out.R = position * dim.v;
    out.color = color;

    return out;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    let r = length(in.R);
    let Rn = in.R / r; // = (cos_a, sin_a)

    let re = dim.v.x * dim.v.y / length(Rn * vec2<f32>(dim.v.y, dim.v.x)); // distance at edge

    if (r > re) {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0);
        // discard;
    }

    let N = re*Rn / (dim.v * dim.v);
    let No = N/length(N) * dim.flat; // pespective corrected normal

    let ds = (re - r) * dot(Rn, No); // delta to edge in pixel space

    var color = in.color;

    let g = 20.0; // blur threshold

    if (ds < g) {
        color.g = 1.0 * (ds / g);
    }

    return color;
}