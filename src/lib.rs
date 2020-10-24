
// all imports

pub use glsl_to_spirv::ShaderType;

pub use wgpu::{
    BufferUsage as BuffUse, TextureUsage as TexUse, PrimitiveTopology as Primitive
};


// pub mod as_byte_slice;
pub mod byte_slice;


mod macros;
pub use macros::*;


mod gx;
pub use gx::*; // all wgpu handling

/*
mod vec4;
pub use vec4::*;*/

mod color;
pub use color::*;




/*mod refs;
pub use refs::*;*/