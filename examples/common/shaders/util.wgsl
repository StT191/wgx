
// matrix 4d
fn z_rotation(angle: f32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(cos(angle), sin(angle), 0.0, 0.0), vec4<f32>(-sin(angle), cos(angle), 0.0, 0.0), vec4<f32>(0.0, 0.0, 1.0, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));
}

fn translation(x:f32, y:f32, z:f32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0), vec4<f32>(0.0, 1.0, 0.0, 0.0), vec4<f32>(0.0, 0.0, 1.0, 0.0), vec4<f32>(x, y, z, 1.0));
}

fn from_vecs(O:vec3<f32>, X:vec3<f32>, Y:vec3<f32>, Z:vec3<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(X, 0.0), vec4<f32>(Y, 0.0), vec4<f32>(Z, 0.0), vec4<f32>(O, 1.0));
}

fn from_scale(x:f32, y:f32, z:f32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(x, 0.0, 0.0, 0.0), vec4<f32>(0.0, y, 0.0, 0.0), vec4<f32>(0.0, 0.0, z, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));
}


// may need negating in left-handed coordinate system
fn normal_from_triangle(v0:vec3<f32>, v1:vec3<f32>, v2:vec3<f32>) -> vec3<f32> {
    return normalize(cross(v1 - v0, v2 - v0));
}


// homogenisation
fn homogen_2d(v:vec4<f32>) -> vec2<f32> {
    return vec2<f32>(v.x/v.w, v.y/v.w);
}

fn homogen_3d(v:vec4<f32>) -> vec3<f32> {
    return vec3<f32>(v.x/v.w, v.y/v.w, v.z/v.w);
}


// 2d
fn normal_2d(v:vec2<f32>) -> vec2<f32> {
    return vec2<f32>(v.y, -v.x);
}

fn x_angle(v:vec2<f32>, len:f32) -> f32 { // angle towards x axis
    var angle = acos(v.x / len); if (v.y < 0.0) { angle = -angle; }; return angle;
}

fn y_angle(v:vec2<f32>, len:f32) -> f32 { // angle towards y axis
    var angle = acos(v.y / len); if (v.x > 0.0) { angle = -angle; }; return angle;
}

fn rotation_2d(angle: f32) -> mat2x2<f32> {
    return mat2x2<f32>(vec2<f32>(cos(angle), sin(angle)), vec2<f32>(-sin(angle), cos(angle)));
}
