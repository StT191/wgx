
// wgpu::Color drop in

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

// default
impl Default for Color {
    fn default() -> Self { Self::TRANSPARENT }
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

const F: f32 = 255.0;

impl Color {
    pub const fn new(r:f32, g:f32, b:f32, a:f32) -> Self { Self {r, g, b, a} }
    pub const fn new_rgb(r:f32, g:f32, b:f32) -> Self { Self {r, g, b, a: 1.0} }

    // into
    pub const fn f32(self) -> [f32; 4] { [self.r, self.g, self.b, self.a] }
    pub const fn f32_rgb(self) -> [f32; 3] { [self.r, self.g, self.b] }

    pub const fn f64(self) -> [f64; 4] { [self.r as f64, self.g as f64, self.b as f64, self.a as f64] }
    pub const fn f64_rgb(self) -> [f64; 3] { [self.r as f64, self.g as f64, self.b as f64] }

    pub fn u8(self) -> [u8; 4] { [(self.r*F) as u8, (self.g*F) as u8, (self.b*F) as u8, (self.a*F) as u8] }
    pub fn u8_rgb(self) -> [u8; 3] { [(self.r*F) as u8, (self.g*F) as u8, (self.b*F) as u8] }

    pub fn wgpu(self) -> wgpu::Color { wgpu::Color {r: self.r as f64, g: self.g as f64, b: self.b as f64, a: self.a as f64} }

    // from
    pub const fn from_f32([r, g, b, a]:[f32; 4]) -> Self { Self::new(r, g, b, a) }
    pub const fn from_f32_rgb([r, g, b]:[f32; 3]) -> Self { Self::new_rgb(r, g, b) }

    pub const fn from_f64([r, g, b, a]:[f64; 4]) -> Self { Self::new(r as f32, g as f32, b as f32, a as f32) }
    pub const fn from_f64_rgb([r, g, b]:[f64; 3]) -> Self { Self::new_rgb(r as f32, g as f32, b as f32) }

    pub fn from_u8([r, g, b, a]:[u8; 4]) -> Self { Self::new((r as f32)/F, (g as f32)/F, (b as f32)/F, (a as f32)/F) }
    pub fn from_u8_rgb([r, g, b]:[u8; 3]) -> Self { Self::new_rgb((r as f32)/F, (g as f32)/F, (b as f32)/F) }

    pub const fn from_wgpu(wgpu::Color {r, g, b, a}:wgpu::Color) -> Self { Self::from_f64([r, g, b, a]) }


    // format string
    pub fn hex(self) -> ArrayString<8> {
        let mut hex = ArrayString::new();
        let cl = self.u8();
        hex.write_fmt(format_args!("{:02x}{:02x}{:02x}{:02x}", cl[0], cl[1], cl[2], cl[3])).unwrap();
        hex
    }

    pub fn hex_rgb(self) -> ArrayString<6> {
        let mut hex = ArrayString::new();
        let cl = self.u8_rgb();
        hex.write_fmt(format_args!("{:02x}{:02x}{:02x}", cl[0], cl[1], cl[2])).unwrap();
        hex
    }

    // utils
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

    // palette
    pub const TRANSPARENT:Self = Color::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK:Self = Color::new_rgb(0.0, 0.0, 0.0);
    pub const WHITE:Self = Color::new_rgb(1.0, 1.0, 1.0);
    pub const RED:Self = Color::new_rgb(1.0, 0.0, 0.0);
    pub const GREEN:Self = Color::new_rgb(0.0, 1.0, 0.0);
    pub const BLUE:Self = Color::new_rgb(0.0, 0.0, 1.0);
    pub const YELLOW:Self = Color::new_rgb(1.0, 1.0, 0.0);
    pub const ORANGE:Self = Color::new_rgb(1.0, 0.5, 0.0);
    pub const TURKIS:Self = Color::new_rgb(0.0, 1.0, 1.0);
    pub const PURPLE:Self = Color::new_rgb(1.0, 0.0, 1.0);
    pub const GREY:Self = Color::new_rgb(0.5, 0.5, 0.5);
    pub const DARK_GREY:Self = Color::new_rgb(0.25, 0.25, 0.25);
    pub const LIGHT_GREY:Self = Color::new_rgb(0.75, 0.75, 0.75);
}


// to / from conversion

// f32
impl From<[f32; 4]> for Color {
    fn from(color_type: [f32; 4]) -> Color { Color::from_f32(color_type) }
}
impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self { color.f32() }
}
impl From<[f32; 3]> for Color {
    fn from(color_type: [f32; 3]) -> Color { Color::from_f32_rgb(color_type) }
}
impl From<Color> for [f32; 3] {
    fn from(color: Color) -> Self { color.f32_rgb() }
}

// f64
impl From<[f64; 4]> for Color {
    fn from(color_type: [f64; 4]) -> Color { Color::from_f64(color_type) }
}
impl From<Color> for [f64; 4] {
    fn from(color: Color) -> Self { color.f64() }
}
impl From<[f64; 3]> for Color {
    fn from(color_type: [f64; 3]) -> Color { Color::from_f64_rgb(color_type) }
}
impl From<Color> for [f64; 3] {
    fn from(color: Color) -> Self { color.f64_rgb() }
}

// u8
impl From<[u8; 4]> for Color {
    fn from(color_type: [u8; 4]) -> Color { Color::from_u8(color_type) }
}
impl From<Color> for [u8; 4] {
    fn from(color: Color) -> Self { color.u8() }
}
impl From<[u8; 3]> for Color {
    fn from(color_type: [u8; 3]) -> Color { Color::from_u8_rgb(color_type) }
}
impl From<Color> for [u8; 3] {
    fn from(color: Color) -> Self { color.u8_rgb() }
}

// wgpu
impl From<wgpu::Color> for Color {
    fn from(color_type: wgpu::Color) -> Color { Color::from_wgpu(color_type) }
}
impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self { color.wgpu() }
}
