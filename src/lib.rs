
// passing external types
pub use wgpu::{
    BufferUsages as BuffUse,
    TextureUsages as TexUse,
    PrimitiveTopology as Primitive,
    ShaderStages as Shader,
    Features,
};


// macros

mod macros;
pub use macros::*;


// common types

pub use cgmath::prelude::*;

mod projection;
pub use projection::*;

mod byte_slice;

mod color;
pub use color::*;

pub mod refs;


// wgx

mod wgpu_extensions;
pub use wgpu_extensions::*;

mod wgx;
pub use wgx::*;

mod render_target;
pub use render_target::*;


// spirv
#[cfg(feature = "spirv")]
pub use glsl_to_spirv::ShaderType;


// glyph-extemsion
#[cfg(feature = "glyph")]
pub use wgpu_glyph::Text;

#[cfg(feature = "glyph")]
mod glyph_extension;
#[cfg(feature = "glyph")]
pub use glyph_extension::*;

#[cfg(feature = "glyph")]
mod text_input;
#[cfg(feature = "glyph")]
pub use text_input::*;


// iced
#[cfg(feature = "iced")]
mod iced;
#[cfg(feature = "iced")]
pub use iced::Iced;







