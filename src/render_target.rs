
use wgpu::{*, PresentMode as Prs};
use crate::{*, Color};
use anyhow::{Result as Res};


pub trait RenderTarget {

    // to implement
    fn size(&self) -> [u32; 2];
    fn msaa(&self) -> u32;
    fn depth_testing(&self) -> Option<TextureFormat>;
    fn format(&self) -> TextureFormat;
    fn view_format(&self) -> TextureFormat;

    fn render_bundle<'a>(&self, gx: &'a impl WgxDevice, handler: impl FnOnce(&mut wgpu::RenderBundleEncoder<'a>)) -> wgpu::RenderBundle {
        gx.render_bundle(&[Some(self.view_format())], self.depth_testing(), self.msaa(), handler)
    }

    fn render_pipeline(
        &self, gx: &impl WgxDevice,
        layout: Option<(&[wgpu::PushConstantRange], &[&wgpu::BindGroupLayout])>,
        buffers: &[wgpu::VertexBufferLayout],
        vertex_state: (&wgpu::ShaderModule, &str, Primitive),
        (fs_module, fs_entry_point, blend): (&wgpu::ShaderModule, &str, Option<Blend>),
    ) -> wgpu::RenderPipeline {
        gx.render_pipeline(
            self.msaa(), self.depth_testing(), layout, buffers, vertex_state,
            Some((fs_module, fs_entry_point, &[(self.view_format(), blend)])),
        )
    }
}

pub trait RenderAttachable: RenderTarget {

    // to implement
    fn color_views(&self) -> (&wgpu::TextureView, Option<&wgpu::TextureView>);
    fn depth_view(&self) -> Option<(&wgpu::TextureView, wgpu::TextureFormat)>;

    // provided
    fn color_attachment(&self, clear_color: Option<Color>) -> wgpu::RenderPassColorAttachment {
        let (view, msaa) = self.color_views();
        ColorAttachment { view, msaa, clear: clear_color }.into()
    }

    fn depth_attachment(&self, clear_depth: Option<f32>, clear_stencil: Option<u32>) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        self.depth_view().map(|(view, format)| DepthAttachment { view, format, clear_depth, clear_stencil }.into())
    }

    fn attachments(&self, clear_color: Option<Color>, clear_depth: Option<f32>, clear_stencil: Option<u32>) -> RenderAttachments<1> {
        ([Some(self.color_attachment(clear_color))], self.depth_attachment(clear_depth, clear_stencil))
    }
}


// helper
#[derive(Debug)]
pub struct TextureLot {
    pub texture: wgpu::Texture,
    pub descriptor: TexDsc,
    pub view: wgpu::TextureView,
}

impl TextureLot {
    pub fn new(gx:&impl WgxDevice, descriptor: TexDsc) -> Self {
        let texture = gx.texture(&descriptor);
        let view = texture.create_view(&descriptor.default_view());
        TextureLot { texture, descriptor, view }
    }
    pub fn new_with_data<T: ReadBytes>(gx:&impl WgxDeviceQueue, descriptor: TexDsc, data: T) -> Self {
        let texture = gx.texture_with_data(&descriptor, data);
        let view = texture.create_view(&descriptor.default_view());
        TextureLot { texture, descriptor, view }
    }
    pub fn new_2d(
        gx:&impl WgxDevice, size:[u32; 3], sample_count:u32,
        format:TextureFormat, view_format:Option<TextureFormat>, usage:TexUse
    ) -> Self {
        Self::new(gx, TexDsc::new_2d(size, sample_count, format, view_format, usage))
    }
    pub fn new_2d_with_data<T: ReadBytes>(
        gx:&impl WgxDeviceQueue, size:[u32; 3], sample_count:u32,
        format:TextureFormat, view_format:Option<TextureFormat>, usage:TexUse, data: T,
    ) -> Self {
        Self::new_with_data(gx, TexDsc::new_2d(size, sample_count, format, view_format, usage), data)
    }
    pub fn update_view(&mut self) {
        self.view = self.texture.create_view(&self.descriptor.default_view());
    }
}

