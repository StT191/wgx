
use crate::{Wgx, TEXTURE, TexUse, DefaultViewExtension, error::*};
use wgpu::PresentMode as Prs;


// cloneable surface configuration
const SURFACE_CONFIGURATION:wgpu::SurfaceConfiguration = wgpu::SurfaceConfiguration {
    usage: TexUse::RENDER_ATTACHMENT,
    format: TEXTURE, width: 0, height: 0,
    present_mode: Prs::Mailbox,
};


pub struct RenderAttachment<'a> {
    pub view: &'a wgpu::TextureView,
    pub depth: Option<&'a wgpu::TextureView>,
    pub msaa: Option<&'a wgpu::TextureView>,
    pub format: wgpu::TextureFormat,
}

impl RenderAttachment<'_> {
    pub fn srgb(&self) -> bool { self.format.describe().srgb }
}


pub trait RenderTarget {

    fn attachment(&self) -> Res<RenderAttachment>;

    fn format(&self) -> wgpu::TextureFormat;
    fn size(&self) -> (u32, u32);
    fn depth_testing(&self) -> bool;
    fn msaa(&self) -> u32;

    fn srgb(&self) -> bool { self.format().describe().srgb }

    fn render_bundle_encoder<'a>(&self, wgx:&'a Wgx) -> wgpu::RenderBundleEncoder<'a> {
        wgx.render_bundle_encoder(&[Some(self.format())], self.depth_testing(), self.msaa())
    }

    fn render_bundle<'a>(&self, wgx:&'a Wgx, handler: impl FnOnce(&mut wgpu::RenderBundleEncoder<'a>)) -> wgpu::RenderBundle {
        let mut encoder = self.render_bundle_encoder(wgx);
        handler(&mut encoder);
        encoder.finish(&wgpu::RenderBundleDescriptor::default())
    }

    fn render_pipeline(
        &self, wgx:&Wgx, alpha_blend:bool,
        (vs_module, vs_entry_point):(&wgpu::ShaderModule, &str), (fs_module, fs_entry_point):(&wgpu::ShaderModule, &str),
        vertex_layouts:&[wgpu::VertexBufferLayout], topology:wgpu::PrimitiveTopology,
        layout:Option<(&[wgpu::PushConstantRange], &wgpu::BindGroupLayout)>
    ) -> wgpu::RenderPipeline {
        wgx.render_pipeline(
            self.format(), self.depth_testing(), self.msaa(), alpha_blend,
            (vs_module, vs_entry_point), (fs_module, fs_entry_point),
            vertex_layouts, topology, layout
        )
    }
}



pub struct TextureTarget {

    format: wgpu::TextureFormat,

    size: (u32, u32),
    depth_testing: bool,
    msaa: u32,

    // texture / view
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,

    depth_texture: Option<wgpu::Texture>,
    depth_texture_view: Option<wgpu::TextureView>,

    msaa_texture: Option<wgpu::Texture>,
    msaa_texture_view: Option<wgpu::TextureView>,
}


impl RenderTarget for TextureTarget {

    fn attachment(&self) -> Res<RenderAttachment> {
        Ok(RenderAttachment {
            view: &self.texture_view,
            depth: self.depth_texture_view.as_ref(),
            msaa: self.msaa_texture_view.as_ref(),
            format: self.format,
        })
    }

    fn format(&self) -> wgpu::TextureFormat { self.format }
    fn size(&self) -> (u32, u32) { self.size }
    fn depth_testing(&self) -> bool { self.depth_testing }
    fn msaa(&self) -> u32 { self.msaa }
}



impl TextureTarget {

    pub fn texture(&self) -> &wgpu::Texture { &self.texture }
    pub fn depth_texture(&self) -> Option<&wgpu::Texture> { self.depth_texture.as_ref() }
    pub fn msaa_texture(&self) -> Option<&wgpu::Texture> { self.msaa_texture.as_ref() }

    pub fn new(wgx:&Wgx,
        size:(u32, u32), depth_testing: bool, msaa:u32, usage:wgpu::TextureUsages, format:wgpu::TextureFormat
    ) -> Self
    {
        // sample count is always one for output attachments
        let texture = wgx.texture(size, 1, TexUse::RENDER_ATTACHMENT | usage, format);
        Self::from_texture(wgx, texture, size, depth_testing, msaa, format)
    }

    pub fn from_texture(wgx:&Wgx,
        texture:wgpu::Texture, size:(u32, u32), depth_testing:bool, msaa:u32, format:wgpu::TextureFormat
    ) -> Self
    {
        let texture_view = texture.create_default_view();

        let (depth_texture, depth_texture_view) = create_depth_option(wgx, size, depth_testing, msaa);
        let (msaa_texture, msaa_texture_view) = create_msaa_option(wgx, size, msaa, format);

        Self {
            format, size, depth_testing, msaa,
            texture, texture_view,
            depth_texture, depth_texture_view,
            msaa_texture, msaa_texture_view,
        }
    }

