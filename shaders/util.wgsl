
fn x_angle(vec:vec2<f32>, len:f32) -> f32 { // angle towards x axis
    var angle = acos(vec.x / len); if (vec.y < 0.0) { angle = -angle; }; return angle;
}

fn y_angle(vec:vec2<f32>, len:f32) -> f32 { // angle towards y axis
    var angle = acos(vec.y / len); if (vec.x > 0.0) { angle = -angle; }; return angle;
}

fn rotation_2d(angle: f32) -> mat2x2<f32> {
    return mat2x2<f32>(vec2<f32>(cos(angle), sin(angle)), vec2<f32>(-sin(angle), cos(angle)));
}

fn z_rotation(angle: f32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(cos(angle), sin(angle), 0.0, 0.0), vec4<f32>(-sin(angle), cos(angle), 0.0, 0.0), vec4<f32>(0.0, 0.0, 1.0, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));
}

fn translation(x:f32, y:f32, z:f32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, x), vec4<f32>(0.0, 1.0, 0.0, y), vec4<f32>(0.0, 0.0, 1.0, z), vec4<f32>(0.0, 0.0, 0.0, 1.0));
}

fn normal_2d(vec:vec2<f32>) -> vec2<f32> {
    return vec2<f32>(vec.y, -vec.x);
}

fn homogen_2d(vec:vec4<f32>) -> vec2<f32> {
    return vec2<f32>(vec.x/vec.w, vec.y/vec.w);
}