
// use core::ops::Range;
use std::{num::NonZeroU32};
use crate::RenderAttachments;


pub trait EncoderExtension {

    fn buffer_to_buffer(
        &mut self, src_buffer:&wgpu::Buffer, src_offset:wgpu::BufferAddress,
        dst_buffer:&wgpu::Buffer, dst_offset:wgpu::BufferAddress, size:wgpu::BufferAddress,
    );

    fn buffer_to_texture(
        &mut self,
        buffer:&wgpu::Buffer, bf_extend:(u32, u32, u64),
        texture:&wgpu::Texture, tx_extend:(u32, u32, u32, u32)
    );

    fn texture_to_buffer(
        &mut self,
        texture:&wgpu::Texture, tx_extend:(u32, u32, u32, u32),
        buffer:&wgpu::Buffer, bf_extend:(u32, u32, u64),
    );

    fn compute_pass<'a>(&'a mut self) -> wgpu::ComputePass<'a>;

    fn with_compute_pass<'a, T>(&'a mut self, handler: impl FnOnce(wgpu::ComputePass<'a>) -> T) -> T;

    fn render_pass<'a, const S: usize>(&'a mut self, attachments: RenderAttachments<'a, S>) -> wgpu::RenderPass<'a>;

    fn with_render_pass<'a, const S: usize, T>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        handler: impl FnOnce(wgpu::RenderPass<'a>) -> T
    ) -> T;

    fn render_bundles<'a, const S: usize>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        bundles: impl IntoIterator<Item = &'a wgpu::RenderBundle>
    );
}



impl EncoderExtension for wgpu::CommandEncoder {

    fn buffer_to_buffer(
        &mut self,
        src_buffer:&wgpu::Buffer, src_offset:wgpu::BufferAddress,
        dst_buffer:&wgpu::Buffer, dst_offset:wgpu::BufferAddress,
        size:wgpu::BufferAddress,
    ) {
        self.copy_buffer_to_buffer(src_buffer, src_offset, dst_buffer, dst_offset, size);
    }


    fn buffer_to_texture(
        &mut self,
        buffer:&wgpu::Buffer, (buffer_width, buffer_height, offset):(u32, u32, u64),
        texture:&wgpu::Texture, (x, y, width, height):(u32, u32, u32, u32)
    ) {
        self.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset,
                    bytes_per_row: NonZeroU32::new(4 * buffer_width),
                    rows_per_image: NonZeroU32::new(buffer_height),
                }
            },
            wgpu::ImageCopyTexture { texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z: 0, }, aspect: wgpu::TextureAspect::All },
            wgpu::Extent3d {width, height, depth_or_array_layers: 1},
        );
    }


    fn texture_to_buffer(
        &mut self,
        texture:&wgpu::Texture, (x, y, width, height):(u32, u32, u32, u32),
        buffer:&wgpu::Buffer, (buffer_width, buffer_height, offset):(u32, u32, u64)
    ) {
        self.copy_texture_to_buffer(
            wgpu::ImageCopyTexture { texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z: 0, }, aspect: wgpu::TextureAspect::All },
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset,
                    bytes_per_row: NonZeroU32::new(4 * buffer_width),
                    rows_per_image: NonZeroU32::new(buffer_height),
                }
            },
            wgpu::Extent3d {width, height, depth_or_array_layers: 1},
        );
    }


    fn compute_pass<'a>(&'a mut self) -> wgpu::ComputePass<'a> {
        self.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None })
    }


    fn with_compute_pass<'a, T>(&'a mut self, handler: impl FnOnce(wgpu::ComputePass<'a>) -> T) -> T{
        handler(self.compute_pass())
    }


    fn render_pass<'a, const S: usize>(&'a mut self, (color_attachments, depth_stencil_attachment): RenderAttachments<'a, S>)
        -> wgpu::RenderPass<'a>
    {
        self.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None, color_attachments: &color_attachments, depth_stencil_attachment
        })
    }


    fn with_render_pass<'a, const S: usize, T>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        handler: impl FnOnce(wgpu::RenderPass<'a>) -> T
    ) -> T {
        handler(self.render_pass(attachments))
    }


    fn render_bundles<'a, const S: usize>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        bundles: impl IntoIterator<Item = &'a wgpu::RenderBundle>
    ) {
        self.render_pass(attachments).execute_bundles(bundles.into_iter());
    }
}