
// hight map function
const d_v0 = 0.007; const p_v0 = 88.0;
const d_v1 = 0.007; const p_v1 = 44.0;
const d_h0 = 0.005; const p_h0 = 124.0;
const d_h1 = 0.004; const p_h1 = 33.0;

const z =  0.978; // 1.0 - (d_v0 + d_v1 + d_h0 + d_h1);

fn height(v: f32, h: f32) -> f32 {
    return z + d_v0 * sin(p_v0 * v) + d_v1 * cos(p_v1 * v) + d_h0 * sin(p_h0 * h) + d_h1 * cos(p_h1 * h);
}
