
use std::slice;
use wgpu::{TextureFormat, TextureDimension, TextureViewDimension, TextureViewDescriptor, TextureAspect};
use crate::*;

// default Texture Formats

pub const DEFAULT_SRGB: TextureFormat = TextureFormat::Rgba8UnormSrgb;
pub const DEFAULT_LINEAR: TextureFormat = TextureFormat::Rgba8Unorm;
pub const DEFAULT_DEPTH: TextureFormat = TextureFormat::Depth32Float;


// extend Texture
fn dimension_to_view(dim: TextureDimension, depth: u32) -> TextureViewDimension {
    match dim {
        TextureDimension::D1 => TextureViewDimension::D1,
        TextureDimension::D2 => if depth == 1 { TextureViewDimension::D2 } else { TextureViewDimension::D2Array },
        TextureDimension::D3 => TextureViewDimension::D3,
    }
}

fn view_to_dimension(view: TextureViewDimension) -> TextureDimension {
    match view {
        TextureViewDimension::D1 => TextureDimension::D1,
        TextureViewDimension::D3 => TextureDimension::D3,
        _ => TextureDimension::D2,
    }
}


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
    fn tex_dsc(&self) -> TexDsc {
        let size = self.size().to_arr();
        TexDsc {
            label: None, format: self.format(), view_format: self.format(), usage: self.usage(),
            size, mip_level_count: self.mip_level_count(), sample_count: self.sample_count(),
            view_dimension: dimension_to_view(self.dimension(), size[2]),
            view_aspect: TextureAspect::All,
        }
    }
}


// our own TextureDescriptor
#[derive(Debug, Clone, Copy)]
pub struct TexDsc {
    pub label: Option<&'static str>,
    pub size: [u32; 3],
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub view_dimension: TextureViewDimension,
    pub view_aspect: TextureAspect,
    pub format: TextureFormat,
    pub view_format: TextureFormat,
    pub usage: TexUse,
}


impl TexDsc {
    pub fn new_2d(
        size: [u32; 3], sample_count: u32,
        format: TextureFormat, view_format: Option<TextureFormat>, usage: TexUse,
    ) -> Self {
        Self {
            label: None, size, mip_level_count: 1, sample_count, usage,
            view_dimension: dimension_to_view(TextureDimension::D2, size[2]),
            view_aspect: TextureAspect::All,
            format, view_format: view_format.unwrap_or(format),
        }
    }
    pub fn srgb(&self) -> bool { self.view_format.is_srgb() }
    pub fn size_2d(&self) -> [u32; 2] { [self.size[0], self.size[1]] }
    pub fn set_size_2d(&mut self, [width, height]: [u32; 2]) {
        self.size[0] = width;
        self.size[1] = height;
    }

    pub fn default_view(&self) -> TextureViewDescriptor<'static> {
        TextureViewDescriptor {
            label: None,
            format: Some(self.view_format),
            dimension: Some(self.view_dimension),
            aspect: self.view_aspect,
            base_mip_level: 0,
            mip_level_count: Some(self.mip_level_count),
            base_array_layer: 0,
            array_layer_count: Some(self.size[2]),
        }
    }
}


// to / from TextureDescriptor
use wgpu::Label;

type TextureDescriptor<'a> = wgpu_types::TextureDescriptor<Label<'static>, &'a [TextureFormat]>;


impl<'a> From<&'a TexDsc> for TextureDescriptor<'a> {
    fn from(dsc: &'a TexDsc) -> TextureDescriptor<'a> {
        TextureDescriptor {
            label: dsc.label,
            size: ToExtent3d::to(dsc.size),
            mip_level_count: dsc.mip_level_count,
            sample_count: dsc.sample_count,
            dimension: view_to_dimension(dsc.view_dimension),
            format: dsc.format,
            usage: dsc.usage,
            view_formats: slice::from_ref(&dsc.view_format),
        }
    }
}


impl From<&TextureDescriptor<'_>> for TexDsc {
    fn from(tdsc: &TextureDescriptor<'_>) -> TexDsc {
        let size = tdsc.size.to_arr();
        TexDsc {
            label: tdsc.label,
            size,
            mip_level_count: tdsc.mip_level_count,
            sample_count: tdsc.sample_count,
            view_dimension: dimension_to_view(tdsc.dimension, size[2]),
            view_aspect: TextureAspect::All,
            format: tdsc.format,
            view_format: *tdsc.view_formats.first().unwrap_or(&tdsc.format),
            usage: tdsc.usage,
        }
    }
}

impl From<TextureDescriptor<'_>> for TexDsc {
    fn from(tdsc: TextureDescriptor<'_>) -> TexDsc { (&tdsc).into() }
}