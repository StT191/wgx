
// uniform
@group(0) @binding(0) var<uniform> viewport: vec3<f32>;
@group(0) @binding(1) var<uniform> scale: f32;

var<push_constant> time: f32; // time in secs


// math constants

let pi0 = 1.5707963267948966;
let pi = 3.141592653589793;
let pi2 = 6.283185307179586;
let sqrt2 = 1.4142135623730951;