impl RenderTarget for TextureLot {
    fn size(&self) -> [u32; 2] { [self.texture.width(), self.texture.height()] }
    fn msaa(&self) -> u32 { 1 }
    fn depth_testing(&self) -> Option<TextureFormat> { None }
    fn format(&self) -> TextureFormat { self.descriptor.format }
    fn view_format(&self) -> TextureFormat { self.descriptor.view_format }
}

impl RenderAttachable for TextureLot {
    fn color_views(&self) -> (&wgpu::TextureView, Option<&wgpu::TextureView>) { (&self.view, None) }
    fn depth_view(&self) -> Option<(&wgpu::TextureView, wgpu::TextureFormat)> { None }
}


type Surface = wgpu::Surface<'static>;

#[derive(Debug)]
pub struct SurfaceTarget {
    pub config: SurfaceConfiguration,
    pub surface: Surface,
    pub view_format: TextureFormat,
    pub msaa: u32,
    pub msaa_opt: Option<TextureLot>,
    pub depth_opt: Option<TextureLot>,
}

impl RenderTarget for SurfaceTarget {
    fn size(&self) -> [u32; 2] { [self.config.width, self.config.height] }
    fn msaa(&self) -> u32 { self.msaa }
    fn depth_testing(&self) -> Option<TextureFormat> { self.depth_opt.as_ref().map(|d| d.view_format()) }
    fn format(&self) -> TextureFormat { self.config.format }
    fn view_format(&self) -> TextureFormat { self.view_format }
}


#[derive(Debug)]
pub struct SurfaceFrame<'a> {
    pub target: &'a mut SurfaceTarget,
    pub view: wgpu::TextureView,
}

impl RenderTarget for SurfaceFrame<'_> {
    fn size(&self) -> [u32; 2] { self.target.size() }
    fn msaa(&self) -> u32 { self.target.msaa() }
    fn depth_testing(&self) -> Option<TextureFormat> { self.target.depth_testing() }
    fn format(&self) -> TextureFormat { self.target.format() }
    fn view_format(&self) -> TextureFormat { self.target.view_format() }
}

impl RenderAttachable for SurfaceFrame<'_> {
    fn color_views(&self) -> (&wgpu::TextureView, Option<&wgpu::TextureView>) {
        (&self.view, self.target.msaa_opt.as_ref().map(|o| &o.view))
    }
    fn depth_view(&self) -> Option<(&wgpu::TextureView, TextureFormat)> {
        self.target.depth_opt.as_ref().map(|d| (&d.view, d.view_format()))
    }
}


pub fn configure_surface_defaults(
    config: &mut SurfaceConfiguration, capabilites: &SurfaceCapabilities,
    downlevel_flags: &DownlevelFlags, srgb: bool,
) {
    // format
    if let Some(format) = capabilites.formats.iter().find(|fmt| fmt.is_srgb() == srgb) { config.format = *format; }

    let other_format = if config.format.is_srgb() {
        config.format.remove_srgb_suffix()
    } else {
        config.format.add_srgb_suffix()
    };

    if downlevel_flags.contains(DownlevelFlags::SURFACE_VIEW_FORMATS) && !config.view_formats.contains(&other_format) {
        config.view_formats.push(other_format);
    }

    // present modes
    if capabilites.present_modes.contains(&Prs::Mailbox) { config.present_mode = Prs::Mailbox; }
    else if capabilites.present_modes.contains(&Prs::AutoVsync) { config.present_mode = Prs::AutoVsync; }

    // alpha_modes
    if capabilites.alpha_modes.contains(&CompositeAlphaMode::Auto) { config.alpha_mode = CompositeAlphaMode::Auto; }

    // latency
    config.desired_maximum_frame_latency = 2;
}


impl SurfaceTarget {

    pub fn new_with_default_config(gx:&Wgx, surface:Surface, size:impl Into<[u32; 2]>, srgb: bool, msaa:u32, depth_testing:Option<TextureFormat>) -> Self
    {
        let [width, height] = size.into();
        let mut config = surface.get_default_config(&gx.adapter, width, height).unwrap();

        configure_surface_defaults(
            &mut config, &surface.get_capabilities(&gx.adapter),
            &gx.adapter.get_downlevel_capabilities().flags, srgb,
        );

        let format = config.format;
        let view_format = if srgb { format.add_srgb_suffix() } else { format.remove_srgb_suffix() };

        if view_format != format {
            assert!(config.view_formats.contains(&view_format), "view_formats may not be supported");
        }

        Self::new(gx, surface, config, view_format, msaa, depth_testing)
    }


