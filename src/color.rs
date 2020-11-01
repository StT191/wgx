
// wgpu::Color drop in

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color { pub r: f64, pub g: f64, pub b: f64, pub a: f64 }

// From

impl From<[f64; 4]> for Color {
    fn from([r, g, b, a]:[f64; 4]) -> Self { Self { r, g, b, a } }
}
impl From<[f64; 3]> for Color {
    fn from([r, g, b]:[f64; 3]) -> Self { Self { r, g, b, a: 1.0 } }
}


impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]:[f32; 4]) -> Self { Self::new(r as f64, g as f64, b as f64, a as f64) }
}
impl From<[f32; 3]> for Color {
    fn from([r, g, b]:[f32; 3]) -> Self { Self::from([r, g, b, 1.0]) }
}


const F:f64 = 255.0;

impl From<[u8; 4]> for Color {
    fn from([r, g, b, a]:[u8; 4]) -> Self { Self::new((r as f64)/F, (g as f64)/F, (b as f64)/F, (a as f64)/F) }
}
impl From<[u8; 3]> for Color {
    fn from([r, g, b]:[u8; 3]) -> Self { Self::from([r, g, b, 255]) }
}


impl From<wgpu::Color> for Color {
    fn from(wgpu::Color {r, g, b, a}:wgpu::Color) -> Self { Self {r, g, b, a} }
}


// Into

impl Into<[f64; 4]> for Color {
    fn into(self) -> [f64; 4] { [self.r, self.g, self.b, self.a] }
}
impl Into<[f64; 3]> for Color {
    fn into(self) -> [f64; 3] { [self.r, self.g, self.b] }
}


impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] { [self.r as f32, self.g as f32, self.b as f32, self.a as f32] }
}
impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] { [self.r as f32, self.g as f32, self.b as f32] }
}


impl Into<[u8; 4]> for Color {
    fn into(self) -> [u8; 4] { [(F*self.r) as u8, (F*self.g) as u8, (F*self.b) as u8, (F*self.a) as u8] }
}
impl Into<[u8; 3]> for Color {
    fn into(self) -> [u8; 3] { [(F*self.r) as u8, (F*self.g) as u8, (F*self.b) as u8] }
}


impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color { wgpu::Color {r: self.r, g: self.g, b: self.b, a: self.a} }
}



impl Color {
    pub fn new(r:f64, g:f64, b:f64, a:f64) -> Self { Self {r, g, b, a} }

    pub fn f64(self) -> [f64; 4] { self.into() }
    pub fn f64_rgb(self) -> [f64; 3] { self.into() }

    pub fn f32(self) -> [f32; 4] { self.into() }
    pub fn f32_rgb(self) -> [f32; 3] { self.into() }

    pub fn u8(self) -> [u8; 4] { self.into() }
    pub fn u8_rgb(self) -> [u8; 3] { self.into() }

    pub fn wgpu(self) -> wgpu::Color { self.into() }


    pub const TRANSPARENT: Self = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const BLACK: Self = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Self = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED: Self = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Self = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Self = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const YELLOW: Self = Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const ORANGE: Self = Color { r: 1.0, g: 0.5, b: 0.0, a: 1.0 };
    pub const TURKIS: Self = Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const PURPLE: Self = Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };

}