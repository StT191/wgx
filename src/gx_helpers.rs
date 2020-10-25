

use std::{ops::Range};
use crate::Color;



// Texture Format Option enum

pub const OUTPUT_FORMAT:wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const TEXTURE_FORMAT:wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;


#[derive(Debug)]
pub enum TexOpt { Output, Texture, Depth }

impl TexOpt {
    pub fn select(format:Self) -> wgpu::TextureFormat {
        match format {
            TexOpt::Output => OUTPUT_FORMAT,
            TexOpt::Texture => TEXTURE_FORMAT,
            TexOpt::Depth => DEPTH_FORMAT
        }
    }
}


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
    fn draw(
        &mut self,
        frame:(&wgpu::TextureView, Option<&wgpu::TextureView>, Option<&wgpu::TextureView>),
        color:Option<Color>,
        draws:&[(&wgpu::RenderPipeline, &wgpu::BindGroup, wgpu::BufferSlice, Range<u32>)]
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
            wgpu::BufferCopyViewBase {
                buffer, layout: wgpu::TextureDataLayout { offset, bytes_per_row: 4 * bf_w, rows_per_image: bf_h }
            },
            wgpu::TextureCopyViewBase { texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z: 0, } },
            wgpu::Extent3d {width: w, height: h, depth: 1},
        );
    }


    fn texture_to_buffer(
        &mut self,
        texture:&wgpu::Texture, (x, y, w, h):(u32, u32, u32, u32),
        buffer:&wgpu::Buffer, (bf_w, bf_h, offset):(u32, u32, u64)
    ) {
        self.copy_texture_to_buffer(
            wgpu::TextureCopyViewBase { texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z: 0, } },
            wgpu::BufferCopyViewBase {
                buffer, layout:  wgpu::TextureDataLayout { offset, bytes_per_row: 4 * bf_w, rows_per_image: bf_h }
            },
            wgpu::Extent3d {width: w, height: h, depth: 1},
        );
    }



    // pass render function

    fn draw(
        &mut self,
        (attachment, depth_attachment, mssa_attachment)
        :(&wgpu::TextureView, Option<&wgpu::TextureView>, Option<&wgpu::TextureView>),
        color:Option<Color>,
        draws:&[(&wgpu::RenderPipeline, &wgpu::BindGroup, wgpu::BufferSlice, Range<u32>)]
    ) {
        let mut rpass = self.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: if let Some(ms_at) = mssa_attachment { ms_at } else { attachment },
                resolve_target: if mssa_attachment.is_some() { Some(attachment) } else { None },
                ops: wgpu::Operations {
                    load: if let Some(cl) = color
                        { wgpu::LoadOp::Clear( cl.into() ) }
                        else { wgpu::LoadOp::Load },
                    store: true
                }
            }],
            depth_stencil_attachment: if let Some(depth_attachment) = depth_attachment {
              Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: depth_attachment,
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
        });

        /*let mut l_render_pipeline = None;
        let mut l_bind_group = None;
        let mut l_vertices = None;*/

        for (render_pipeline, bind_group, vertices, range) in draws {

            // if l_render_pipeline != Some(render_pipeline) {
            rpass.set_pipeline(render_pipeline);
                /*l_render_pipeline = Some(render_pipeline);
            }*/

            // if l_bind_group != Some(bind_group) {
            rpass.set_bind_group(0, bind_group, &[]);
                /*l_bind_group = Some(bind_group);
            }*/

            // if l_vertices != Some(vertices) {
            rpass.set_vertex_buffer(0, *vertices);
                /*l_vertices = Some(vertices);
            }*/

            rpass.draw(range.clone(), 0..1);
        }
    }

}


