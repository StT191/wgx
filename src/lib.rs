#![feature(iterator_try_collect, linked_list_cursors, iter_intersperse)]

// passs wgpu types
pub use wgpu::{
    self,
    BufferUsages as BufUse,
    TextureUsages as TexUse,
    PrimitiveTopology as Primitive,
    ShaderStages as Shader,
    Features, Limits,
};

// macros
mod macros;
pub use macros::*;

// common types
mod byte_slice;

mod color;
pub use color::*;

// wgx
mod wgpu_extensions;
pub use wgpu_extensions::*;

mod wgx;
pub use wgx::*;

mod render_target;
pub use render_target::*;

mod vertex_buffer;
pub use vertex_buffer::*;


// features

#[cfg(feature = "projection")]
pub use cgmath::{self, prelude::*};

#[cfg(feature = "projection")]
mod projection;
#[cfg(feature = "projection")]
pub use projection::*;


#[cfg(feature = "wav_obj")]
pub mod wav_obj;


#[cfg(feature = "ticks")]
pub mod ticks;


// glyph
#[cfg(feature = "glyph")]
pub use wgpu_glyph::{self, Text};

#[cfg(feature = "glyph")]
mod glyph_extension;
#[cfg(feature = "glyph")]
pub use glyph_extension::*;


// iced
#[cfg(feature = "iced")]
mod iced;
#[cfg(feature = "iced")]
pub use iced::Iced;
#[cfg(feature = "iced")]
pub use iced_wgpu;
#[cfg(feature = "iced")]
pub use iced_winit;


// wgsl modules
#[cfg(feature = "wgsl_modules")]
pub use wgsl_modules::{
    load as load_wgsl_module,
    include as include_wgsl_module,
};


// error handling

pub mod error {
    // Results and error Handling
    pub type Error = String;

    // map most errors to Error
    pub fn error(err: impl std::fmt::Display) -> Error {
        err.to_string()
    }

    pub type Res<T> = Result<T, Error>;
}
