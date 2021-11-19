
// use core::ops::Range;
use std::{num::NonZeroU32};
use crate::{Color, RenderAttachment};


// default view extension
pub trait DefaultViewExtension<T> {
    fn create_default_view(&self) -> T;
}

impl DefaultViewExtension<wgpu::TextureView> for wgpu::Texture {
    fn create_default_view(&self) -> wgpu::TextureView {
        self.create_view(&wgpu::TextureViewDescriptor::default())
    }
}


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

    fn render_pass<'a>(&'a mut self, attachment:&'a RenderAttachment, color:Option<Color>) -> wgpu::RenderPass<'a>;

    fn with_render_pass<'a>(
        &'a mut self, attachment:&'a RenderAttachment, color:Option<Color>,
        handler: impl FnOnce(wgpu::RenderPass<'a>)
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
        buffer:&wgpu::Buffer, (bf_w, bf_h, offset):(u32, u32, u64),
        texture:&wgpu::Texture, (x, y, w, h):(u32, u32, u32, u32)
    ) {
        self.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer, layout: wgpu::ImageDataLayout { offset, bytes_per_row: NonZeroU32::new(4 * bf_w), rows_per_image: NonZeroU32::new(bf_h) }
            },
            wgpu::ImageCopyTexture { texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z: 0, }, aspect: wgpu::TextureAspect::All },
            wgpu::Extent3d {width: w, height: h, depth_or_array_layers: 1},
        );
    }


    fn texture_to_buffer(
        &mut self,
        texture:&wgpu::Texture, (x, y, w, h):(u32, u32, u32, u32),
        buffer:&wgpu::Buffer, (bf_w, bf_h, offset):(u32, u32, u64)
    ) {
        self.copy_texture_to_buffer(
            wgpu::ImageCopyTexture { texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z: 0, }, aspect: wgpu::TextureAspect::All },
            wgpu::ImageCopyBuffer {
                buffer, layout:  wgpu::ImageDataLayout { offset, bytes_per_row: NonZeroU32::new(4 * bf_w), rows_per_image: NonZeroU32::new(bf_h) }
            },
            wgpu::Extent3d {width: w, height: h, depth_or_array_layers: 1},
        );
    }


    fn render_pass<'a>(&'a mut self, attachment:&'a RenderAttachment, color:Option<Color>) -> wgpu::RenderPass<'a>
    {
        self.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: if let Some(ms_at) = attachment.msaa { ms_at } else { attachment.view },
                resolve_target: if attachment.msaa.is_some() { Some(attachment.view) } else { None },
                ops: wgpu::Operations {
                    load: if let Some(cl) = color
                        { wgpu::LoadOp::Clear(cl.into()) }
                        else { wgpu::LoadOp::Load },
                    store: true
                }
            }],
            depth_stencil_attachment: if let Some(depth_attachment) = attachment.depth {
              Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_attachment,
                depth_ops: Some(wgpu::Operations {
                    load: if color.is_some() { wgpu::LoadOp::Clear(1.0) } else { wgpu::LoadOp::Load },
                    store: true
                }),
                stencil_ops: None,
                /*stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true
                }),*/
            })} else { None },
        })
    }


    fn with_render_pass<'a>(
        &'a mut self, attachment:&'a RenderAttachment, color:Option<Color>,
        handler: impl FnOnce(wgpu::RenderPass<'a>)
    ) {
        handler(self.render_pass(attachment, color));
    }
}



// render pass

/*pub trait Drawable<'a> {
    fn draw(&'a self, render_pass:&mut wgpu::RenderPass<'a>);
}


// commit draw commands, pass the render pass
pub trait RenderPassExtension<'a> {
    fn pass<T: Drawable<'a>>(self, drawable: &'a T) -> Self;
}

impl<'a> RenderPassExtension<'a> for wgpu::RenderPass<'a> {
    fn pass<T: Drawable<'a>>(mut self, drawable: &'a T) -> Self {
        drawable.draw(&mut self);
        self
    }
}


// impl for drawables

impl<'a> Drawable<'a> for (&'a wgpu::RenderPipeline, &'a wgpu::BindGroup, wgpu::BufferSlice<'a>, Range<u32>) {
    fn draw(&'a self, rpass:&mut wgpu::RenderPass<'a>) {
        rpass.set_pipeline(self.0);
        rpass.set_bind_group(0, self.1, &[]);
        rpass.set_vertex_buffer(0, self.2);
        rpass.draw(self.3.clone(), 0..1);
    }
}*/