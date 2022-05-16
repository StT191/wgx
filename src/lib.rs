
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


// features

#[cfg(feature = "projection")]
pub use cgmath::{self, prelude::*};

#[cfg(feature = "projection")]
mod projection;
#[cfg(feature = "projection")]
pub use projection::*;


#[cfg(feature = "wav_obj")]
pub mod wav_obj;


#[cfg(feature = "frames")]
pub mod frames;


// glyph
#[cfg(feature = "glyph")]
pub use wgpu_glyph::{self, Text};

#[cfg(feature = "glyph")]
mod glyph_extension;
#[cfg(feature = "glyph")]
pub use glyph_extension::*;

#[cfg(feature = "simple_text_input")]
mod text_input;
#[cfg(feature = "simple_text_input")]
pub use text_input::*;


// iced
#[cfg(feature = "iced")]
mod iced;
#[cfg(feature = "iced")]
pub use iced::Iced;
#[cfg(feature = "iced")]
pub use iced_wgpu;
#[cfg(feature = "iced")]
pub use iced_winit;



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
