
// wgpu::Color drop in

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

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

    pub const fn u8(self) -> [u8; 4] { [(self.r*F) as u8, (self.g*F) as u8, (self.b*F) as u8, (self.a*F) as u8] }
    pub const fn u8_rgb(self) -> [u8; 3] { [(self.r*F) as u8, (self.g*F) as u8, (self.b*F) as u8] }

    pub const fn packed_u32(self) -> u32 { u32::from_be_bytes(self.u8()) } // packed u32

    pub const fn wgpu(self) -> wgpu::Color { wgpu::Color {r: self.r as f64, g: self.g as f64, b: self.b as f64, a: self.a as f64} }

    // from
    pub const fn from_value_f32(v: f32) -> Self { Self::new_rgb(v, v, v) }
    pub const fn from_f32([r, g, b, a]:[f32; 4]) -> Self { Self::new(r, g, b, a) }
    pub const fn from_f32_rgb([r, g, b]:[f32; 3]) -> Self { Self::new_rgb(r, g, b) }

    pub const fn from_value_f64(v: f64) -> Self { Self::from_value_f32(v as f32) }
    pub const fn from_f64([r, g, b, a]:[f64; 4]) -> Self { Self::new(r as f32, g as f32, b as f32, a as f32) }
    pub const fn from_f64_rgb([r, g, b]:[f64; 3]) -> Self { Self::new_rgb(r as f32, g as f32, b as f32) }

    pub const fn from_value_u8(v: u8) -> Self { Self::from_value_f32((v as f32)/F) }
    pub const fn from_u8([r, g, b, a]:[u8; 4]) -> Self { Self::new((r as f32)/F, (g as f32)/F, (b as f32)/F, (a as f32)/F) }
    pub const fn from_u8_rgb([r, g, b]:[u8; 3]) -> Self { Self::new_rgb((r as f32)/F, (g as f32)/F, (b as f32)/F) }

    pub const fn from_packed_u32(c: u32) -> Self { Self::from_u8(c.to_be_bytes()) } // packed u32

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

    pub const fn interpolate(self, Self {r, g, b, a}: Self, factor: f32) -> Self {
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
impl From<f32> for Color {
    fn from(value: f32) -> Color { Color::from_value_f32(value) }
}
impl From<[f32; 4]> for Color {
    fn from(value: [f32; 4]) -> Color { Color::from_f32(value) }
}
impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self { color.f32() }
}
impl From<[f32; 3]> for Color {
    fn from(value: [f32; 3]) -> Color { Color::from_f32_rgb(value) }
}
impl From<Color> for [f32; 3] {
    fn from(color: Color) -> Self { color.f32_rgb() }
}

// f64
impl From<f64> for Color {
    fn from(value: f64) -> Color { Color::from_value_f64(value) }
}
impl From<[f64; 4]> for Color {
    fn from(value: [f64; 4]) -> Color { Color::from_f64(value) }
}
impl From<Color> for [f64; 4] {
    fn from(color: Color) -> Self { color.f64() }
}
impl From<[f64; 3]> for Color {
    fn from(value: [f64; 3]) -> Color { Color::from_f64_rgb(value) }
}
impl From<Color> for [f64; 3] {
    fn from(color: Color) -> Self { color.f64_rgb() }
}

// u8
impl From<u8> for Color {
    fn from(value: u8) -> Color { Color::from_value_u8(value) }
}
impl From<[u8; 4]> for Color {
    fn from(value: [u8; 4]) -> Color { Color::from_u8(value) }
}
impl From<Color> for [u8; 4] {
    fn from(color: Color) -> Self { color.u8() }
}
impl From<[u8; 3]> for Color {
    fn from(value: [u8; 3]) -> Color { Color::from_u8_rgb(value) }
}
impl From<Color> for [u8; 3] {
    fn from(color: Color) -> Self { color.u8_rgb() }
}

// packed
impl From<u32> for Color {
    fn from(packed: u32) -> Color { Color::from_packed_u32(packed) }
}
impl From<Color> for u32 {
    fn from(color: Color) -> Self { color.packed_u32() }
}

// wgpu
impl From<wgpu::Color> for Color {
    fn from(value: wgpu::Color) -> Color { Color::from_wgpu(value) }
}
impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self { color.wgpu() }
}


#[cfg(feature = "math")]
mod math_color_conversion {

    use super::Color;
    use crate::math::*;

    impl Color {

