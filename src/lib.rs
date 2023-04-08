#![feature(div_duration, duration_constants, array_methods)]

// passs wgpu types
pub use wgpu::{
    self,
    BufferUsages as BufUse,
    TextureUsages as TexUse,
    PrimitiveTopology as Primitive,
    ShaderStages as Shader,
    Features, Limits,
    BlendState, BlendComponent, BlendFactor, BlendOperation,
    TextureDescriptor as TexDsc,
};

// macros
mod macros;
pub use macros::*;

// common types
mod read_bytes;
pub use read_bytes::*;

mod color;
pub use color::*;

// wgx
mod wgx;
pub use wgx::*;

mod encoder_extension;
pub use encoder_extension::*;

mod texture_extension;
pub use texture_extension::*;

mod render_target;
pub use render_target::*;

mod buffer_helper;
pub use buffer_helper::*;


// features

#[cfg(feature = "projection")]
pub use cgmath::{self, prelude::*};

#[cfg(feature = "projection")]
mod projection;
#[cfg(feature = "projection")]
pub use projection::*;


#[cfg(feature = "wav_obj")]
pub mod wav_obj;


#[cfg(feature = "timer")]
pub mod timer;


// glyph
#[cfg(feature = "glyph")]
pub use wgpu_glyph::{self, Text};

#[cfg(feature = "glyph")]
mod glyph_extension;
#[cfg(feature = "glyph")]
pub use glyph_extension::*;



// wgsl modules
#[cfg(feature = "wgsl_modules")]
pub use wgsl_modules::include as include_wgsl_module;

#[cfg(feature = "wgsl_modules_loader")]
pub use wgsl_modules::load as load_wgsl_module;


// error handling
pub mod error;


// control flow helper

pub trait ImplicitControlflow {
    fn should_continue(&self) -> bool;
}

impl ImplicitControlflow for () {
    fn should_continue(&self) -> bool { true }
}

impl<A, B> ImplicitControlflow for std::ops::ControlFlow<A, B> {
    fn should_continue(&self) -> bool { self.is_continue() }
}