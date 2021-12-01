
use crate::{Wgx, OUTPUT, TexUse, DefaultViewExtension};


// cloneable surface configuration
const SURFACE_CONFIGURATION:wgpu::SurfaceConfiguration = wgpu::SurfaceConfiguration {
    usage: TexUse::RENDER_ATTACHMENT,
    format: OUTPUT, width: 0, height: 0,
    present_mode: wgpu::PresentMode::Mailbox,
};


pub struct RenderAttachment<'a> {
    pub view: &'a wgpu::TextureView,
    pub depth: Option<&'a wgpu::TextureView>,
    pub msaa: Option<&'a wgpu::TextureView>,
}


pub trait RenderTarget {

    fn attachment(&self) -> RenderAttachment;

    fn format(&self) -> wgpu::TextureFormat;
    fn size(&self) -> (u32, u32);
    fn depth_testing(&self) -> bool;
    fn msaa(&self) -> u32;

    fn render_bundle_encoder<'a>(&self, wgx:&'a Wgx) -> wgpu::RenderBundleEncoder<'a> {
        wgx.render_bundle_encoder(&[self.format()], self.depth_testing(), self.msaa())
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
        layout:Option<(&[wgpu::PushConstantRange], &[&wgpu::BindGroupLayout])>
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

    fn attachment(&self) -> RenderAttachment {
        RenderAttachment {
            view: &self.texture_view,
            depth: self.depth_texture_view.as_ref(),
            msaa: self.msaa_texture_view.as_ref(),
        }
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

    fn attachment(&self) -> RenderAttachment {
        RenderAttachment {
            view: &self.current_frame_view.as_ref().expect("no current frame view"),
            depth: self.depth_texture_view.as_ref(),
            msaa: self.msaa_texture_view.as_ref(),
        }
    }

    fn format(&self) -> wgpu::TextureFormat { OUTPUT }
    fn size(&self) -> (u32, u32) { (self.config.width, self.config.height) }
    fn depth_testing(&self) -> bool { self.depth_testing }
    fn msaa(&self) -> u32 { self.msaa }
}


impl SurfaceTarget {

    pub(super) fn new(wgx:&Wgx, surface:wgpu::Surface, (width, height):(u32, u32), depth_testing:bool, msaa:u32)
        -> Result<Self, String>
    {
        let mut config = SURFACE_CONFIGURATION.clone();
        config.width = width;
        config.height = height;

        config.format = surface.get_preferred_format(&wgx.adapter).expect("couldn't get default format");

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

    pub fn with_encoder_frame(
        &mut self, wgx:&Wgx, handler: impl FnOnce(&mut wgpu::CommandEncoder, &RenderAttachment)
    ) -> Result<(), wgpu::SurfaceError>
    {
        self.current_frame = Some(self.surface.get_current_texture()?);
        self.current_frame_view = Some(self.current_frame.as_ref().unwrap().texture.create_default_view());

        wgx.with_encoder(|mut encoder| handler(&mut encoder, &self.attachment()));

        self.current_frame_view = None;

        let current_frame = self.current_frame.take().unwrap();
        current_frame.present();

        Ok(())
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
