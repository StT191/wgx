
struct VertexData {
    @builtin(position) position: vec4f,
    @location(0) index: vec2f,
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexData {
    let x = f32(i % 2);
    let y = f32(i / 2 % 2);
    return VertexData(
        vec4f(mix(-1.0, 1.0, x), mix(-1.0, 1.0, y), 0.0, 1.0),
        vec2f(6.0 * steps * x, 3.0 * steps * y),
    );
}


struct FragmetOutput {
    @location(0) pos: vec4f,
    @location(1) normal: vec4f,
}

override steps = 0.0;
override step_da = 0.0; // delta angle per step

@fragment
fn fs_main(in: VertexData) -> FragmetOutput {

    let index = floor(in.index);

    let i = u32(index.x) % 6; // 0..5

    let vert = i % 3; // 0,1,2
    let face = i / 3; // 0,1

    var delta_step: vec2f;

    switch vert {
        default: { delta_step = vec2f(0.0, 0.0); }
        case 1u: { switch face {
            default: { delta_step = vec2f(1.0, 0.0); }
            case 1u: { delta_step = vec2f(1.0, 1.0); }
        }}
        case 2u: { switch face {
            default: { delta_step = vec2f(1.0, 1.0); }
            case 1u: { delta_step = vec2f(0.0, 1.0); }
        }}
    }

    let step = vec2f(
        floor(index.x / 6.0), // 0..steps
        index.y % steps, // 0..steps
    );

    let angles = (step + delta_step) * step_da;
    let angles_n = (step + vec2f(0.5)) * step_da;

    let c = tan(angles); // components
    let c_n = tan(angles_n);

    let z = u32(index.y / steps); // 0,1,2

    var proj: vec3f;
    var proj_n: vec3f;

    switch z {
        default: {
            proj = vec3f(c.x, c.y, -1.0);
            proj_n = vec3f(c_n.x, c_n.y, -1.0);
        }
        case 1u: {
            proj = vec3f(1.0, c.x, -c.y);
            proj_n = vec3f(1.0, c_n.x, -c_n.y);
        }
        case 2u: {
            proj = vec3f(c.y, 1.0, -c.x);
            proj_n = vec3f(c_n.y, 1.0, -c_n.x);
        }
    }

    let pos = normalize(proj); // point on sphere
    let normal = normalize(proj_n); // point on sphere

    return FragmetOutput(
        vec4f(pos, 0.0), // positon
        vec4f(normal, 0.0), // normal
    );
}