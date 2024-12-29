
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
    fn from_uniform_scale(f: f32) -> Self { Self::from_scale([f; 3].into()) }
}

impl FromUniformScaleExtension for DMat4 {
    type F = f64;
    fn from_uniform_scale(f: f64) -> Self { Self::from_scale([f; 3].into()) }
}


pub trait FromToAttitudeExtension {
    type A;
    fn from_attitude(attitude: Self::A) -> Self;
    fn to_attitude(&self) -> Self::A;
}

pub const ATTITUDE: EulerRot = EulerRot::YXZEx; // yaw|pitch|roll

impl_multi!{
    FromToAttitudeExtension => Mat4, Mat3, Mat3A, Quat => {
        type A = Vec3;
        fn from_attitude(Vec3 {x:yaw, y:pitch, z:roll}: Vec3) -> Self {
            Self::from_euler(ATTITUDE, yaw, pitch, roll)
        }
        fn to_attitude(&self) -> Vec3 {
            let (yaw, pitch, roll) = self.to_euler(ATTITUDE);
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
            Self::from_euler(ATTITUDE, yaw, pitch, roll)
        }
        fn to_attitude(&self) -> DVec3 {
            let (yaw, pitch, roll) = self.to_euler(ATTITUDE);
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


pub trait AngleExtension: Sized {
    fn normalize_angle(self) -> Self;
    fn norm_angle_as_delta(self) -> Self;
    fn angle_as_delta(self) -> Self { Self::norm_angle_as_delta(Self::normalize_angle(self)) }
}

use std::f32::consts::PI as PI_32;
use std::f64::consts::PI as PI_64;

impl AngleExtension for f32 {
    fn normalize_angle(self) -> Self { self.rem_euclid(2.0 * PI_32) }
    fn norm_angle_as_delta(self) -> Self { if self > PI_32 { self - 2.0*PI_32 } else { self } }
}

impl AngleExtension for f64 {
    fn normalize_angle(self) -> Self { self.rem_euclid(2.0 * PI_64) }
    fn norm_angle_as_delta(self) -> Self { if self > PI_64 { self - 2.0*PI_64 } else { self } }
}

pub trait AnglesExtension: Sized {
    fn normalize_angles(self) -> Self;
    fn norm_angles_as_delta(self) -> Self;
    fn angles_as_delta(self) -> Self { Self::norm_angles_as_delta(Self::normalize_angles(self)) }
}

impl_multi!{
    AnglesExtension => Vec4, Vec3, Vec3A, Vec2 => {
        fn normalize_angles(self) -> Self { self.map(f32::normalize_angle) }
        fn norm_angles_as_delta(self) -> Self { self.map(f32::norm_angle_as_delta) }
    }
}

impl_multi!{
    AnglesExtension => DVec4, DVec3, DVec2 => {
        fn normalize_angles(self) -> Self { self.map(f64::normalize_angle) }
        fn norm_angles_as_delta(self) -> Self { self.map(f64::norm_angle_as_delta) }
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

#[cfg(feature = "mint")] use mint::{IntoMint, Vector3, Point3, ColumnMatrix3, RowMatrix3};
use std::borrow::*;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C, align(16))]
pub struct Vec3P { vec3: Vec3, _p: f32 }

impl Vec3P {
    pub const fn new(vec3: Vec3) -> Self { Self { vec3, _p: 0.0 } }
    pub const fn from_vec3a(vec3a: Vec3A) -> Self { Self::new(Vec3::from_array(vec3a.to_array())) }
    pub const fn vec3(self) -> Vec3 { self.vec3 }
    pub const fn vec3a(self) -> Vec3A { Vec3A::from_array(self.vec3.to_array()) }
}

impl<T> From<T> for Vec3P where Vec3: From<T> { fn from(value: T) -> Self { Self::new(value.into()) } }
impl Into<Vec3> for Vec3P { fn into(self) -> Vec3 { self.vec3 } }

impl Borrow<Vec3> for Vec3P { fn borrow(&self) -> &Vec3 { &self.vec3 } }
impl BorrowMut<Vec3> for Vec3P { fn borrow_mut(&mut self) -> &mut Vec3 { &mut self.vec3 } }

impl<T> AsRef<T> for Vec3P where Vec3: AsRef<T> { fn as_ref(&self) -> &T { self.vec3.as_ref() } }
impl<T> AsMut<T> for Vec3P where Vec3: AsMut<T> { fn as_mut(&mut self) -> &mut T { self.vec3.as_mut() } }

impl From<Vec3P> for Vec3A { fn from(vec3p: Vec3P) -> Self { vec3p.vec3a() } }

#[cfg(feature = "mint")] impl Into<Vector3<f32>> for Vec3P { fn into(self) -> Vector3<f32> { self.vec3.into() } }
#[cfg(feature = "mint")] impl IntoMint for Vec3P { type MintType = Vector3<f32>; }
#[cfg(feature = "mint")] impl From<Vec3P> for Point3<f32> { fn from(vec3p: Vec3P) -> Self { vec3p.vec3.into() } }

#[cfg(feature = "serde")]
mod vec3p_serde_impl {
    use super::*;
    use serde::*;
    impl Serialize for Vec3P {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            Vec3::serialize(&self.vec3, serializer)
        }
    }
    impl<'de> Deserialize<'de> for Vec3P {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Vec3::deserialize(deserializer).map(Self::new)
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat3P {
    pub x_axis: Vec3P,
    pub y_axis: Vec3P,
    pub z_axis: Vec3P,
}

impl Default for Mat3P { fn default() -> Self { Self::from_mat3a(Mat3A::IDENTITY) } }

impl Mat3P {

    pub const fn new(mat3: Mat3) -> Self {
        Self {
            x_axis: Vec3P::new(mat3.x_axis),
            y_axis: Vec3P::new(mat3.y_axis),
            z_axis: Vec3P::new(mat3.z_axis),
        }
    }

    pub const fn from_mat3a(mat3a: Mat3A) -> Self {
        Self {
            x_axis: Vec3P::from_vec3a(mat3a.x_axis),
            y_axis: Vec3P::from_vec3a(mat3a.y_axis),
            z_axis: Vec3P::from_vec3a(mat3a.z_axis),
        }
    }

    pub const fn mat3(self) -> Mat3 {
        Mat3::from_cols(self.x_axis.vec3, self.y_axis.vec3, self.z_axis.vec3)
    }

    pub const fn mat3a(self) -> Mat3A {
        Mat3A::from_cols(self.x_axis.vec3a(), self.y_axis.vec3a(), self.z_axis.vec3a())
    }
}

impl<T> From<T> for Mat3P where Mat3A: From<T> { fn from(value: T) -> Self { Self::from_mat3a(value.into()) } }
impl Into<Mat3A> for Mat3P { fn into(self) -> Mat3A { self.mat3a() } }

impl From<Mat3P> for Mat3 { fn from(mat3p: Mat3P) -> Self { mat3p.mat3() } }

#[cfg(feature = "mint")] impl From<Mat3P> for ColumnMatrix3<f32> { fn from(mat3p: Mat3P) -> Self { mat3p.mat3().into() } }
#[cfg(feature = "mint")] impl From<Mat3P> for RowMatrix3<f32> { fn from(mat3p: Mat3P) -> Self { mat3p.mat3().into() } }
#[cfg(feature = "mint")] impl IntoMint for Mat3P { type MintType = ColumnMatrix3<f32>; }

#[cfg(feature = "serde")]
mod mat3p_serde_impl {
    use super::*;
    use serde::*;
    impl Serialize for Mat3P {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            Mat3A::serialize(&self.mat3a(), serializer)
        }
    }
    impl<'de> Deserialize<'de> for Mat3P {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Mat3A::deserialize(deserializer).map(Self::from_mat3a)
        }
    }
}