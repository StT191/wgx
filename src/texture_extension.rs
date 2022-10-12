
use crate::{TexUse, TexDsc};


// default Texture Formats

pub const TEXTURE:wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const TEXTURE_LINEAR:wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;



// extend TextureDescriptor

pub trait TextureDescriptorExtension {
    fn new_2d(size: (u32, u32), sample_count: u32, format: wgpu::TextureFormat, usage: TexUse) -> Self;
    fn srgb(&self) -> bool;
    fn size_2d(&self) -> (u32, u32);
    fn set_size_2d(&mut self, size: (u32, u32));
}

impl TextureDescriptorExtension for TexDsc<'_> {
    fn new_2d(size: (u32, u32), sample_count: u32, format: wgpu::TextureFormat, usage: TexUse) -> Self {
        Self {
            usage, label: None, mip_level_count: 1, sample_count, dimension: wgpu::TextureDimension::D2,
            size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 }, format,
        }
    }
    fn srgb(&self) -> bool { self.format.describe().srgb }
    fn size_2d(&self) -> (u32, u32) { (self.size.width, self.size.height) }
    fn set_size_2d(&mut self, (width, height): (u32, u32)) {
        self.size.width = width;
        self.size.height = height;
    }
}


// extend Texture

pub trait TextureExtension<T> {
    fn create_default_view(&self) -> T;
}

impl TextureExtension<wgpu::TextureView> for wgpu::Texture {
    fn create_default_view(&self) -> wgpu::TextureView {
        self.create_view(&wgpu::TextureViewDescriptor::default())
    }
}
