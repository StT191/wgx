
// vertex
struct Matrix { m: mat4x4<f32> };
// struct Vec2 { v: vec2<f32> };
struct U32 { u: u32 };

// @group(0), binding(0) var<uniform> world: Matrix;
@group(0) @binding(1) var<uniform> clip: Matrix;
// @group(0) @binding(2) var<uniform> steps: U32;
// @group(0) @binding(2) var<uniform> viewport: Vec2;


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) @interpolate(flat) color: vec4<f32>,
};


// untility functions
fn from_vecs(O:vec3<f32>, X:vec3<f32>, Y:vec3<f32>, Z:vec3<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(X, 0.0), vec4<f32>(Y, 0.0), vec4<f32>(Z, 0.0), vec4<f32>(O, 1.0));
}
fn homogen_3d(v:vec4<f32>) -> vec3<f32> {
    return vec3<f32>(v.x/v.w, v.y/v.w, v.z/v.w);
}

let Z0 = vec3<f32>(0.0, 0.0, 0.0);

@vertex
fn vs_main(
    @location(0) X: vec3<f32>,
    @location(1) O: vec3<f32>,
    @location(2) Y: vec3<f32>,
    @location(3) color: vec4<f32>,
    @location(4) R: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = color;

    out.position = clip.m * from_vecs(O, X-O, Y-O, Z0) * R;

    return out;
}



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    // if (in.color.a == 0.0) {
    //     discard;
    // }

    return in.color;
}