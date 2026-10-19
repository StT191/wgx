
use wgpu::{*, PresentMode as Prs};
use crate::*;


#[derive(Debug, Clone)]
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

    #[allow(clippy::misnamed_getters)]
    fn format(&self) -> TextureFormat { self.descriptor.view_format }
}

impl RenderAttachable for TextureLot {
    fn color_views(&self) -> (&wgpu::TextureView, wgpu::TextureFormat, Option<&wgpu::TextureView>) { (&self.view, self.format(), None) }
    fn depth_view(&self) -> Option<(&wgpu::TextureView, wgpu::TextureFormat)> { None }
}


// Surface Target
type Surface = wgpu::Surface<'static>;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum FrameState {
    Success,
    Suboptimal,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum FrameError {
    Discontinued,
    Timeout,
    Occluded,
    Outdated,
    Lost,
    Validation,
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FrameError::*;
        match self {
            Discontinued => formatter.write_str("The control flow was broken"),
            Timeout => formatter.write_str("The frame request timed out"),
            Occluded => formatter.write_str("The surface is occluded"),
            Outdated => formatter.write_str("The surface is outdated"),
            Lost => formatter.write_str("The surface was lost"),
            Validation => formatter.write_str("A validation error occurred"),
        }
    }
}

impl std::error::Error for FrameError {}

