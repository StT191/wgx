
pub use glam::{self, *};


// helper macro
macro_rules! impl_multi {
    ( $trait:ty => $($type:ty),+ => $tokens:tt) => { $( impl $trait for $type $tokens )+ }
}


pub trait NormalFromTriangleExtension {
    fn normal_from_triangle(v0: Self, v1: Self, v2: Self) -> Self;
}

impl_multi!{
    NormalFromTriangleExtension => Vec3, Vec3A, DVec3 => {
        fn normal_from_triangle(v0: Self, v1: Self, v2: Self) -> Self {
            (v1 - v0).cross(v2 - v0).normalize()
        }
    }
}


pub trait FlatViewPortLhExtension {
    type F;
    fn flat_viewport_lh(width: Self::F, height: Self::F, depth: Self::F) -> Self;
}

impl FlatViewPortLhExtension for Mat4 {
    type F = f32;
    fn flat_viewport_lh(width: f32, height: f32, depth: f32) -> Self {
        Self::from_translation([-1.0, 1.0, 0.0].into()) *
        Self::from_scale([2.0/width, -2.0/height, if depth == 0.0 { 0.0 } else { 1.0/depth }].into())
    }
}

impl FlatViewPortLhExtension for DMat4 {
    type F = f64;
    fn flat_viewport_lh(width: f64, height: f64, depth: f64) -> Self {
        Self::from_translation([-1.0, 1.0, 0.0].into()) *
        Self::from_scale([2.0/width, -2.0/height, if depth == 0.0 { 0.0 } else { 1.0/depth }].into())
    }
}


pub trait FromUniformScaleExtension {
    type F;
    fn from_uniform_scale(f: Self::F) -> Self;
}

impl FromUniformScaleExtension for Mat4 {
    type F = f32;
    fn from_uniform_scale(f: f32) -> Self { Self::from_scale([f, f, f].into()) }
}

impl FromUniformScaleExtension for DMat4 {
    type F = f64;
    fn from_uniform_scale(f: f64) -> Self { Self::from_scale([f, f, f].into()) }
}


pub trait FromToAttitudeExtension {
    type A;
    fn from_attitude(attitude: Self::A) -> Self;
    fn to_attitude(&self) -> Self::A;
}

pub const ATTITUDE: EulerRot = EulerRot::ZXY; // roll|pich|yaw

impl_multi!{
    FromToAttitudeExtension => Mat4, Mat3, Mat3A, Quat => {
        type A = Vec3;
        fn from_attitude(Vec3 {x:yaw, y:pitch, z:roll}: Vec3) -> Self {
            Self::from_euler(ATTITUDE, roll, pitch, yaw)
        }
        fn to_attitude(&self) -> Vec3 {
            let (roll, pitch, yaw) = self.to_euler(ATTITUDE);
            Vec3::new(yaw, pitch, roll)
        }
    }
}

impl FromToAttitudeExtension for Affine3A {
    type A = Vec3;
    fn from_attitude(attitude: Vec3) -> Self {
        Self { matrix3: Mat3A::from_attitude(attitude), ..Self::default() }
    }
    fn to_attitude(&self) -> Vec3 { self.matrix3.to_attitude() }
}

impl_multi!{
    FromToAttitudeExtension => DMat4, DMat3, DQuat => {
        type A = DVec3;
        fn from_attitude(DVec3 {x:yaw, y:pitch, z:roll}: DVec3) -> Self {
            Self::from_euler(ATTITUDE, roll, pitch, yaw)
        }
        fn to_attitude(&self) -> DVec3 {
            let (roll, pitch, yaw) = self.to_euler(ATTITUDE);
            DVec3::new(yaw, pitch, roll)
        }
    }
}

impl FromToAttitudeExtension for DAffine3 {
    type A = DVec3;
    fn from_attitude(attitude: DVec3) -> Self {
        Self { matrix3: DMat3::from_attitude(attitude), ..Self::default() }
    }
    fn to_attitude(&self) -> DVec3 { self.matrix3.to_attitude() }
}