        pub const fn vec4(self) -> Vec4 { Vec4::from_array(self.f32()) }
        pub const fn from_vec4(vec4: Vec4) -> Self { Self::from_f32(vec4.to_array()) }

        pub const fn vec3(self) -> Vec3 { Vec3::from_array(self.f32_rgb()) }
        pub const fn vec3a(self) -> Vec3A { Vec3A::from_array(self.f32_rgb()) }
        pub const fn vec3p(self) -> Vec3P { Vec3P::new(Vec3::from_array(self.f32_rgb())) }
        pub const fn from_vec3(vec3: Vec3) -> Self { Self::from_f32_rgb(vec3.to_array()) }
        pub const fn from_vec3a(vec3a: Vec3A) -> Self { Self::from_f32_rgb(vec3a.to_array()) }
        pub const fn from_vec3p(vec3p: Vec3P) -> Self { Self::from_f32_rgb(vec3p.vec3().to_array()) }

        pub const fn dvec4(self) -> DVec4 { DVec4::from_array(self.f64()) }
        pub const fn from_dvec4(dvec4: DVec4) -> Self { Self::from_f64(dvec4.to_array()) }

        pub const fn dvec3(self) -> DVec3 { DVec3::from_array(self.f64_rgb()) }
        pub const fn from_dvec3(dvec3: DVec3) -> Self { Self::from_f64_rgb(dvec3.to_array()) }

    }

    impl From<Vec4> for Color {
        fn from(value: Vec4) -> Color { Color::from_vec4(value) }
    }
    impl From<Color> for Vec4 {
        fn from(color: Color) -> Self { color.vec4() }
    }

    impl From<Vec3> for Color {
        fn from(value: Vec3) -> Color { Color::from_vec3(value) }
    }
    impl From<Color> for Vec3 {
        fn from(color: Color) -> Self { color.vec3() }
    }
    impl From<Vec3A> for Color {
        fn from(value: Vec3A) -> Color { Color::from_vec3a(value) }
    }
    impl From<Color> for Vec3A {
        fn from(color: Color) -> Self { color.vec3a() }
    }
    impl From<Vec3P> for Color {
        fn from(value: Vec3P) -> Color { Color::from_vec3p(value) }
    }
    // from Color is genericly implemented for Vec3P

    impl From<DVec4> for Color {
        fn from(value: DVec4) -> Color { Color::from_dvec4(value) }
    }
    impl From<Color> for DVec4 {
        fn from(color: Color) -> Self { color.dvec4() }
    }

    impl From<DVec3> for Color {
        fn from(value: DVec3) -> Color { Color::from_dvec3(value) }
    }
    impl From<Color> for DVec3 {
        fn from(color: Color) -> Self { color.dvec3() }
    }

}


#[cfg(feature = "serde")]
mod color_serde_impl {

    use super::Color;
    use serde::{ser::*, de::{self, *}};

    impl Serialize for Color {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut state = serializer.serialize_tuple_struct("Color", 4)?;
            state.serialize_field(&self.r)?;
            state.serialize_field(&self.g)?;
            state.serialize_field(&self.b)?;
            state.serialize_field(&self.a)?;
            state.end()
        }
    }

    impl<'de> Deserialize<'de> for Color {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {

            struct ColorVisitor;

            impl<'de> Visitor<'de> for ColorVisitor {

                type Value = Color;

                fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str("a sequence of 4 f32 values")
                }

                fn visit_seq<V: SeqAccess<'de>>(self, mut seq: V) -> Result<Color, V::Error> {

                    let r = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                    let g = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                    let b = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
                    let a = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?;

                    Ok(Color::new(r, g, b, a))
                }
            }

            deserializer.deserialize_tuple_struct("Color", 3, ColorVisitor)
        }
    }


    #[cfg(test)]
    mod test {

        use super::Color;

        #[test]
        fn deserialize_json() {

            let json = "[0.6, 0.9, 0.3, 1.0]";

            let color: Color = match serde_json::from_str(json) {
                Ok(cl) => cl,
                Err(err) => panic!("{:?}", err),
            };

            assert_eq!(color, Color::new(0.6, 0.9, 0.3, 1.0));
        }

        #[test]
        fn serialize_json() {

            let color = Color::new(0.6, 0.9, 0.3, 1.0);

            let json: String = match serde_json::to_string(&color) {
                Ok(json) => json,
                Err(err) => panic!("{:?}", err),
            };

            assert_eq!(json, "[0.6,0.9,0.3,1.0]");
        }
    }

}