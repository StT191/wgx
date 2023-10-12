
use std::slice;
use wgpu::{TextureFormat, TextureDimension, Extent3d};
use crate::TexUse;

// default Texture Formats

pub const TEXTURE: TextureFormat = TextureFormat::Rgba8UnormSrgb;
pub const TEXTURE_LINEAR: TextureFormat = TextureFormat::Rgba8Unorm;
pub const DEPTH: TextureFormat = TextureFormat::Depth32Float;


// extend Texture

pub trait TextureExtension<T> {
    fn create_default_view(&self) -> T;
}

impl TextureExtension<wgpu::TextureView> for wgpu::Texture {
    fn create_default_view(&self) -> wgpu::TextureView {
        self.create_view(&wgpu::TextureViewDescriptor::default())
    }
}


// our own TextureDescriptor
#[derive(Debug, Clone, Copy)]
pub struct TexDsc {
    pub label: Option<&'static str>,
    pub size: Extent3d,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub usage: TexUse,
}


impl TexDsc {
    pub fn new_2d(size: (u32, u32), sample_count: u32, format: TextureFormat, usage: TexUse) -> Self {
        Self {
            label: None, size: Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count, dimension: wgpu::TextureDimension::D2, format, usage,
        }
    }
    pub fn srgb(&self) -> bool { self.format.is_srgb() }
    pub fn size_2d(&self) -> (u32, u32) { (self.size.width, self.size.height) }
    pub fn set_size_2d(&mut self, (width, height): (u32, u32)) {
        self.size.width = width;
        self.size.height = height;
    }
}


// to / from TextureDescriptor
use wgpu::Label;

type TextureDescriptor<'a> = wgpu_types::TextureDescriptor<Label<'static>, &'a [TextureFormat]>;



impl<'a> From<&'a TexDsc> for TextureDescriptor<'a> {
    fn from(other: &'a TexDsc) -> TextureDescriptor<'a> {
        TextureDescriptor {
            label: other.label,
            size: other.size,
            mip_level_count: other.mip_level_count,
            sample_count: other.sample_count,
            dimension: other.dimension,
            format: other.format,
            usage: other.usage,
            view_formats: slice::from_ref(&other.format),
        }
    }
}


impl From<&TextureDescriptor<'_>> for TexDsc {
    fn from(other: &TextureDescriptor<'_>) -> TexDsc {
        TexDsc {
            label: other.label,
            size: other.size,
            mip_level_count: other.mip_level_count,
            sample_count: other.sample_count,
            dimension: other.dimension,
            format: other.format,
            usage: other.usage,
        }
    }
}

impl From<TextureDescriptor<'_>> for TexDsc {
    fn from(other: TextureDescriptor<'_>) -> TexDsc { (&other).into() }
}