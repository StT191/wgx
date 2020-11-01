
use cgmath::*;


pub fn unit_view(fov_deg: f32, aspect:f32, unit: f32) -> Matrix4<f32> {
    let dist = unit / Deg(fov_deg/2.0).tan();
    Matrix4::from(PerspectiveFov {
        fovy: Deg(fov_deg).into(),
        aspect, near: unit/1.0e3, far: 2.0e3*dist,
    }) *
    Matrix4::<f32>::from_translation((0.0, 0.0, -dist).into())
}