
// wgpu::Color drop in

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

// From

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]:[f32; 4]) -> Self { Self {r, g, b, a} }
}
impl From<[f32; 3]> for Color {
    fn from([r, g, b]:[f32; 3]) -> Self { Self::from([r, g, b, 1.0]) }
}

impl From<[f64; 4]> for Color {
    fn from([r, g, b, a]:[f64; 4]) -> Self { Self::new((r as f32).into(), (g as f32).into(), (b as f32).into(), (a as f32).into()) }
}
impl From<[f64; 3]> for Color {
    fn from([r, g, b]:[f64; 3]) -> Self { Self::from([r, g, b, 1.0]) }
}



const F:f32 = 255.0;

impl From<[u8; 4]> for Color {
    fn from([r, g, b, a]:[u8; 4]) -> Self { Self::new((r as f32)/F, (g as f32)/F, (b as f32)/F, (a as f32)/F) }
}
impl From<[u8; 3]> for Color {
    fn from([r, g, b]:[u8; 3]) -> Self { Self::from([r, g, b, 255]) }
}


impl From<wgpu::Color> for Color {
    fn from(wgpu::Color {r, g, b, a}:wgpu::Color) -> Self { Self::from([r, g, b, a]) }
}


#[cfg(feature = "iced")]
impl From<iced_wgpu::Color> for Color {
    fn from(iced_wgpu::Color {r, g, b, a}:iced_wgpu::Color) -> Self { Self {r, g, b, a} }
}


// Into

impl Into<[f64; 4]> for Color {
    fn into(self) -> [f64; 4] { [self.r as f64, self.g as f64, self.b as f64, self.a as f64] }
}
impl Into<[f64; 3]> for Color {
    fn into(self) -> [f64; 3] { [self.r as f64, self.g as f64, self.b as f64] }
}


impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] { [self.r, self.g, self.b, self.a] }
}
impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] { [self.r, self.g, self.b] }
}


impl Into<[u8; 4]> for Color {
    fn into(self) -> [u8; 4] { [(F*self.r) as u8, (F*self.g) as u8, (F*self.b) as u8, (F*self.a) as u8] }
}
impl Into<[u8; 3]> for Color {
    fn into(self) -> [u8; 3] { [(F*self.r) as u8, (F*self.g) as u8, (F*self.b) as u8] }
}


impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color { wgpu::Color {r: self.r as f64, g: self.g as f64, b: self.b as f64, a: self.a as f64} }
}


#[cfg(feature = "iced")]
impl Into<iced_wgpu::Color> for Color {
    fn into(self) -> iced_wgpu::Color { iced_wgpu::Color {r: self.r, g: self.g, b: self.b, a: self.a} }
}


// color channel srgb to linear
fn linear_component(u: f32) -> f32 {
    if u < 0.04045 {
        u / 12.92
    } else {
        ((u + 0.055) / 1.055).powf(2.4)
    }
}

// color channel linear to srgb
fn srgb_component(u: f32) -> f32 {
    if u < 0.0031308 {
        u * 12.92
    } else {
        u.powf(1.0 / 2.4) * 1.055 - 0.055
    }
}

use arrayvec::ArrayString;
use std::fmt::Write;

impl Color {
    pub fn new(r:f32, g:f32, b:f32, a:f32) -> Self { Self {r, g, b, a} }

    pub fn f32(self) -> [f32; 4] { self.into() }
    pub fn f32_rgb(self) -> [f32; 3] { self.into() }

    pub fn f64(self) -> [f64; 4] { self.into() }
    pub fn f64_rgb(self) -> [f64; 3] { self.into() }

    pub fn u8(self) -> [u8; 4] { self.into() }
    pub fn u8_rgb(self) -> [u8; 3] { self.into() }

    pub fn hex(self) -> ArrayString<9> {
        let mut hex = ArrayString::new();
        let cl = self.u8();
        hex.write_fmt(format_args!("#{:02x}{:02x}{:02x}{:02x}", cl[0], cl[1], cl[2], cl[3])).unwrap();
        hex
    }

    pub fn hex_rgb(self) -> ArrayString<7> {
        let mut hex = ArrayString::new();
        let cl = self.u8_rgb();
        hex.write_fmt(format_args!("#{:02x}{:02x}{:02x}", cl[0], cl[1], cl[2])).unwrap();
        hex
    }

    pub fn linear(self) -> Self {
        Self {
            r: linear_component(self.r),
            g: linear_component(self.g),
            b: linear_component(self.b),
            a: self.a,
        }
    }

    pub fn srgb(self) -> Self {
        Self {
            r: srgb_component(self.r),
            g: srgb_component(self.g),
            b: srgb_component(self.b),
            a: self.a,
        }
    }

    pub fn interpolate(self, Self {r, g, b, a}: Self, factor: f32) -> Self {
        Self {
            r: self.r + (r - self.r) * factor,
            g: self.g + (g - self.g) * factor,
            b: self.b + (b - self.b) * factor,
            a: self.a + (a - self.a) * factor,
        }
    }

    pub fn wgpu(self) -> wgpu::Color { self.into() }

    #[cfg(feature = "iced")]
    pub fn iced(self) -> iced_wgpu::Color { self.into() }

    pub const TRANSPARENT:Self = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const BLACK:Self = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE:Self = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED:Self = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN:Self = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE:Self = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const YELLOW:Self = Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const ORANGE:Self = Color { r: 1.0, g: 0.5, b: 0.0, a: 1.0 };
    pub const TURKIS:Self = Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const PURPLE:Self = Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const GREY:Self = Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
    pub const DARK_GREY:Self = Color { r: 0.25, g: 0.25, b: 0.25, a: 1.0 };
    pub const LIGHT_GREY:Self = Color { r: 0.75, g: 0.75, b: 0.75, a: 1.0 };
}