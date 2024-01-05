
// matrix 4d
fn z_rotation(angle: f32) -> mat4x4f {
    return mat4x4f(vec4f(cos(angle), sin(angle), 0.0, 0.0), vec4f(-sin(angle), cos(angle), 0.0, 0.0), vec4f(0.0, 0.0, 1.0, 0.0), vec4f(0.0, 0.0, 0.0, 1.0));
}

fn translation(x:f32, y:f32, z:f32) -> mat4x4f {
    return mat4x4f(vec4f(1.0, 0.0, 0.0, 0.0), vec4f(0.0, 1.0, 0.0, 0.0), vec4f(0.0, 0.0, 1.0, 0.0), vec4f(x, y, z, 1.0));
}

fn from_vecs(O:vec3f, X:vec3f, Y:vec3f, Z:vec3f) -> mat4x4f {
    return mat4x4f(vec4f(X, 0.0), vec4f(Y, 0.0), vec4f(Z, 0.0), vec4f(O, 1.0));
}

fn from_scale(x:f32, y:f32, z:f32) -> mat4x4f {
    return mat4x4f(vec4f(x, 0.0, 0.0, 0.0), vec4f(0.0, y, 0.0, 0.0), vec4f(0.0, 0.0, z, 0.0), vec4f(0.0, 0.0, 0.0, 1.0));
}


// may need negating in left-handed coordinate system
fn normal_from_triangle(v0:vec3f, v1:vec3f, v2:vec3f) -> vec3f {
    return normalize(cross(v1 - v0, v2 - v0));
}


// homogenisation
fn homogen_2d(v:vec4f) -> vec2f {
    return vec2f(v.x/v.w, v.y/v.w);
}

fn homogen_3d(v:vec4f) -> vec3f {
    return vec3f(v.x/v.w, v.y/v.w, v.z/v.w);
}


// 2d
fn normal_2d(v:vec2f) -> vec2f {
    return vec2f(v.y, -v.x);
}

fn x_angle(v:vec2f, len:f32) -> f32 { // angle towards x axis
    var angle = acos(v.x / len); if (v.y < 0.0) { angle = -angle; }; return angle;
}

fn y_angle(v:vec2f, len:f32) -> f32 { // angle towards y axis
    var angle = acos(v.y / len); if (v.x > 0.0) { angle = -angle; }; return angle;
}

fn rotation_2d(angle: f32) -> mat2x2f {
    return mat2x2f(vec2f(cos(angle), sin(angle)), vec2f(-sin(angle), cos(angle)));
}
