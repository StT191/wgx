
use cgmath::*;


pub fn unit_fov_projection(fov_deg: f32, aspect:f32, unit: f32) -> Matrix4<f32> {
    let dist = unit / Deg(fov_deg/2.0).tan();
    Matrix4::from(PerspectiveFov {
        fovy: Deg(fov_deg).into(),
        aspect, near: unit/1.0e3, far: 2.0e3*unit,
    }) *
    Matrix4::<f32>::from_translation((0.0, 0.0, -dist).into())
}


pub fn flat_window_projection(width: f32, height: f32) -> Matrix4<f32> {
    Matrix4::from_translation((-1.0, 1.0, 0.0).into()) *
    Matrix4::from_nonuniform_scale(1.0/width, -1.0/height, 0.0)
}


pub fn window_fov_projection(fov_deg: f32, width: f32, height: f32) -> Matrix4<f32> {
    let unit = f32::max(width, height);
    let dist = 0.5 * height / Deg(fov_deg/2.0).tan();
    Matrix4::from(PerspectiveFov {
        fovy: Deg(fov_deg).into(),
        aspect: width/height,
        near: unit/1.0e3, far: 2.0e3*unit,
    }) *
    Matrix4::<f32>::from_translation((0.0, 0.0, -dist).into())
}