pub trait NormalizeAngleExtension {
    fn normalize_angle(self) -> Self;
}

impl NormalizeAngleExtension for f32 {
    fn normalize_angle(self) -> f32 { self.rem_euclid(2.0 * std::f32::consts::PI) }
}

impl NormalizeAngleExtension for f64 {
    fn normalize_angle(self) -> f64 { self.rem_euclid(2.0 * std::f64::consts::PI) }
}


pub trait NormalizeAnglesExtension {
    fn normalize_angles(self) -> Self;
}

impl_multi!{
    NormalizeAnglesExtension => Vec4, Vec3, Vec3A, Vec2 => {
        fn normalize_angles(self) -> Self { self.map(f32::normalize_angle) }
    }
}

impl_multi!{
    NormalizeAnglesExtension => DVec4, DVec3, DVec2 => {
        fn normalize_angles(self) -> Self { self.map(f64::normalize_angle) }
    }
}


pub trait HomogenizeExtension {
    fn homogenize(self) -> Self;
}

impl_multi!{
    HomogenizeExtension =>
        Vec4, DVec4,
        I16Vec4, IVec4, I64Vec4,
        U16Vec4, UVec4, U64Vec4
    => {
        fn homogenize(self) -> Self {
            let [x, y, z, w] = self.into();
            Self::new(x/w, y/w, z/w, w)
        }
    }
}


// types with no uninit bytes which fit into wgsl aligned types
mod padding_types {

    use std::{borrow::*, mem::transmute};
    use super::{*};

    #[derive(Debug, Clone, Copy, Default)]
    #[repr(C)]
    pub struct Vec3P {
        pub vec3: Vec3,
        _padding: f32,
    }

    impl<T: Into<Vec3>> From<T> for Vec3P {
        fn from(value: T) -> Self { Self { vec3: value.into(), _padding: 0.0 } }
    }

    impl Borrow<Vec3> for Vec3P { fn borrow(&self) -> &Vec3 { &self.vec3 } }
    impl BorrowMut<Vec3> for Vec3P { fn borrow_mut(&mut self) -> &mut Vec3 { &mut self.vec3 } }
    impl AsRef<Vec3> for Vec3P { fn as_ref(&self) -> &Vec3 { &self.vec3 } }
    impl AsMut<Vec3> for Vec3P { fn as_mut(&mut self) -> &mut Vec3 { &mut self.vec3 } }

    impl Into<Vec3A> for Vec3P { fn into(self) -> Vec3A { unsafe {transmute(self)} } }
    impl Borrow<Vec3A> for Vec3P { fn borrow(&self) -> &Vec3A { unsafe {transmute(self)} } }
    impl AsRef<Vec3A> for Vec3P { fn as_ref(&self) -> &Vec3A { unsafe {transmute(self)} } }


    #[derive(Debug, Clone, Copy, Default)]
    #[repr(C)]
    pub struct Mat3P {
        pub x_axis: Vec3P,
        pub y_axis: Vec3P,
        pub z_axis: Vec3P,
    }

    impl Mat3P {
        pub fn mat3(self) -> Mat3 {
            Mat3::from_cols(self.x_axis.vec3, self.y_axis.vec3, self.z_axis.vec3)
        }
    }

    impl<T: Into<Mat3>> From<T> for Mat3P {
        fn from(value: T) -> Self {
            let mat3 = value.into();
            Self {
                x_axis: mat3.x_axis.into(),
                y_axis: mat3.y_axis.into(),
                z_axis: mat3.z_axis.into(),
            }
        }
    }

    impl Into<Mat3A> for Mat3P { fn into(self) -> Mat3A { unsafe {transmute(self)} } }
    impl Borrow<Mat3A> for Mat3P { fn borrow(&self) -> &Mat3A { unsafe {transmute(self)} } }
    impl AsRef<Mat3A> for Mat3P { fn as_ref(&self) -> &Mat3A { unsafe {transmute(self)} } }
}

pub use padding_types::*;