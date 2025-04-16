
use wgpu::{
    Extent3d, Origin3d, Texture, TextureFormat, Buffer,
    TexelCopyBufferLayout, TexelCopyTextureInfo, TexelCopyBufferInfo,
};


pub trait ToArray3 {
    fn to_arr(self) -> [u32; 3];
}


pub trait ToExtent3d {
    fn to(self) -> Extent3d;
}
impl ToExtent3d for Extent3d {
    fn to(self) -> Extent3d { self }
}

impl ToExtent3d for [u32; 3] {
    fn to(self) -> Extent3d {
        Extent3d { width: self[0], height: self[1], depth_or_array_layers: self[2] }
    }
}

impl ToArray3 for Extent3d {
    fn to_arr(self) -> [u32; 3] { [self.width, self.height, self.depth_or_array_layers] }
}


pub trait ToOrigin3d {
    fn to(self) -> Origin3d;
}
impl ToOrigin3d for Origin3d {
    fn to(self) -> Origin3d { self }
}

impl ToOrigin3d for [u32; 3] {
    fn to(self) -> Origin3d {
        Origin3d { x: self[0], y: self[1], z: self[2] }
    }
}

impl ToArray3 for Origin3d {
    fn to_arr(self) -> [u32; 3] { [self.x, self.y, self.z] }
}


pub trait ToTexelCopyBufferLayout {
    fn to(self) -> TexelCopyBufferLayout;
}
impl ToTexelCopyBufferLayout for TexelCopyBufferLayout {
    fn to(self) -> TexelCopyBufferLayout { self }
}

impl ToTexelCopyBufferLayout for (u64, Option<u32>, Option<u32>) {
    fn to(self) -> TexelCopyBufferLayout {
        TexelCopyBufferLayout {
            offset: self.0,
            bytes_per_row: self.1,
            rows_per_image: self.2,
        }
    }
}

impl ToTexelCopyBufferLayout for (u64, (TextureFormat, u32), Option<u32>) {
    fn to(self) -> TexelCopyBufferLayout {
        TexelCopyBufferLayout {
            offset: self.0,
            bytes_per_row: self.1.0.block_copy_size(None).map(|size| size * self.1.1),
            rows_per_image: self.2,
        }
    }
}


pub trait ToTexelCopyTextureInfo<'a> {
    fn to(self) -> TexelCopyTextureInfo<'a>;
}
impl<'a> ToTexelCopyTextureInfo<'a> for TexelCopyTextureInfo<'a> {
    fn to(self) -> TexelCopyTextureInfo<'a> { self }
}


impl<'a> ToTexelCopyTextureInfo<'a> for (&'a Texture, u32, [u32; 3]) {
    fn to(self) -> TexelCopyTextureInfo<'a> {
        TexelCopyTextureInfo {
            texture: self.0, mip_level: self.1,
            origin: ToOrigin3d::to(self.2),
            aspect: wgpu::TextureAspect::All,
        }
    }
}



pub trait ToTexelCopyBufferInfo<'a> {
    fn to(self) -> TexelCopyBufferInfo<'a>;
}
impl<'a> ToTexelCopyBufferInfo<'a> for TexelCopyBufferInfo<'a> {
    fn to(self) -> TexelCopyBufferInfo<'a> { self }
}


impl<'a, L: ToTexelCopyBufferLayout> ToTexelCopyBufferInfo<'a> for (&'a Buffer, L) {
    fn to(self) -> TexelCopyBufferInfo<'a> {
        TexelCopyBufferInfo { buffer: self.0, layout: self.1.to() }
    }
}

impl<'a> ToTexelCopyBufferInfo<'a> for (&'a Buffer, u64, Option<u32>, Option<u32>) {
    fn to(self) -> TexelCopyBufferInfo<'a> {
        TexelCopyBufferInfo { buffer: self.0, layout: (self.1, self.2, self.3).to() }
    }
}

impl<'a> ToTexelCopyBufferInfo<'a> for (&'a Buffer, u64, (TextureFormat, u32), Option<u32>) {
    fn to(self) -> TexelCopyBufferInfo<'a> {
        TexelCopyBufferInfo { buffer: self.0, layout: (self.1, self.2, self.3).to() }
    }
}



use wgpu::{ColorTargetState, ColorWrites};
use crate::Blend;

pub trait AsColorTarget {
    fn target(self) -> Option<ColorTargetState>;
}
impl AsColorTarget for ColorTargetState {
    fn target(self) -> Option<ColorTargetState> { Some(self) }
}

impl AsColorTarget for TextureFormat {
    fn target(self) -> Option<ColorTargetState> {
        Some(ColorTargetState {
            format: self,
            blend: None,
            write_mask: ColorWrites::all(),
        })
    }
}

impl AsColorTarget for (TextureFormat, Option<Blend>, ColorWrites) {
    fn target(self) -> Option<ColorTargetState> {
        Some(ColorTargetState {
            format: self.0,
            blend: self.1,
            write_mask: self.2,
        })
    }
}

impl AsColorTarget for (TextureFormat, Option<Blend>) {
    fn target(self) -> Option<ColorTargetState> {
        (self.0, self.1, ColorWrites::all()).target()
    }
}

impl AsColorTarget for (TextureFormat, Blend) {
    fn target(self) -> Option<ColorTargetState> {
        (self.0, Some(self.1)).target()
    }
}