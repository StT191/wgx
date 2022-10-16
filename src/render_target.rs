
use crate::{*, error::*, byte_slice::AsByteSlice};
use wgpu::PresentMode as Prs;


pub type RenderAttachments<'a, const S: usize> = (
    [Option<wgpu::RenderPassColorAttachment<'a>>; S],
    Option<wgpu::RenderPassDepthStencilAttachment<'a>>
);


#[derive(Debug, Clone, Copy)]
pub struct ColorAttachment<'a> {
    pub view: &'a wgpu::TextureView,
    pub msaa: Option<&'a wgpu::TextureView>,
    pub clear: Option<(Color, bool)>,
}

impl<'a> Into<wgpu::RenderPassColorAttachment<'a>> for ColorAttachment<'a> {
    fn into(self) -> wgpu::RenderPassColorAttachment<'a> {
        wgpu::RenderPassColorAttachment {
            view: if let Some(msaa_view) = self.msaa { &msaa_view } else { self.view },
            resolve_target: if self.msaa.is_some() { Some(self.view) } else { None },
            ops: wgpu::Operations {
                load: if let Some((color, srgb)) = self.clear { wgpu::LoadOp::Clear(
                    if self.msaa.is_none() || !srgb { color.linear().into() } // convert to linear color space
                    else { color.into() } // unless using attachment with srgb
                )}
                else { wgpu::LoadOp::Load },
                store: true,
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DepthAttachment<'a> {
    pub view: &'a wgpu::TextureView,
    pub clear: Option<f32>,
}

impl<'a> Into<wgpu::RenderPassDepthStencilAttachment<'a>> for DepthAttachment<'a> {
    fn into(self) -> wgpu::RenderPassDepthStencilAttachment<'a> {
        wgpu::RenderPassDepthStencilAttachment {
            view: self.view,
            depth_ops: Some(wgpu::Operations {
                load: if let Some(cl) = self.clear { wgpu::LoadOp::Clear(cl) } else { wgpu::LoadOp::Load },
                store: true,
            }),
            stencil_ops: None,
            /*stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                store: true
            }),*/
        }
    }
}


pub trait RenderTarget {

    // to implement
    fn size(&self) -> (u32, u32);
    fn msaa(&self) -> u32;
    fn depth_testing(&self) -> bool;
    fn format(&self) -> wgpu::TextureFormat;

    // provided

    fn srgb(&self) -> bool { self.format().describe().srgb }

    fn render_bundle_encoder<'a>(&self, gx: &'a Wgx) -> wgpu::RenderBundleEncoder<'a> {
        gx.render_bundle_encoder(&[Some(self.format())], self.depth_testing(), self.msaa())
    }

    fn render_bundle<'a>(&self, gx: &'a Wgx, handler: impl FnOnce(&mut wgpu::RenderBundleEncoder<'a>)) -> wgpu::RenderBundle {
        let mut encoder = self.render_bundle_encoder(gx);
        handler(&mut encoder);
        encoder.finish(&wgpu::RenderBundleDescriptor::default())
    }

    fn render_pipeline(
        &self, gx: &Wgx,
        layout: Option<(&[wgpu::PushConstantRange], &[&wgpu::BindGroupLayout])>,
        buffers: &[wgpu::VertexBufferLayout],
        vertex_state: (&wgpu::ShaderModule, &str, wgpu::PrimitiveTopology),
        (fs_module, fs_entry_point, blend): (&wgpu::ShaderModule, &str, Option<BlendState>),
    ) -> wgpu::RenderPipeline {
        gx.render_pipeline(
            self.depth_testing(), self.msaa(), layout, buffers, vertex_state,
            Some((fs_module, fs_entry_point, &[(self.format(), blend)])),
        )
    }
}

pub trait RenderAttachable: RenderTarget {

    // to implement
    fn color_views(&self) -> (&wgpu::TextureView, Option<&wgpu::TextureView>);
    fn depth_view(&self) -> Option<&wgpu::TextureView>;

    // provided
    fn color_attachment(&self, clear_color: Option<Color>) -> wgpu::RenderPassColorAttachment {
        let (view, msaa) = self.color_views();
        ColorAttachment { view, msaa, clear: clear_color.map(|cl| (cl, self.srgb())) }.into()
    }

    fn depth_attachment(&self, clear: Option<f32>) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        self.depth_view().map(|view| DepthAttachment { view, clear }.into())
    }

    fn attachments(&self, clear_color: Option<Color>, clear_depth: Option<f32>) -> RenderAttachments<1> {
        ([Some(self.color_attachment(clear_color))], self.depth_attachment(clear_depth))
    }
}


// helper
#[derive(Debug)]
pub struct TextureLot<'a> {
    pub texture: wgpu::Texture,
    pub descriptor: TexDsc<'a>,
    pub view: wgpu::TextureView,
}

