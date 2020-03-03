
use zerocopy::{FromBytes, AsBytes};


// Color
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, FromBytes, AsBytes)]
#[repr(C)]
pub struct Color { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }


impl<T: Into<u8>> From<(T, T, T)> for Color {
    fn from((r, g, b):(T, T, T)) -> Self { Self { r: r.into(), g: g.into(), b: b.into(), a: 255 } }
}
impl<T: Into<u8>> From<(T, T, T, T)> for Color{
    fn from((r, g, b, a):(T, T, T, T)) -> Self { Self { r: r.into(), g: g.into(), b: b.into(), a: a.into() } }
}

impl<T: From<u8>> Into<(T, T, T)> for Color {
    fn into(self) -> (T, T, T) { (self.r.into(), self.g.into(), self.b.into()) }
}
impl<T: From<u8>> Into<(T, T, T, T)> for Color {
    fn into(self) -> (T, T, T, T) { (self.r.into(), self.g.into(), self.b.into(), self.a.into()) }
}

impl Color {
    pub fn rgb(self) -> (u8, u8, u8) { (self.r, self.g, self.b) }
    pub fn rgba(self) -> (u8, u8, u8, u8) { (self.r, self.g, self.b, self.a) }
}