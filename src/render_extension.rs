
use wgpu::{StoreOp, TextureFormat, Texture};
use crate::*;

// render attachments

pub type RenderAttachments<'a, const S: usize> = (
    [Option<wgpu::RenderPassColorAttachment<'a>>; S],
    Option<wgpu::RenderPassDepthStencilAttachment<'a>>
);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorAttachment<'a> {
    pub view: &'a wgpu::TextureView,
    pub format: TextureFormat,
    pub msaa: Option<&'a wgpu::TextureView>,
    pub clear: Option<Color>,
}

impl<'a> From<ColorAttachment<'a>> for wgpu::RenderPassColorAttachment<'a> {
    fn from(att: ColorAttachment<'a>) -> Self {
        Self {
            view: if let Some(msaa_view) = att.msaa { msaa_view } else { att.view },
            resolve_target: if att.msaa.is_some() { Some(att.view) } else { None },
            ops: wgpu::Operations {
                load: match att.clear {
                    Some(color) => wgpu::LoadOp::Clear(color.into()),
                    None => wgpu::LoadOp::Load,
                },
                store: StoreOp::Store,
            },
            depth_slice: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
                    load: match att.clear_depth {
                        Some(depth) => wgpu::LoadOp::Clear(depth),
                        None => wgpu::LoadOp::Load,
                    },
                    store: StoreOp::Store,
                })
            } else { None },

            stencil_ops: if att.format.has_stencil_aspect() {
                Some(wgpu::Operations {
                    load: match att.clear_stencil {
                        Some(stencil) => wgpu::LoadOp::Clear(stencil),
                        None => wgpu::LoadOp::Load,
                    },
                    store: StoreOp::Store,
                })
            } else { None },
        }
    }
}


// render target

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetDsc {
    pub msaa: u32,
    pub depth_testing: Option<TextureFormat>,
    pub format: TextureFormat,
}

pub trait RenderTarget {

    // to implement
    fn size(&self) -> [u32; 2];
    fn msaa(&self) -> u32;
    fn depth_testing(&self) -> Option<TextureFormat>;
    fn format(&self) -> TextureFormat;

    fn target_dsc(&self) -> TargetDsc {
        TargetDsc { msaa: self.msaa(), depth_testing: self.depth_testing(), format: self.format() }
    }

    fn bytes_per_row(&self) -> Option<u32> {
        self.format().block_copy_size(None).map(|bytes| bytes * self.size()[0])
    }

    fn render_bundle_encoder<'a>(&self, gx: &'a impl WgxDevice, config_fn: impl FnOnce(&mut wgpu::RenderBundleEncoderDescriptor))
        -> wgpu::RenderBundleEncoder<'a>
    {
        gx.render_bundle_encoder(
            render_bundle_encoder_descriptor(self.msaa(), self.depth_testing(), &[Some(self.format())]),
            config_fn,
        )
    }

    fn render_bundle<'a>(&self, gx: &'a impl WgxDevice,
        config_fn: impl FnOnce(&mut wgpu::RenderBundleEncoderDescriptor),
        handler: impl FnOnce(&mut wgpu::RenderBundleEncoder<'a>),
    ) -> wgpu::RenderBundle {
        self.render_bundle_encoder(gx, config_fn).record(handler)
    }
}

pub trait RenderAttachable {

    // to implement
    fn color_views(&self) -> (&wgpu::TextureView, wgpu::TextureFormat, Option<&wgpu::TextureView>);
    fn depth_view(&self) -> Option<(&wgpu::TextureView, wgpu::TextureFormat)>;

