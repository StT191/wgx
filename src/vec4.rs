

// Vec4

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vec4 (f64, f64, f64, f64);


impl Vec4 {
    pub fn f64(self) -> (f64, f64, f64, f64) { self.into() }
    pub fn f32(self) -> (f32, f32, f32, f32) { (self.0 as f32, self.1 as f32, self.2 as f32, self.3 as f32) }
}


// From

impl<T: Into<f64>> From<(T, T, T, T)> for Vec4 {
    fn from((x, y, z, v):(T, T, T, T)) -> Self { Self (x.into(), y.into(), z.into(), v.into()) }
}
impl<T: Into<f64>> From<(T, T, T)> for Vec4 {
    fn from((x, y, z):(T, T, T)) -> Self { Self (x.into(), y.into(), z.into(), 0.0) }
}
impl<T: Into<f64>> From<(T, T)> for Vec4 {
    fn from((x, y):(T, T)) -> Self { Self (x.into(), y.into(), 0.0, 0.0) }
}


impl<T: From<f64>> Into<(T, T, T, T)> for Vec4 {
    fn into(self) -> (T, T, T, T) {
        (self.0.into(), self.1.into(), self.2.into(), self.3.into())
    }
}
impl<T: From<f64>> Into<(T, T, T)> for Vec4 {
    fn into(self) -> (T, T, T) {
        (self.0.into(), self.1.into(), self.2.into())
    }
}
impl<T: From<f64>> Into<(T, T)> for Vec4 {
    fn into(self) -> (T, T) {
        (self.0.into(), self.1.into())
    }
}