
use crate::error::*;
use cgmath::*;

// utilities

#[macro_export]
macro_rules! apply {
    ($matrix:expr, $value:expr) => {
        $matrix = $value * $matrix
    }
}


pub fn within<S:BaseFloat>(outer:&Matrix4<S>, inner:&Matrix4<S>) -> Res<Matrix4<S>> {
    Ok(outer.invert().ok_or("couldn't invert matrix")? * inner * outer)
}



// projections

pub struct FovProjection {
    pub fovy: Deg<f32>,
    pub aspect: f32,
    pub unit: f32,
    pub distance: f32,
    pub projection: Matrix4<f32>,
    pub translation: Matrix4<f32>,
}


impl FovProjection {

    pub fn new(fov_deg:f32, aspect:f32, unit:f32, distance:f32) -> Self {

        let fovy = Deg(fov_deg);

        Self {
            fovy, aspect, unit, distance,
            projection: Matrix4::from(PerspectiveFov {
                fovy: fovy.into(),
                aspect,
                near: unit/1.0e3,
                far: 2.0e3*unit,
            }) * Matrix4::from_nonuniform_scale(1.0, 1.0, -1.0),
            translation: Matrix4::from_translation((0.0, 0.0, distance).into()),
        }
    }

    pub fn unit(fov_deg:f32, aspect:f32, unit:f32) -> Self {
        Self::new(fov_deg, aspect, unit, unit / Deg(fov_deg/2.0).tan())
    }

    pub fn window(fov_deg:f32, width:f32, height:f32) -> Self {
        Self::new(fov_deg, width/height, f32::max(width, height), 0.5 * height / Deg(fov_deg/2.0).tan())
    }
}




pub fn flat_window_projection(width:f32, height:f32, depth:f32) -> Matrix4<f32> {
    Matrix4::from_translation((-1.0, 1.0, 0.0).into()) *
    Matrix4::from_nonuniform_scale(2.0/width, -2.0/height, if depth == 0.0 { 0.0 } else { 1.0/depth})
}