pub trait SurfaceStatus {
    fn needs_recreate(&self) -> bool;
    fn needs_config(&self) -> bool;
    fn inspect_msg(&self, handler: impl FnOnce(std::fmt::Arguments<'_>));
}

impl SurfaceStatus for Result<FrameState, FrameError> {

    fn needs_recreate(&self) -> bool { matches!(self, Err(FrameError::Lost) ) }
    fn needs_config(&self) -> bool { matches!(self, Ok(FrameState::Suboptimal) | Err(FrameError::Outdated) ) }

    fn inspect_msg(&self, handler: impl FnOnce(std::fmt::Arguments<'_>)) {
        match self {
            Ok(FrameState::Suboptimal) => handler(format_args!("The surface is suboptimal")),
            Err(err) => handler(format_args!("{err}")),
            _ => {},
        }
    }
}


#[derive(Debug)]
pub struct SurfaceMount {
    pub surface: Surface,
    pub frame: Option<SurfaceTexture>,
    needs_config: bool,
}

impl SurfaceMount {
    pub fn needs_config(&self) -> bool { self.needs_config }
}

impl Drop for SurfaceMount {
    fn drop(&mut self) {
        // drop SurfaceTexture before Surface
        self.frame = None;
    }
}


#[derive(Debug)]
pub struct SurfaceTarget {
    pub config: SurfaceConfiguration,
    pub mount: Option<SurfaceMount>,
    pub view_format: TextureFormat,
    pub msaa: u32,
    pub msaa_opt: Option<TextureLot>,
    pub depth_opt: Option<TextureLot>,
}

impl RenderTarget for SurfaceTarget {
    fn size(&self) -> [u32; 2] { [self.config.width, self.config.height] }
    fn msaa(&self) -> u32 { self.msaa }
    fn depth_testing(&self) -> Option<TextureFormat> { self.depth_opt.as_ref().map(|d| d.format()) }
    fn format(&self) -> TextureFormat { self.view_format }
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
}

impl RenderAttachable for SurfaceFrame<'_> {
    fn color_views(&self) -> (&wgpu::TextureView, wgpu::TextureFormat, Option<&wgpu::TextureView>) {
        (&self.view, self.format(), self.target.msaa_opt.as_ref().map(|o| &o.view))
    }
    fn depth_view(&self) -> Option<(&wgpu::TextureView, TextureFormat)> {
        self.target.depth_opt.as_ref().map(|d| (&d.view, d.format()))
    }
}



impl SurfaceTarget {

    pub fn new_with_default_config(
        gx:&Wgx, surface:Surface, size:impl Into<[u32; 2]>, srgb: bool,
        msaa:u32, depth_testing:Option<TextureFormat>,
    ) -> Self {

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

    pub fn new(
        gx:&impl WgxDevice, surface:Surface, config:SurfaceConfiguration,
        view_format:TextureFormat, msaa:u32, depth_testing:Option<TextureFormat>,
    ) -> Self {
        let mut target = Self {
            config, mount: Some(SurfaceMount { surface, frame: None, needs_config: true }),
            view_format, msaa, msaa_opt: None, depth_opt: None,
        };
        target.configure(gx, depth_testing).unwrap();
        target
    }

    pub fn configure(&mut self, gx:&impl WgxDevice, depth_testing:Option<TextureFormat>) -> Result<(), FrameError> {

        let Some(mount) = self.mount.as_mut() else {
            return Err(FrameError::Lost)
        };

        mount.frame = None; // drop current frame
        mount.surface.configure(gx.device(), &self.config);
        mount.needs_config = false;

        let [width, height] = self.size();

        self.msaa_opt = (self.msaa > 1).then(||
            TextureLot::new_2d(gx, [width, height, 1], self.msaa, self.format(), Some(self.view_format), TexUse::RENDER_ATTACHMENT)
        );

        self.depth_opt = depth_testing.map(|depth_format|
            TextureLot::new_2d(gx, [width, height, 1], self.msaa, depth_format, None, TexUse::RENDER_ATTACHMENT)
        );

        Ok(())
    }

    pub fn update(&mut self, gx:&impl WgxDevice, size:impl Into<[u32; 2]>) -> Result<(), FrameError> {
        let [width, height] = size.into();
        self.config.width = width;
        self.config.height = height;
        self.configure(gx, self.depth_testing())?;
        Ok(())
    }

    pub fn needs_config(&self) -> bool {
        match &self.mount { None => true, Some(m) => m.needs_config }
    }

    pub fn drop(&mut self) { self.mount = None; }

    pub fn drop_frame(&mut self) { self.mount.as_mut().map(|m| m.frame = None); }

    pub fn needs_recreate(&self) -> bool { self.mount.is_none() }

    pub fn recreate(&mut self, gx:&impl WgxDevice, create_surface:impl FnOnce() -> Surface, size:impl Into<[u32; 2]>) {
        self.mount = None;
        let surface = create_surface();
        self.mount = Some(SurfaceMount { surface, frame: None, needs_config: true });
        self.update(gx, size).unwrap();
    }


    pub fn request_frame(&mut self) -> Result<FrameState, FrameError> {

        let Some(mount) = self.mount.as_mut() else {
            return Err(FrameError::Lost)
        };

        mount.frame = None;

        use wgpu::CurrentSurfaceTexture::*;

        let result = match mount.surface.get_current_texture() {
            Success(texture) => { mount.frame = Some(texture); Ok(FrameState::Success) },
            Suboptimal(texture) => { mount.frame = Some(texture); Ok(FrameState::Suboptimal) },
            Timeout => Err(FrameError::Timeout),
            Occluded => Err(FrameError::Occluded),
            Outdated => Err(FrameError::Outdated),
            Lost => Err(FrameError::Lost),
            Validation => Err(FrameError::Validation),
        };

        if result.needs_recreate() {
            self.mount = None;
        } else {
            mount.needs_config = result.needs_config();
        }

        result
    }


    pub fn handle_frame<C: ImplicitControlFlow>(
        &mut self, dsc: Option<&wgpu::TextureViewDescriptor>, handler: impl FnOnce(&SurfaceFrame) -> C
    ) -> Option<C>
    {
        let frame = self.mount.as_ref()?.frame.as_ref()?;

        let res = handler(&SurfaceFrame {
            view: if let Some(dsc) = dsc {
                frame.texture.create_view(dsc)
            } else {
                frame.texture.create_default_view(Some(self.format()))
            },
            target: self,
        });

        Some(res)
    }


    pub fn present(&mut self, gx:&impl WgxQueue) -> Option<()> {
        self.mount.as_mut()?.frame.take().map(|frame| gx.queue().present(frame))
    }


    pub fn with_frame<C: ImplicitControlFlow>(
        &mut self, gx:&impl WgxQueue, dsc: Option<&wgpu::TextureViewDescriptor>,
        handler: impl FnOnce(&SurfaceFrame) -> C,
    ) -> Result<FrameState, FrameError>
    {
        match &self.mount {
            None => return Err(FrameError::Lost),
            Some(mount) => if mount.frame.is_none() {
                self.request_frame()?;
            }
        }

        let control_flow = self.handle_frame(dsc, handler).unwrap();

        if control_flow.should_continue() {

            self.present(gx).unwrap();

            if self.needs_config() { Ok(FrameState::Suboptimal) }
            else { Ok(FrameState::Success) }
        } else {
            Err(FrameError::Discontinued)
        }
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

    // present mode
    config.present_mode = Prs::Fifo;

    // alpha_modes
    if capabilites.alpha_modes.contains(&CompositeAlphaMode::Auto) { config.alpha_mode = CompositeAlphaMode::Auto; }

    // frame latency
    config.desired_maximum_frame_latency = 0;
}



#[derive(Debug, Clone)]
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
    fn depth_testing(&self) -> Option<TextureFormat> { self.depth_opt.as_ref().map(|d| d.format()) }

    #[allow(clippy::misnamed_getters)]
    fn format(&self) -> TextureFormat { self.descriptor.view_format }
}

impl RenderAttachable for TextureTarget {
    fn color_views(&self) -> (&wgpu::TextureView, wgpu::TextureFormat, Option<&wgpu::TextureView>) {
        (&self.view, self.format(), self.msaa_opt.as_ref().map(|o| &o.view))
    }
    fn depth_view(&self) -> Option<(&wgpu::TextureView, TextureFormat)> {
        self.depth_opt.as_ref().map(|d| (&d.view, d.format()))
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
