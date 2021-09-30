
// passing external types
pub use wgpu::{
    BufferUsages as BuffUse, TextureUsages as TexUse, PrimitiveTopology as Primitive
};


// macros

mod macros;
pub use macros::*;


// common types

pub use cgmath::{prelude::*};

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


// extensions

#[cfg(feature = "spirv")]
pub use glsl_to_spirv::ShaderType;

#[cfg(feature = "glyph")]
pub use wgpu_glyph::Text;

#[cfg(feature = "glyph")]
mod glyph_extension;
#[cfg(feature = "glyph")]
pub use glyph_extension::*;


// extra modules

mod projection;
pub use projection::*;

mod text_input;
pub use text_input::*;


