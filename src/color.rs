

// wgpu::Color

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color { pub r: f64, pub g: f64, pub b: f64, pub a: f64 }

// From

impl From<(f64, f64, f64, f64)> for Color {
    fn from((r, g, b, a):(f64, f64, f64, f64)) -> Self { Self { r, g, b, a } }
}
impl From<(f64, f64, f64)> for Color {
    fn from((r, g, b):(f64, f64, f64)) -> Self { Self { r, g, b, a: 1.0 } }
}


impl From<(f32, f32, f32, f32)> for Color {
    fn from((r, g, b, a):(f32, f32, f32, f32)) -> Self { Self::from((r as f64, g as f64, b as f64, a as f64)) }
}
impl From<(f32, f32, f32)> for Color {
    fn from((r, g, b):(f32, f32, f32)) -> Self { Self::from((r, g, b, 1.0)) }
}


const F:f64 = 255.0;

impl From<(u8, u8, u8, u8)> for Color {
    fn from((r, g, b, a):(u8, u8, u8, u8)) -> Self { Self::from(((r as f64)/F, (g as f64)/F, (b as f64)/F, (a as f64)/F)) }
}
impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b):(u8, u8, u8)) -> Self { Self::from((r, g, b, 255)) }
}


impl From<wgpu::Color> for Color {
    fn from(wgpu::Color {r, g, b, a}:wgpu::Color) -> Self { Self {r, g, b, a} }
}


// Into

impl Into<(f64, f64, f64, f64)> for Color {
    fn into(self) -> (f64, f64, f64, f64) { (self.r, self.g, self.g, self.a) }
}
impl Into<(f64, f64, f64)> for Color {
    fn into(self) -> (f64, f64, f64) { (self.r, self.g, self.g) }
}


impl Into<(f32, f32, f32, f32)> for Color {
    fn into(self) -> (f32, f32, f32, f32) { (self.r as f32, self.g as f32, self.g as f32, self.a as f32) }
}
impl Into<(f32, f32, f32)> for Color {
    fn into(self) -> (f32, f32, f32) { (self.r as f32, self.g as f32, self.g as f32) }
}


impl Into<(u8, u8, u8, u8)> for Color {
    fn into(self) -> (u8, u8, u8, u8) { ((F*self.r) as u8, (F*self.g) as u8, (F*self.b) as u8, (F*self.a) as u8) }
}
impl Into<(u8, u8, u8)> for Color {
    fn into(self) -> (u8, u8, u8) { ((F*self.r) as u8, (F*self.g) as u8, (F*self.b) as u8) }
}


impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color { wgpu::Color {r: self.r, g: self.g, b: self.b, a: self.a} }
}



impl Color {
    pub fn f64(self) -> (f64, f64, f64, f64) { self.into() }
    pub fn f64_rgb(self) -> (f64, f64, f64) { self.into() }

    pub fn f32(self) -> (f32, f32, f32, f32) { self.into() }
    pub fn f32_rgb(self) -> (f32, f32, f32) { self.into() }

    pub fn u8(self) -> (u8, u8, u8, u8) { self.into() }
    pub fn u8_rgb(self) -> (u8, u8, u8) { self.into() }

    pub fn wgpu(self) -> wgpu::Color { self.into() }


    pub const TRANSPARENT: Self = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const BLACK: Self = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Self = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED: Self = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Self = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Self = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };

}