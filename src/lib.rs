
// passs wgpu types
pub use wgpu::{
    self,
    Features, Limits,
    BufferUsages as BufUse,
    TextureUsages as TexUse,
    ShaderStages as Stage,
    BlendState as Blend, BlendComponent, BlendFactor, BlendOperation,
    MapMode, // buffer mapping
    PrimitiveState as Primitive,
    PrimitiveTopology as Topology,
    IndexFormat, Face, FrontFace, PolygonMode as Polygon,
    util::{StagingBelt, DrawIndirect, DrawIndexedIndirect, DispatchIndirect},
};

// macros
mod macros;

// common types
mod read_bytes;
pub use read_bytes::*;

mod color;
pub use color::*;

// wgx
mod wgx;
pub use wgx::*;

mod render_extension;
pub use render_extension::*;

mod buffer_extension;
pub use buffer_extension::*;

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