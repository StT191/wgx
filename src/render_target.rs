
use crate::{Wgx, OUTPUT, TexUse, DefaultViewExtension};


// cloneable swapchain descriptor
const SWAP_CHAIN_DESC:wgpu::SwapChainDescriptor = wgpu::SwapChainDescriptor {
    usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    format: OUTPUT, width: 0, height: 0,
    present_mode: wgpu::PresentMode::Mailbox,
};


pub type RenderAttachment<'a> = (&'a wgpu::TextureView, Option<&'a wgpu::TextureView>, Option<&'a wgpu::TextureView>);


pub trait RenderTarget {

    fn attachment(&self) -> RenderAttachment;

    fn format(&self) -> wgpu::TextureFormat;
    fn size(&self) -> (u32, u32);
    fn depth_testing(&self) -> bool;
    fn msaa(&self) -> u32;

    fn render_pipeline(
        &self, wgx: &Wgx, alpha_blend:bool, vs_module:&wgpu::ShaderModule, fs_module:&wgpu::ShaderModule,
        vertex_layout:wgpu::VertexBufferDescriptor, topology:wgpu::PrimitiveTopology,
        bind_group_layout:&wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        wgx.render_pipeline(
            self.format(), self.depth_testing(), self.msaa(), alpha_blend,
            vs_module, fs_module, vertex_layout, topology, bind_group_layout
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
        (
            &self.texture_view,
            self.depth_texture_view.as_ref(),
            self.msaa_texture_view.as_ref(),
        )
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
        (width, height):(u32, u32), depth_testing: bool, msaa:u32,
        usage:wgpu::TextureUsage, format:wgpu::TextureFormat
    ) -> Self
    {
        // sample count is always one for output attachments
        let texture = wgx.texture((width, height), 1, TexUse::OUTPUT_ATTACHMENT | usage, format);

        let texture_view = texture.create_default_view();


        let (depth_texture, depth_texture_view) = if depth_testing {
            let depth_texture = wgx.depth_texture((width, height), msaa);
            let depth_texture_view = depth_texture.create_default_view();
            (Some(depth_texture), Some(depth_texture_view))
        } else {
            (None, None)
        };

        let (msaa_texture, msaa_texture_view) = if msaa > 1 {
            let msaa_texture = wgx.msaa_texture((width, height), msaa, format);
            let msaa_texture_view = msaa_texture.create_default_view();
            (Some(msaa_texture), Some(msaa_texture_view))
        } else {
            (None, None)
        };

        Self {
            format, size: (width, height), depth_testing, msaa,
            texture, texture_view,
            depth_texture, depth_texture_view,
            msaa_texture, msaa_texture_view,
        }
    }
}



pub struct SurfaceTarget {
    size: (u32, u32),
    depth_testing: bool,
    msaa: u32,

    // texture / view
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    current_frame: Option<wgpu::SwapChainFrame>,

    depth_texture: Option<wgpu::Texture>,
    depth_texture_view: Option<wgpu::TextureView>,

    msaa_texture: Option<wgpu::Texture>,
    msaa_texture_view: Option<wgpu::TextureView>,
}


impl RenderTarget for SurfaceTarget {

    fn attachment(&self) -> RenderAttachment {
        (
            &self.current_frame.as_ref().expect("no current frame").output.view,
            self.depth_texture_view.as_ref(),
            self.msaa_texture_view.as_ref(),
        )
    }

    fn format(&self) -> wgpu::TextureFormat { OUTPUT }
    fn size(&self) -> (u32, u32) { self.size }
    fn depth_testing(&self) -> bool { self.depth_testing }
    fn msaa(&self) -> u32 { self.msaa }
}


impl SurfaceTarget {

    pub(super) fn new(wgx:&Wgx, surface:wgpu::Surface, (width, height):(u32, u32), depth_testing:bool, msaa:u32)
        -> Result<Self, String>
    {
        let mut sc_desc = SWAP_CHAIN_DESC.clone();
        sc_desc.width = width;
        sc_desc.height = height;

        let swap_chain = wgx.device.create_swap_chain(&surface, &sc_desc);


        let mut target = Self {
            size: (width, height), depth_testing, msaa,
            surface, swap_chain, current_frame: None,
            depth_texture: None, depth_texture_view: None,
            msaa_texture: None, msaa_texture_view: None,
        };

        if depth_testing || msaa > 1 { target.update(wgx, (width, height)); }

        Ok(target)
    }


    pub fn update(&mut self, wgx:&Wgx, (width, height):(u32, u32)) {
        self.size = (width, height);

        let mut sc_desc = SWAP_CHAIN_DESC.clone();
        sc_desc.width = width;
        sc_desc.height = height;

        self.swap_chain = wgx.device.create_swap_chain(&self.surface, &sc_desc);

        if self.depth_testing {
            let depth_texture = wgx.depth_texture((width, height), self.msaa);
            self.depth_texture_view = Some(depth_texture.create_default_view());
            self.depth_texture = Some(depth_texture);
        }

        if self.msaa > 1 {
            let msaa_texture = wgx.msaa_texture((width, height), self.msaa, OUTPUT);
            self.msaa_texture_view = Some(msaa_texture.create_default_view());
            self.msaa_texture = Some(msaa_texture);
        }
    }


    pub fn with_encoder_frame<'a, F>(&mut self, wgx:&Wgx, handler: F) -> Result<(), wgpu::SwapChainError>
        where F: 'a + FnOnce(&mut wgpu::CommandEncoder, RenderAttachment)
    {
        self.current_frame = Some(self.swap_chain.get_current_frame()?);

        wgx.with_encoder(|mut encoder| { handler(&mut encoder, self.attachment()) });

        self.current_frame = None;

        Ok(())
    }

}