impl<'a> TextureLot<'a> {
    pub fn new(gx:&Wgx, descriptor: TexDsc<'a>) -> Self {
        let texture = gx.texture(&descriptor);
        let view = texture.create_default_view();
        TextureLot { texture, descriptor, view }
    }
    pub fn new_2d(gx:&Wgx, size:(u32, u32), sample_count:u32, format:wgpu::TextureFormat, usage:TexUse) -> Self {
        Self::new(gx, TexDsc::new_2d(size, sample_count, format, usage))
    }
    pub fn new_2d_bound(gx:&Wgx, size:(u32, u32), sample_count:u32, format:wgpu::TextureFormat, usage:TexUse) -> Self {
        Self::new_2d(gx, size, sample_count, format, TexUse::TEXTURE_BINDING | usage)
    }
    pub fn new_2d_attached(gx:&Wgx, size:(u32, u32), sample_count:u32, format:wgpu::TextureFormat, usage:TexUse) -> Self {
        Self::new_2d(gx, size, sample_count, format, TexUse::RENDER_ATTACHMENT | usage)
    }
    pub fn new_with_data<T: AsByteSlice<U>, U>(gx:&Wgx, descriptor: TexDsc<'a>, data: T) -> Self {
        let texture = gx.texture_with_data(&descriptor, data);
        let view = texture.create_default_view();
        TextureLot { texture, descriptor, view }
    }
    pub fn new_2d_with_data<T: AsByteSlice<U>, U>(
        gx:&Wgx, size:(u32, u32), sample_count:u32, format:wgpu::TextureFormat, usage:TexUse, data: T)
    -> Self {
        Self::new_with_data(gx, TexDsc::new_2d(size, sample_count, format, usage), data)
    }
    pub fn update(&mut self, gx: &Wgx) { *self = Self::new(gx, self.descriptor.clone()) }
}

impl RenderTarget for TextureLot<'_> {
    fn size(&self) -> (u32, u32) { self.descriptor.size_2d() }
    fn msaa(&self) -> u32 { 1 }
    fn depth_testing(&self) -> bool { false }
    fn format(&self) -> wgpu::TextureFormat { self.descriptor.format }
}

impl RenderAttachable for TextureLot<'_> {
    fn color_views(&self) -> (&wgpu::TextureView, Option<&wgpu::TextureView>) { (&self.view, None) }
    fn depth_view(&self) -> Option<&wgpu::TextureView> { None }
}



#[derive(Debug)]
pub struct SurfaceTarget<'a> {
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface,
    pub msaa: u32,
    pub msaa_opt: Option<TextureLot<'a>>,
    pub depth_opt: Option<TextureLot<'a>>,
}

impl RenderTarget for SurfaceTarget<'_> {
    fn size(&self) -> (u32, u32) { (self.config.width, self.config.height) }
    fn msaa(&self) -> u32 { self.msaa }
    fn depth_testing(&self) -> bool { self.depth_opt.is_some() }
    fn format(&self) -> wgpu::TextureFormat { self.config.format }
}

#[derive(Debug)]
pub struct SurfaceFrame<'a, 'b> {
    pub view: wgpu::TextureView,
    pub target: &'b mut SurfaceTarget<'a>,
}

impl RenderTarget for SurfaceFrame<'_, '_> {
    fn size(&self) -> (u32, u32) { self.target.size() }
    fn msaa(&self) -> u32 { self.target.msaa() }
    fn depth_testing(&self) -> bool { self.target.depth_testing() }
    fn format(&self) -> wgpu::TextureFormat { self.target.format() }
}

impl RenderAttachable for SurfaceFrame<'_, '_> {
    fn color_views(&self) -> (&wgpu::TextureView, Option<&wgpu::TextureView>) {
        (&self.view, self.target.msaa_opt.as_ref().map(|o| &o.view))
    }
    fn depth_view(&self) -> Option<&wgpu::TextureView> {
        self.target.depth_opt.as_ref().map(|o| &o.view)
    }
}


// cloneable surface configuration
const SURFACE_CONFIGURATION: wgpu::SurfaceConfiguration = wgpu::SurfaceConfiguration {
    usage: TexUse::RENDER_ATTACHMENT,
    format: TEXTURE, width: 0, height: 0,
    present_mode: Prs::Mailbox,
};


impl<'a> SurfaceTarget<'a> {

    pub fn new(gx:&Wgx, surface:wgpu::Surface, size:(u32, u32), msaa:u32, depth_testing:bool) -> Res<Self>
    {
        let mut config = SURFACE_CONFIGURATION.clone();
        config.width = size.0;
        config.height = size.1;

        let formats = surface.get_supported_formats(&gx.adapter);

        // let format = *formats.get(0).ok_or("couldn't get default format")?;
        let format = *formats.iter().find(|fmt| fmt.describe().srgb).ok_or("couldn't get srgb format")?;
        config.format = format;

        let modes = surface.get_supported_modes(&gx.adapter);

        config.present_mode =
            if modes.contains(&Prs::Mailbox) { Prs::Mailbox }
            else if modes.contains(&Prs::AutoVsync) { Prs::AutoVsync }
            else { *modes.get(0).ok_or("couldn't get default mode")? }
        ;

        surface.configure(&gx.device, &config);

        Ok(Self {
            config, surface, msaa,
            msaa_opt: if msaa > 1 { Some(TextureLot::new_2d(gx, size, msaa, format, TexUse::RENDER_ATTACHMENT)) } else { None },
            depth_opt: if depth_testing { Some(TextureLot::new_2d(gx, size, msaa, DEPTH, TexUse::RENDER_ATTACHMENT)) } else { None },
        })
    }


