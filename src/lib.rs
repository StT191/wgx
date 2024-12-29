#![feature(step_trait)]

// passs wgpu types
pub use wgpu::{
    self,
    Features, Limits, PresentMode,
    Buffer, Texture, TextureView,
    BufferUsages as BufUse,
    TextureUsages as TexUse,
    ShaderStages as Stage,
    TextureFormat as TexFmt,
    TextureDimension as Dimension,
    TextureViewDimension as ViewDimension,
    BlendState as Blend, BlendComponent, BlendFactor, BlendOperation,
    MapMode, // buffer mapping
    PrimitiveState as Primitive,
    PrimitiveTopology as Topology,
    IndexFormat, Face, FrontFace, PolygonMode as Polygon,
    util::{StagingBelt, DrawIndirectArgs, DrawIndexedIndirectArgs, DispatchIndirectArgs, RenderEncoder},
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

mod texture_extension;
pub use texture_extension::*;

mod render_target;
pub use render_target::*;

mod util_extension;
pub use util_extension::*;

mod staging_extension;
pub use staging_extension::*;

mod buffer_helper;
pub use buffer_helper::*;


// features

#[cfg(feature = "math")]
pub mod math;


// wgsl modules
#[cfg(feature = "wgsl_modules")]
pub use wgsl_modules;


// control flow helper

pub trait ImplicitControlFlow {
    fn should_continue(&self) -> bool;
}

impl ImplicitControlFlow for () {
    fn should_continue(&self) -> bool { true }
}

impl<B, C> ImplicitControlFlow for std::ops::ControlFlow<B, C> {
    fn should_continue(&self) -> bool { self.is_continue() }
}


// shader_constants
use std::collections::HashMap;

pub type ShaderConstants = HashMap<String, f64>;