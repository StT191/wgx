
struct Vertex { x: f32, y: f32, z: f32, tx: f32, ty: f32, tz: f32, nx: f32, ny: f32, nz: f32 };

@group(0) @binding(0) var<storage, read_write> vertices: array<Vertex>;


/* &include "util.wgsl" */
/* &include "cp_height_map.wgsl" */

// consts
const FRAC_PI_4 = 0.7853981633974483;

const t = vec3<f32>(-1.0, -1.0, -1.0); // texture coordinates



@compute @workgroup_size(8, 8, 3)
fn cp_main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(num_workgroups) workgroups: vec3<u32>,
) {

    let v = f32(global_id.y);
    let h = f32(global_id.x);

    let da = FRAC_PI_4 / f32(8u * workgroups.y);

    let v0 = tan(da * (v + 0.0));
    let v1 = tan(da * (v + 1.0));

    let h0 = tan(da * (h + 0.0));
    let h1 = tan(da * (h + 1.0));


    var a: vec3<f32>; var b: vec3<f32>; var c: vec3<f32>; var d: vec3<f32>;

    if (global_id.z == 0u) {
        a = vec3<f32>(h0, v0, -1.0);
        b = vec3<f32>(h1, v0, -1.0);
        c = vec3<f32>(h1, v1, -1.0);
        d = vec3<f32>(h0, v1, -1.0);
    }
    else if (global_id.z == 1u) {
        a = vec3<f32>(1.0, h0, -v0);
        b = vec3<f32>(1.0, h1, -v0);
        c = vec3<f32>(1.0, h1, -v1);
        d = vec3<f32>(1.0, h0, -v1);
    }
    else if (global_id.z == 2u) {
        a = vec3<f32>(v0, 1.0, -h0);
        b = vec3<f32>(v0, 1.0, -h1);
        c = vec3<f32>(v1, 1.0, -h1);
        d = vec3<f32>(v1, 1.0, -h0);
    }

    a = normalize(a);
    b = normalize(b);
    c = normalize(c);
    d = normalize(d);


    let d_v0 = 0.004; let p_v0 = 88.0;
    let d_v1 = 0.003; let p_v1 = 11.0;
    let d_h0 = 0.005; let p_h0 = 124.0;
    let d_h1 = 0.004; let p_h1 = 05.0;

    let z = 1.0 - (d_v0 + d_v1 + d_h0 + d_h1);

    a = a * height(v0, h0);
    b = b * height(v0, h1);
    c = c * height(v1, h1);
    d = d * height(v1, h0);

    // index
    let size_f = 6u; // size of field
    let size_r = 8u * workgroups.x * size_f; // row size
    let size_g = 8u * workgroups.y * size_r; // whole grid size

    let i = global_id.x * size_f + global_id.y * size_r + global_id.z * size_g;

    let n0 = -normal_from_triangle(a, b, c);

    vertices[i+0u] = Vertex(a.x, a.y, a.z,  v0, h0, 0.0,  n0.x, n0.y, n0.z);
    vertices[i+1u] = Vertex(b.x, b.y, b.z,  v0, h1, 0.0,  n0.x, n0.y, n0.z);
    vertices[i+2u] = Vertex(c.x, c.y, c.z,  v1, h1, 0.0,  n0.x, n0.y, n0.z);

    // vertices[i+0u] = Vertex(a.x, a.y, a.z,  v0, h0, 0.0,  a.x, a.y, a.z);
    // vertices[i+1u] = Vertex(b.x, b.y, b.z,  v0, h1, 0.0,  b.x, b.y, b.z);
    // vertices[i+2u] = Vertex(c.x, c.y, c.z,  v1, h1, 0.0,  c.x, c.y, c.z);

    let n1 = -normal_from_triangle(a, c, d);

    vertices[i+3u] = Vertex(a.x, a.y, a.z,  v0, h0, 0.0,  n1.x, n1.y, n1.z);
    vertices[i+4u] = Vertex(c.x, c.y, c.z,  v1, h1, 0.0,  n1.x, n1.y, n1.z);
    vertices[i+5u] = Vertex(d.x, d.y, d.z,  v1, h0, 0.0,  n1.x, n1.y, n1.z);

    // vertices[i+3u] = Vertex(a.x, a.y, a.z,  v0, h0, 0.0,  a.x, a.y, a.z);
    // vertices[i+4u] = Vertex(c.x, c.y, c.z,  v1, h1, 0.0,  c.x, c.y, c.z);
    // vertices[i+5u] = Vertex(d.x, d.y, d.z,  v1, h0, 0.0,  d.x, d.y, d.z);
}