    pub fn update(&mut self, gx:&Wgx, size:(u32, u32)) {

        self.config.width = size.0;
        self.config.height = size.1;

        self.surface.configure(&gx.device, &self.config);

        let map_opt = |lot:&TextureLot<'a>| {
            let mut descriptor = lot.descriptor.clone();
            descriptor.set_size_2d(size);
            descriptor.sample_count = self.msaa;
            TextureLot::new(gx, descriptor)
        };

        self.msaa_opt = if self.msaa > 1 { self.msaa_opt.as_ref().map(map_opt).or_else(||
            Some(TextureLot::new_2d(gx, size, self.msaa, self.format(), TexUse::RENDER_ATTACHMENT))
        )}
        else { None };

        self.depth_opt = self.depth_opt.as_ref().map(map_opt);
    }


    pub fn with_encoder_frame<C: ImplicitControlflow>(
        &mut self, gx:&Wgx,
        handler: impl FnOnce(&mut wgpu::CommandEncoder, &SurfaceFrame) -> C
    ) -> Res<()>
    {
        let frame = self.surface.get_current_texture().map_err(error)?;

        let controlflow = gx.with_encoder(|mut encoder| handler(&mut encoder, &SurfaceFrame {
            view: frame.texture.create_default_view(),
            target: self,
        }));

        if controlflow.should_continue() {
            frame.present();
        }

        Ok(())
    }
}



#[derive(Debug)]
pub struct TextureTarget<'a> {
    pub texture: wgpu::Texture,
    pub descriptor: TexDsc<'a>,
    pub view: wgpu::TextureView,

    pub msaa: u32,
    pub msaa_opt: Option<TextureLot<'a>>,
    pub depth_opt: Option<TextureLot<'a>>,
}

impl RenderTarget for TextureTarget<'_> {
    fn size(&self) -> (u32, u32) { self.descriptor.size_2d() }
    fn msaa(&self) -> u32 { self.msaa }
    fn depth_testing(&self) -> bool { self.depth_opt.is_some() }
    fn format(&self) -> wgpu::TextureFormat { self.descriptor.format }
}

impl RenderAttachable for TextureTarget<'_> {
    fn color_views(&self) -> (&wgpu::TextureView, Option<&wgpu::TextureView>) {
        (&self.view, self.msaa_opt.as_ref().map(|o| &o.view))
    }
    fn depth_view(&self) -> Option<&wgpu::TextureView> {
        self.depth_opt.as_ref().map(|o| &o.view)
    }
}

impl<'a> TextureTarget<'a> {

    pub fn new(
        gx:&Wgx, size:(u32, u32), msaa:u32, depth_testing: bool, format:wgpu::TextureFormat, usage:wgpu::TextureUsages,
    ) -> Self
    {
        let TextureLot { texture, descriptor, view } = TextureLot::new_2d(gx, size, 1, format, usage | TexUse::RENDER_ATTACHMENT);
        Self {
            texture, descriptor, view, // output attachment can have only one sample
            msaa,
            msaa_opt: if msaa > 1 { Some(TextureLot::new_2d(gx, size, msaa, format, TexUse::RENDER_ATTACHMENT)) } else { None },
            depth_opt: if depth_testing { Some(TextureLot::new_2d(gx, size, msaa, DEPTH, TexUse::RENDER_ATTACHMENT)) } else { None },
        }
    }

    pub fn update(&mut self, gx:&Wgx, size:(u32, u32)) {

        let map_opt = |descriptor: &TexDsc<'a>| {
            let mut descriptor = descriptor.clone();
            descriptor.set_size_2d(size);
            descriptor.sample_count = self.msaa;
            TextureLot::new(gx, descriptor)
        };

        let TextureLot { texture, descriptor, view } = map_opt(&self.descriptor);
        self.texture = texture; self.descriptor = descriptor; self.view = view;

        self.msaa_opt = if self.msaa > 1 { self.msaa_opt.as_ref().map(|d| map_opt(&d.descriptor)).or_else(||
            Some(TextureLot::new_2d(gx, size, self.msaa, self.format(), TexUse::RENDER_ATTACHMENT))
        )}
        else { None };

        self.depth_opt = self.depth_opt.as_ref().map(|d| map_opt(&d.descriptor));
    }
}
