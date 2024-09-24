
use wgpu::{StoreOp, TextureFormat};
use crate::Color;

// render attachments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorTransform { None, Srgb, Linear }


pub type RenderAttachments<'a, const S: usize> = (
    [Option<wgpu::RenderPassColorAttachment<'a>>; S],
    Option<wgpu::RenderPassDepthStencilAttachment<'a>>
);


#[derive(Debug, Clone, Copy)]
pub struct ColorAttachment<'a> {
    pub view: &'a wgpu::TextureView,
    pub msaa: Option<&'a wgpu::TextureView>,
    pub clear: Option<(Color, ColorTransform)>,
}

impl<'a> From<ColorAttachment<'a>> for wgpu::RenderPassColorAttachment<'a> {
    fn from(att: ColorAttachment<'a>) -> Self {
        Self {
            view: if let Some(msaa_view) = att.msaa { msaa_view } else { att.view },
            resolve_target: if att.msaa.is_some() { Some(att.view) } else { None },
            ops: wgpu::Operations {
                load: if let Some((color, correct)) = att.clear { wgpu::LoadOp::Clear(
                    match correct {
                        ColorTransform::None => color.into(),
                        ColorTransform::Srgb => color.srgb().into(),
                        ColorTransform::Linear => color.linear().into(),
                    }
                ) }
                else { wgpu::LoadOp::Load },
                store: StoreOp::Store,
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DepthAttachment<'a> {
    pub view: &'a wgpu::TextureView,
    pub format: TextureFormat,
    pub clear_depth: Option<f32>,
    pub clear_stencil: Option<u32>,
}

impl<'a> From<DepthAttachment<'a>> for wgpu::RenderPassDepthStencilAttachment<'a> {
    fn from(att: DepthAttachment<'a>) -> Self {
        Self {
            view: att.view,

            depth_ops: if att.format.has_depth_aspect() {
                Some(wgpu::Operations {
                    load: if let Some(cl) = att.clear_depth { wgpu::LoadOp::Clear(cl) } else { wgpu::LoadOp::Load },
                    store: StoreOp::Store,
                })
            } else { None },

            stencil_ops: if att.format.has_stencil_aspect() {
                Some(wgpu::Operations {
                    load: if let Some(cl) = att.clear_stencil { wgpu::LoadOp::Clear(cl) } else { wgpu::LoadOp::Load },
                    store: StoreOp::Store,
                })
            } else { None },
        }
    }
}


// encoder extension
use crate::util_extension::*;

pub trait EncoderExtension {

    fn buffer_to_buffer(
        &mut self, src_buffer:&wgpu::Buffer, src_offset:wgpu::BufferAddress,
        dst_buffer:&wgpu::Buffer, dst_offset:wgpu::BufferAddress, size:wgpu::BufferAddress,
    );

    fn buffer_to_texture<'b, 't>(
        &mut self, buffer: impl ToImageCopyBuffer<'b>, texture: impl ToImageCopyTexture<'t>, extent: impl ToExtent3d,
    );

    fn texture_to_buffer<'t, 'b>(
        &mut self, texture: impl ToImageCopyTexture<'t>, buffer: impl ToImageCopyBuffer<'b>, extent: impl ToExtent3d,
    );

    fn compute_pass(&mut self) -> wgpu::ComputePass;

    fn with_compute_pass<'a, T>(&'a mut self, handler: impl FnOnce(&mut wgpu::ComputePass<'a>) -> T) -> T;

    fn render_pass<'a, const S: usize>(&'a mut self, attachments: RenderAttachments<'a, S>) -> wgpu::RenderPass<'a>;

    fn with_render_pass<'a, const S: usize, T>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        handler: impl FnOnce(&mut wgpu::RenderPass<'a>) -> T
    ) -> T;

    fn pass_bundles<'a, const S: usize>(
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


    fn buffer_to_texture<'b, 't>(
        &mut self, buffer: impl ToImageCopyBuffer<'b>, texture: impl ToImageCopyTexture<'t>, extent: impl ToExtent3d,
    ) {
        self.copy_buffer_to_texture(buffer.to(), texture.to(), extent.to());
    }


    fn texture_to_buffer<'t, 'b>(
        &mut self, texture: impl ToImageCopyTexture<'t>, buffer: impl ToImageCopyBuffer<'b>, extent: impl ToExtent3d,
    ) {
        self.copy_texture_to_buffer(texture.to(), buffer.to(), extent.to());
    }


    fn compute_pass(&mut self) -> wgpu::ComputePass {
        self.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        })
    }


    fn with_compute_pass<'a, T>(&'a mut self, handler: impl FnOnce(&mut wgpu::ComputePass<'a>) -> T) -> T {
        handler(&mut self.compute_pass())
    }


    fn render_pass<'a, const S: usize>(&'a mut self, (color_attachments, depth_stencil_attachment): RenderAttachments<'a, S>)
        -> wgpu::RenderPass<'a>
    {
        self.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }


    fn with_render_pass<'a, const S: usize, T>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        handler: impl FnOnce(&mut wgpu::RenderPass<'a>) -> T
    ) -> T {
        handler(&mut self.render_pass(attachments))
    }


    fn pass_bundles<'a, const S: usize>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        bundles: impl IntoIterator<Item = &'a wgpu::RenderBundle> + 'a
    ) {
        self.render_pass(attachments).execute_bundles(bundles);
    }
}