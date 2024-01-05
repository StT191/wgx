
// sdf functions

fn sdSphere(P: vec3f, pos: vec3f, r: f32) -> f32 {
  return length(P - pos) - r;
}

fn sdBox(P: vec3f, pos: vec3f, dim: vec3f) -> f32 {
  return length(max(abs(P - pos) - dim, vec3f(0.0)));
}


// operations

fn opSmoothUnion(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2-d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k*h * (1.0-h);
}