    // provided
    fn color_attachment(&self, clear_color: Option<Color>) -> ColorAttachment<'_> {
        let (view, format, msaa) = self.color_views();
        ColorAttachment { view, msaa, format, clear: clear_color }
    }

    fn depth_attachment(&self, clear_depth: Option<f32>, clear_stencil: Option<u32>) -> Option<DepthAttachment<'_>> {
        self.depth_view().map(|(view, format)| DepthAttachment { view, format, clear_depth, clear_stencil })
    }

    fn attachments(&self, clear_color: Option<Color>, clear_depth: Option<f32>, clear_stencil: Option<u32>) -> RenderAttachments<'_, 1> {
        ([Some(self.color_attachment(clear_color).into())], self.depth_attachment(clear_depth, clear_stencil).map(|a| a.into()))
    }
}


impl RenderTarget for TargetDsc {
    fn size(&self) -> [u32; 2] { [0, 0] }
    fn msaa(&self) -> u32 { self.msaa }
    fn depth_testing(&self) -> Option<TextureFormat> { self.depth_testing }
    fn format(&self) -> TextureFormat { self.format }
    fn target_dsc(&self) -> Self { *self }
}


impl RenderTarget for Texture {
    fn size(&self) -> [u32; 2] { [self.width(), self.height()] }
    fn msaa(&self) -> u32 { 1 }
    fn depth_testing(&self) -> Option<TextureFormat> { None }
    fn format(&self) -> TextureFormat { self.format() }
}


pub type ColorViews<'a> = (&'a wgpu::TextureView, wgpu::TextureFormat, Option<&'a wgpu::TextureView>);

impl RenderAttachable for ColorViews<'_> {
    fn color_views(&self) -> (&wgpu::TextureView, wgpu::TextureFormat, Option<&wgpu::TextureView>) { *self }
    fn depth_view(&self) -> Option<(&wgpu::TextureView, wgpu::TextureFormat)> { None }
}


pub trait RenderBundleEncoderExtension {
    fn bundle(self) -> wgpu::RenderBundle;
    fn record(self, handler: impl FnOnce(&mut Self)) -> wgpu::RenderBundle;
}


impl RenderBundleEncoderExtension for wgpu::RenderBundleEncoder<'_> {

    fn bundle(self) -> wgpu::RenderBundle {
        self.finish(&wgpu::RenderBundleDescriptor::default())
    }

    fn record(mut self, handler: impl FnOnce(&mut Self)) -> wgpu::RenderBundle {
        handler(&mut self);
        self.bundle()
    }
}


pub trait EncoderExtension {

    fn compute_pass(&mut self) -> wgpu::ComputePass<'_>;

    fn with_compute_pass<'a, T>(&mut self, handler: impl FnOnce(&mut wgpu::ComputePass<'a>) -> T) -> T;

    fn render_pass<'a, const S: usize>(&'a mut self, attachments: RenderAttachments<'a, S>) -> wgpu::RenderPass<'a>;

    fn with_render_pass<'a, 'b, const S: usize, T>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        handler: impl FnOnce(&mut wgpu::RenderPass<'b>) -> T
    ) -> T;

    fn pass_bundles<'a, const S: usize>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        bundles: impl IntoIterator<Item = &'a wgpu::RenderBundle> + 'a
    );
}

impl EncoderExtension for wgpu::CommandEncoder {

    fn compute_pass(&mut self) -> wgpu::ComputePass<'_> {
        self.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        })
    }

    fn with_compute_pass<'a, T>(&mut self, handler: impl FnOnce(&mut wgpu::ComputePass<'a>) -> T) -> T {
        handler(&mut self.compute_pass().forget_lifetime())
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

    fn with_render_pass<'a, 'b, const S: usize, T>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        handler: impl FnOnce(&mut wgpu::RenderPass<'b>) -> T
    ) -> T {
        handler(&mut self.render_pass(attachments).forget_lifetime())
    }


    fn pass_bundles<'a, const S: usize>(
        &'a mut self, attachments: RenderAttachments<'a, S>,
        bundles: impl IntoIterator<Item = &'a wgpu::RenderBundle> + 'a
    ) {
        self.render_pass(attachments).execute_bundles(bundles);
    }
}