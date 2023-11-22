
use std::mem::zeroed;
use crate::error::*;
use cgmath::*;

// cast helper
pub fn cast_unwrap<U: num_traits::NumCast>(n: f64) -> U { num_traits::cast(n).unwrap() }
use cast_unwrap as c;


// projections
#[derive(Debug, Clone)]
pub struct FovProjection<S> {
    pub fov: PerspectiveFov<S>,
    pub projection: Matrix4<S>,
    pub distance: S,
    pub translation: Matrix4<S>,
}


impl<S:BaseFloat> FovProjection<S> {

    pub fn update(&mut self) {
        self.projection = Matrix4::from(self.fov) * Matrix4::from_nonuniform_scale(c(1.0), c(1.0), c(-1.0));
        self.translation = Matrix4::from_translation((c(0.0), c(0.0), self.distance).into());
    }

    pub fn new(fov_deg:S, aspect:S, near:S, far:S, distance:S) -> Self {
        let mut this = Self {
            fov: PerspectiveFov { fovy: Deg(fov_deg).into(), aspect, near, far },
            distance,
            // SAFETY: types are valid when zeroed
            projection: unsafe { zeroed() }, translation: unsafe { zeroed() },
        };
        this.update();
        this
    }

    pub fn unit(fov_deg:S, aspect:S, unit:S) -> Self {
        let near = unit / c(1.0e3);
        let far = unit * c(2.0e3);
        let distance = unit / Deg(fov_deg/c(2.0)).tan();
        Self::new(fov_deg, aspect, near, far, distance)
    }

    pub fn window(fov_deg:S, width:S, height:S) -> Self {
        let unit = S::max(width, height);
        let near = unit / c(1.0e3);
        let far = unit * c(2.0e3);
        let distance = height * c(0.5) / Deg(fov_deg/c(2.0)).tan();
        Self::new(fov_deg, width/height, near, far, distance)
    }

    pub fn resize_window(&mut self, width: S, height: S, update_distances: bool) {

        let fov_deg = Deg::from(self.fov.fovy).0;

        if update_distances {
            *self = FovProjection::window(fov_deg, width, height);
        } else {
            // or aspect only
            self.fov.aspect = width/height;
            self.update();
        };
    }
}


pub fn flat_window_projection<S:BaseFloat>(width:S, height:S, depth:S) -> Matrix4<S> {
    Matrix4::from_translation((c(-1.0), c(1.0), c(0.0)).into()) *
    Matrix4::from_nonuniform_scale(
        c::<S>(2.0)/width,
        c::<S>(-2.0)/height,
        if depth == c(0.0) { c(0.0) } else { c::<S>(1.0)/depth},
    )
}



// utilities

#[macro_export]
macro_rules! apply {
    ($matrix:expr, $value:expr) => {
        $matrix = $value * $matrix
    }
}


// in left handed coordinate system
pub fn normal_from_triangle<S:BaseFloat, V: Into<Vector3<S>> + Copy >(v0:V, v1:V, v2:V) -> Vector3<S> {
    (v1.into() - v0.into()).cross(v2.into() - v0.into()).normalize()
}



pub trait Vector4Extension {
    fn homogenize(self) -> Self;
}

impl<S:BaseFloat> Vector4Extension for Vector4<S> {

    fn homogenize(self) -> Self {
        Self { x: self.x/self.w, y: self.y/self.w, z: self.z/self.w, w: self.w/self.w }
    }
}



pub trait SquareMatrixExtension: SquareMatrix
    where <Self as VectorSpace>::Scalar: num_traits::Float
{
    fn within(&self, outer:&Self) -> Res<Self>;
}

use std::ops::Mul;

impl<T: SquareMatrix> SquareMatrixExtension for T
    where
        <Self as cgmath::VectorSpace>::Scalar: num_traits::Float,
        Self: for<'a> Mul<&'a Self, Output = Self>,
{
    fn within(&self, outer:&Self) -> Res<Self> {
        Ok(outer.invert().ok_or("couldn't invert matrix")? * self * outer)
    }
}