    pub fn from_texture_and_depth(wgx:&Wgx,
        texture:wgpu::Texture, depth_texture:wgpu::Texture, size:(u32, u32), msaa:u32, format:wgpu::TextureFormat
    ) -> Self
    {
        let texture_view = texture.create_default_view();

        let depth_texture_view = Some(depth_texture.create_default_view());
        let (msaa_texture, msaa_texture_view) = create_msaa_option(wgx, size, msaa, format);

        Self {
            format, size, depth_testing: true, msaa,
            texture, texture_view,
            depth_texture: Some(depth_texture), depth_texture_view,
            msaa_texture, msaa_texture_view,
        }
    }

    pub fn downgrade(self) -> (wgpu::Texture, Option<wgpu::Texture>) {
        (self.texture, self.depth_texture)
    }
}



pub struct SurfaceTarget {

    config: wgpu::SurfaceConfiguration,

    // size: (u32, u32),
    depth_testing: bool,
    msaa: u32,

    // texture / view
    surface: wgpu::Surface,
    current_frame: Option<wgpu::SurfaceTexture>,
    current_frame_view: Option<wgpu::TextureView>,

    depth_texture: Option<wgpu::Texture>,
    depth_texture_view: Option<wgpu::TextureView>,

    msaa_texture: Option<wgpu::Texture>,
    msaa_texture_view: Option<wgpu::TextureView>,
}


impl RenderTarget for SurfaceTarget {

    fn attachment(&self) -> Res<RenderAttachment> {
        Ok(RenderAttachment {
            view: self.current_frame_view.as_ref().ok_or("no current frame view")?,
            depth: self.depth_texture_view.as_ref(),
            msaa: self.msaa_texture_view.as_ref(),
            format: self.config.format,
        })
    }

    fn format(&self) -> wgpu::TextureFormat { self.config.format }
    fn size(&self) -> (u32, u32) { (self.config.width, self.config.height) }
    fn depth_testing(&self) -> bool { self.depth_testing }
    fn msaa(&self) -> u32 { self.msaa }
}


impl SurfaceTarget {

    pub(super) fn new(wgx:&Wgx, surface:wgpu::Surface, (width, height):(u32, u32), depth_testing:bool, msaa:u32) -> Res<Self>
    {
        let mut config = SURFACE_CONFIGURATION.clone();
        config.width = width;
        config.height = height;

        let formats = surface.get_supported_formats(&wgx.adapter);

        config.format = *formats.iter().find(|fmt| fmt.describe().srgb).ok_or("couldn't get srgb format")?;

        let modes = surface.get_supported_modes(&wgx.adapter);

        config.present_mode =
            if modes.contains(&Prs::Mailbox) { Prs::Mailbox }
            else if modes.contains(&Prs::AutoVsync) { Prs::AutoVsync }
            else { *modes.get(0).ok_or("couldn't get default mode")? }
        ;


        surface.configure(&wgx.device, &config);

        let mut target = Self {
            config, depth_testing, msaa,
            surface, current_frame: None, current_frame_view: None,
            depth_texture: None, depth_texture_view: None,
            msaa_texture: None, msaa_texture_view: None,
        };

        if depth_testing || msaa > 1 { target.update(wgx, (width, height)); }

        Ok(target)
    }

    pub fn update(&mut self, wgx:&Wgx, (width, height):(u32, u32)) {

        self.config.width = width;
        self.config.height = height;

        self.surface.configure(&wgx.device, &self.config);

        let (depth_texture, depth_texture_view) = create_depth_option(wgx, self.size(), self.depth_testing, self.msaa);
        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;

        let (msaa_texture, msaa_texture_view) = create_msaa_option(wgx, self.size(), self.msaa, self.format());
        self.msaa_texture = msaa_texture;
        self.msaa_texture_view = msaa_texture_view;
    }

    pub fn with_encoder_frame<T>(
        &mut self, wgx:&Wgx, handler: impl FnOnce(&mut wgpu::CommandEncoder, &RenderAttachment) -> T
    ) -> Res<T>
    {
        self.current_frame = Some(self.surface.get_current_texture().map_err(error)?);
        self.current_frame_view = Some(self.current_frame.as_ref().unwrap().texture.create_default_view());

        let attachment = self.attachment()?;

        let res = wgx.with_encoder(|mut encoder| handler(&mut encoder, &attachment));

        self.current_frame_view = None;

        let current_frame = self.current_frame.take().ok_or("couldn't get current frame")?;
        current_frame.present();

        Ok(res)
    }
}



// helper
fn create_depth_option(wgx:&Wgx, size:(u32, u32), depth_testing:bool, msaa:u32) ->
    (Option<wgpu::Texture>, Option<wgpu::TextureView>)
{
    if depth_testing {
        let depth_texture = wgx.depth_texture(size, msaa);
        let depth_texture_view = depth_texture.create_default_view();
        (Some(depth_texture), Some(depth_texture_view))
    } else {
        (None, None)
    }
}

fn create_msaa_option(wgx:&Wgx, size:(u32, u32), msaa:u32, format:wgpu::TextureFormat) ->
    (Option<wgpu::Texture>, Option<wgpu::TextureView>)
{
    if msaa > 1 {
        let msaa_texture = wgx.msaa_texture(size, msaa, format);
        let msaa_texture_view = msaa_texture.create_default_view();
        (Some(msaa_texture), Some(msaa_texture_view))
    } else {
        (None, None)
    }
}
