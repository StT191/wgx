
use std::slice;
use wgpu::{TextureFormat, TextureDimension, TextureViewDescriptor, Extent3d};
use crate::*;

// default Texture Formats

pub const DEFAULT_SRGB: TextureFormat = TextureFormat::Rgba8UnormSrgb;
pub const DEFAULT_LINEAR: TextureFormat = TextureFormat::Rgba8Unorm;
pub const DEFAULT_DEPTH: TextureFormat = TextureFormat::Depth32Float;


// extend Texture

pub trait TextureExtension<T> {
    fn create_default_view(&self, format: Option<TextureFormat>) -> T;
    fn tex_dsc(&self) -> TexDsc;
}

impl TextureExtension<wgpu::TextureView> for wgpu::Texture {
    fn create_default_view(&self, format: Option<TextureFormat>) -> wgpu::TextureView {
        self.create_view(&TextureViewDescriptor {
            format, ..TextureViewDescriptor::default()
        })
    }
    fn tex_dsc(&self) -> TexDsc { TexDsc {
        label: None, format: self.format(), view_format: self.format(), usage: self.usage(),
        size: self.size(), mip_level_count: self.mip_level_count(),
        sample_count: self.sample_count(), dimension: self.dimension(),
    } }
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
    pub view_format: TextureFormat,
    pub usage: TexUse,
}


impl TexDsc {
    pub fn new_2d(
        size: (u32, u32), sample_count: u32,
        format: TextureFormat, view_format: Option<TextureFormat>, usage: TexUse
    ) -> Self {
        Self {
            label: None, size: Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count, dimension: wgpu::TextureDimension::D2,
            format, view_format: view_format.unwrap_or(format), usage,
        }
    }
    pub fn srgb(&self) -> bool { self.view_format.is_srgb() }
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
    fn from(dsc: &'a TexDsc) -> TextureDescriptor<'a> {
        TextureDescriptor {
            label: dsc.label,
            size: dsc.size,
            mip_level_count: dsc.mip_level_count,
            sample_count: dsc.sample_count,
            dimension: dsc.dimension,
            format: dsc.format,
            usage: dsc.usage,
            view_formats: slice::from_ref(&dsc.view_format),
        }
    }
}


impl From<&TextureDescriptor<'_>> for TexDsc {
    fn from(tdsc: &TextureDescriptor<'_>) -> TexDsc {
        TexDsc {
            label: tdsc.label,
            size: tdsc.size,
            mip_level_count: tdsc.mip_level_count,
            sample_count: tdsc.sample_count,
            dimension: tdsc.dimension,
            format: tdsc.format,
            view_format: *tdsc.view_formats.get(0).unwrap_or(&tdsc.format),
            usage: tdsc.usage,
        }
    }
}

impl From<TextureDescriptor<'_>> for TexDsc {
    fn from(tdsc: TextureDescriptor<'_>) -> TexDsc { (&tdsc).into() }
}