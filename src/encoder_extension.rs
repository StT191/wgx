
use crate::RenderAttachments;


pub trait EncoderExtension {

    fn buffer_to_buffer(
        &mut self, src_buffer:&wgpu::Buffer, src_offset:wgpu::BufferAddress,
        dst_buffer:&wgpu::Buffer, dst_offset:wgpu::BufferAddress, size:wgpu::BufferAddress,
    );

    fn buffer_to_texture(
        &mut self,
        buffer:&wgpu::Buffer, bf_extend:(u64, [u32;2]),
        texture:&wgpu::Texture, tx_extend:([u32;3], [u32;3])
    );

    fn texture_to_buffer(
        &mut self,
        texture:&wgpu::Texture, tx_extend:([u32;3], [u32;3]),
        buffer:&wgpu::Buffer, bf_extend:(u64, [u32;2]),
    );

    fn compute_pass(&mut self) -> wgpu::ComputePass;

    fn with_compute_pass<'a, T>(&'a mut self, handler: impl FnOnce(wgpu::ComputePass<'a>) -> T) -> T;

    fn render_pass<'a, const S: usize>(&'a mut self, attachments: RenderAttachments<'a, S>) -> wgpu::RenderPass<'a>;

    fn with_render_pass<'a, const S: usize, T>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        handler: impl FnOnce(wgpu::RenderPass<'a>) -> T
    ) -> T;

    fn render_bundles<'a, const S: usize>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        bundles: impl IntoIterator<Item = &'a wgpu::RenderBundle> + 'a
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
        buffer:&wgpu::Buffer, (offset, [buffer_width, buffer_height]):(u64, [u32;2]),
        texture:&wgpu::Texture, ([x, y, z], [width, height, layers]):([u32;3], [u32;3])
    ) {
        self.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset,
                    bytes_per_row: Some(4 * buffer_width),
                    rows_per_image: Some(buffer_height),
                }
            },
            wgpu::ImageCopyTexture { texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z, }, aspect: wgpu::TextureAspect::All },
            wgpu::Extent3d {width, height, depth_or_array_layers: layers},
        );
    }


    fn texture_to_buffer(
        &mut self,
        texture:&wgpu::Texture, ([x, y, z], [width, height, layers]):([u32;3], [u32;3]),
        buffer:&wgpu::Buffer, (offset, [buffer_width, buffer_height]):(u64, [u32;2])
    ) {
        self.copy_texture_to_buffer(
            wgpu::ImageCopyTexture { texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z, }, aspect: wgpu::TextureAspect::All },
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset,
                    bytes_per_row: Some(4 * buffer_width),
                    rows_per_image: Some(buffer_height),
                }
            },
            wgpu::Extent3d {width, height, depth_or_array_layers: layers},
        );
    }


    fn compute_pass(&mut self) -> wgpu::ComputePass {
        self.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None })
    }


    fn with_compute_pass<'a, T>(&'a mut self, handler: impl FnOnce(wgpu::ComputePass<'a>) -> T) -> T {
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
        bundles: impl IntoIterator<Item = &'a wgpu::RenderBundle> + 'a
    ) {
        self.render_pass(attachments).execute_bundles(bundles.into_iter());
    }
}