    pub fn new(gx:&impl WgxDevice, surface:Surface, config:SurfaceConfiguration, view_format:TextureFormat, msaa:u32, depth_testing:Option<TextureFormat>)
        -> Self
    {
        let mut target = Self { config, surface, view_format, msaa, msaa_opt: None, depth_opt: None };
        target.configure(gx, depth_testing);
        target
    }


    pub fn configure(&mut self, gx:&impl WgxDevice, depth_testing:Option<TextureFormat>) {

        let [width, height] = self.size();

        self.surface.configure(gx.device(), &self.config);

        self.msaa_opt = (self.msaa > 1).then(||
            TextureLot::new_2d(gx, [width, height, 1], self.msaa, self.format(), Some(self.view_format), TexUse::RENDER_ATTACHMENT)
        );

        self.depth_opt = depth_testing.map(|depth_format|
            TextureLot::new_2d(gx, [width, height, 1], self.msaa, depth_format, None, TexUse::RENDER_ATTACHMENT)
        );
    }


    pub fn update(&mut self, gx:&impl WgxDevice, size:impl Into<[u32; 2]>) {
        let [width, height] = size.into();
        self.config.width = width;
        self.config.height = height;
        self.configure(gx, self.depth_testing());
    }


    pub fn with_frame<T: ImplicitControlFlow>(
        &mut self, dsc: Option<&wgpu::TextureViewDescriptor>, handler: impl FnOnce(&SurfaceFrame) -> T
    ) -> Res<T>
    {
        let frame = self.surface.get_current_texture()?;

        let res = handler(&SurfaceFrame {
            view: if let Some(dsc) = dsc {
                frame.texture.create_view(dsc)
            } else {
                frame.texture.create_default_view(Some(self.view_format()))
            },
            target: self,
        });

        if res.should_continue() {
            frame.present();
        }

        Ok(res)
    }
}



#[derive(Debug)]
pub struct TextureTarget {
    pub texture: wgpu::Texture,
    pub descriptor: TexDsc,
    pub view: wgpu::TextureView,

    pub msaa: u32,
    pub msaa_opt: Option<TextureLot>,
    pub depth_opt: Option<TextureLot>,
}

impl RenderTarget for TextureTarget {
    fn size(&self) -> [u32; 2] { self.descriptor.size_2d() }
    fn msaa(&self) -> u32 { self.msaa }
    fn depth_testing(&self) -> Option<TextureFormat> { self.depth_opt.as_ref().map(|d| d.view_format()) }
    fn format(&self) -> TextureFormat { self.descriptor.format }
    fn view_format(&self) -> TextureFormat { self.descriptor.view_format }
}

impl RenderAttachable for TextureTarget {
    fn color_views(&self) -> (&wgpu::TextureView, Option<&wgpu::TextureView>) {
        (&self.view, self.msaa_opt.as_ref().map(|o| &o.view))
    }
    fn depth_view(&self) -> Option<(&wgpu::TextureView, TextureFormat)> {
        self.depth_opt.as_ref().map(|d| (&d.view, d.view_format()))
    }
}

impl TextureTarget {
    pub fn new(
        gx:&impl WgxDevice, size:impl Into<[u32; 2]>, msaa:u32, depth_testing:Option<TextureFormat>,
        format:TextureFormat, view_format:Option<TextureFormat>, usage:wgpu::TextureUsages,
    ) -> Self
    {
        let [w, h] = size.into();
        let TextureLot { texture, descriptor, view } = TextureLot::new_2d(gx, [w, h, 1], 1, format, view_format, usage | TexUse::RENDER_ATTACHMENT);
        Self {
            texture, descriptor, view, msaa, // output attachment can have only one sample

            msaa_opt: (msaa > 1).then(||
                TextureLot::new_2d(gx, [w, h, 1], msaa, format, view_format, TexUse::RENDER_ATTACHMENT)
            ),

            depth_opt: depth_testing.map(|depth_format|
                TextureLot::new_2d(gx, [w, h, 1], msaa, depth_format, None, TexUse::RENDER_ATTACHMENT)
            ),
        }
    }
}
