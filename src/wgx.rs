
use arrayvec::ArrayVec;
use wgpu::util::{DeviceExt, TextureDataOrder};
use std::{ops::{RangeBounds, Bound}, borrow::Cow};
use crate::*;
use anyhow::{Result as Res, Context, anyhow};


// wgx
#[derive(Debug)]
pub struct Wgx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
}

impl Wgx {
    pub fn instance() -> wgpu::Instance {
        wgpu::Instance::new(Default::default())
    }

    pub async fn request_adapter<W: Into<wgpu::SurfaceTarget<'static>>>(
        instance: &wgpu::Instance, window: Option<W>
    )
        -> Res<(wgpu::Adapter, Option<wgpu::Surface<'static>>)>
    {
        let surface = if let Some(win) = window {
            Some(instance.create_surface(win)?)
        }
        else { None };

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: surface.as_ref(),
        }).await.context("couldn't get adapter")?;

        Ok((adapter, surface))
    }

    pub async fn request_device(adapter: &wgpu::Adapter, features:wgpu::Features, limits:wgpu::Limits) -> Res<(wgpu::Device, wgpu::Queue)> {

        let adapter_limits = adapter.limits();

        #[cfg(target_family = "wasm")] let limits = limits.using_resolution(adapter_limits.clone());

        let limits = wgpu::Limits {
            // choose the smallest uniform offset alignment possible
            min_uniform_buffer_offset_alignment: adapter_limits.min_uniform_buffer_offset_alignment,
            min_storage_buffer_offset_alignment: adapter_limits.min_storage_buffer_offset_alignment,
            ..limits
        };

        adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: features,
                required_limits: limits,
                memory_hints: Default::default(),
            },
            None,
        ).await.map_err(|err| anyhow!("{err:?}"))
    }

    pub async fn new<W: Into<wgpu::SurfaceTarget<'static>>>(
        window:Option<W>, features:wgpu::Features, limits:wgpu::Limits
    )
        -> Res<(Self, Option<wgpu::Surface<'static>>)>
    {
        let instance = Self::instance();
        let (adapter, surface) = Self::request_adapter(&instance, window).await?;
        let (device, queue) = Self::request_device(&adapter, features, limits).await?;
        Ok((Self {device, queue, instance, adapter}, surface))
    }

    pub async fn new_with_target<W: Into<wgpu::SurfaceTarget<'static>>>(
        window: W, features:wgpu::Features, limits:wgpu::Limits, window_size:impl Into<[u32; 2]>, srgb: bool, msaa:u32, depth_testing:Option<TexFmt>,
    )
        -> Res<(Self, SurfaceTarget)>
    {
        let (gx, surface) = Wgx::new(Some(window), features, limits).await?;
        let target = SurfaceTarget::new_with_default_config(&gx, surface.unwrap(), window_size, srgb, msaa, depth_testing);
        Ok((gx, target))
    }
}


// device methods
pub trait WgxDevice {

    fn device(&self) -> &wgpu::Device;

    // texture, sampler

    fn texture(&self, descriptor:&TexDsc) -> wgpu::Texture {
        self.device().create_texture(&descriptor.into())
    }

    fn sampler(&self, descriptor: &wgpu::SamplerDescriptor) -> wgpu::Sampler {
        self.device().create_sampler(descriptor)
    }

    fn default_sampler(&self) -> wgpu::Sampler {
        self.device().create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            border_color: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None, // Some(wgpu::CompareFunction::LessEqual),
            anisotropy_clamp: 1,
        })
    }


    // buffer

    fn buffer(&self, usage:BufUse, size:u64, mapped_at_creation:bool) -> wgpu::Buffer {
        self.device().create_buffer(&wgpu::BufferDescriptor {usage, size, mapped_at_creation, label: None})
    }

    fn buffer_from_data<T: ReadBytes>(&self, usage:BufUse, data:T) -> wgpu::Buffer {
        self.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            usage, contents: data.read_bytes(), label: None
        })
    }


    // shader

    fn load_wgsl<'a>(&self, code: impl Into<Cow<'a, str>>) -> wgpu::ShaderModule {
        let source = wgpu::ShaderSource::Wgsl(code.into());
        self.device().create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source })
    }


    // bind group

    fn layout(&self, entries:&[wgpu::BindGroupLayoutEntry]) -> wgpu::BindGroupLayout {
        self.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries, label: None
        })
    }

    fn bind(&self, layout:&wgpu::BindGroupLayout, entries:&[wgpu::BindGroupEntry]) -> wgpu::BindGroup {
        self.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout, entries, label: None
        })
    }


    fn command_encoder (&self) -> wgpu::CommandEncoder {
        self.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
    }


    // render bundle

    fn render_bundle_encoder(&self, formats: &[Option<TexFmt>], depth_testing:Option<TexFmt>, msaa:u32) -> wgpu::RenderBundleEncoder {
        self.device().create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: formats,
            depth_stencil: depth_testing.map(|format| wgpu::RenderBundleDepthStencil {
                format, depth_read_only: false, stencil_read_only: true,
            }),
            sample_count: msaa,
            multiview: None,
        })
    }

    fn render_bundle<'a>(&'a self,
        formats: &[Option<TexFmt>], depth_testing:Option<TexFmt>, msaa:u32,
        handler: impl FnOnce(&mut wgpu::RenderBundleEncoder<'a>),
    )
        -> wgpu::RenderBundle
    {
        self.render_bundle_encoder(formats, depth_testing, msaa).record(handler)
    }


    // pipelines

    fn pipeline_layout(&self, constants: &[wgpu::PushConstantRange], bind_groups: &[&wgpu::BindGroupLayout]) -> wgpu::PipelineLayout {
        self.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None, push_constant_ranges: constants, bind_group_layouts: bind_groups,
        })
    }


    fn compute_pipeline(
        &self,
        layout:Option<(&[wgpu::PushConstantRange], &[&wgpu::BindGroupLayout])>,
        (module, entry_point, constants):(&wgpu::ShaderModule, &str, Option<&ShaderConstants>),
    ) -> wgpu::ComputePipeline {
        self.device().create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            cache: None,
            layout: layout.map(|(c, b)| self.pipeline_layout(c, b)).as_ref(),
            module,
            entry_point: if entry_point.is_empty() { None } else { Some(entry_point) },
            compilation_options: wgpu::PipelineCompilationOptions {
                zero_initialize_workgroup_memory: false,
                constants: constants.unwrap_or(wgpu::PipelineCompilationOptions::default().constants),
            },
        })
    }


    fn render_pipeline<const S: usize>(
        &self,
        msaa: u32, depth_testing: Option<TexFmt>,
        layout: Option<(&[wgpu::PushConstantRange], &[&wgpu::BindGroupLayout])>,
        buffers: &[wgpu::VertexBufferLayout],
        (module, entry_point, vtx_constants, primitive): (&wgpu::ShaderModule, &str, Option<&ShaderConstants>, wgpu::PrimitiveState),
        fragment: Option<(&wgpu::ShaderModule, &str, Option<&ShaderConstants>, &[(TexFmt, Option<Blend>); S])>,
    ) -> wgpu::RenderPipeline {

        // cache temporar values
        let targets: ArrayVec<_, S>;

        self.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {

            label: None,
            cache: None,

            layout: layout.map(|(c, b)| self.pipeline_layout(c, b)).as_ref(),

            vertex: wgpu::VertexState {
                module,
                entry_point: if entry_point.is_empty() { None } else { Some(entry_point) },
                buffers,
                compilation_options: wgpu::PipelineCompilationOptions {
                    zero_initialize_workgroup_memory: false,
                    constants: vtx_constants.unwrap_or(wgpu::PipelineCompilationOptions::default().constants),
                },
            },

            primitive,

            fragment: if let Some((module, entry_point, frag_constants, formats)) = fragment {

                targets = formats.iter().map(|(format, blend)| Some(wgpu::ColorTargetState {
                    format: *format,
                    blend: *blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })).collect();

                Some(wgpu::FragmentState {
                    module,
                    entry_point: if entry_point.is_empty() { None } else { Some(entry_point) },
                    targets: &targets,
                    compilation_options: wgpu::PipelineCompilationOptions {
                        zero_initialize_workgroup_memory: false,
                        constants: frag_constants.unwrap_or(wgpu::PipelineCompilationOptions::default().constants),
                    },

                })
            }
            else {None},

            depth_stencil: depth_testing.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),

            multisample: wgpu::MultisampleState {
                count: msaa, mask: !0, alpha_to_coverage_enabled: false,
            },

            multiview: None,

        })
    }
}


// queue methods
pub trait WgxQueue {

    fn queue(&self) -> &wgpu::Queue;

    fn write_texture<'a, T: ReadBytes>(&self,
        copy_texture: impl ToImageCopyTexture<'a>,
        (data, data_layout):(T, impl ToImageDataLayout),
        extent: impl ToExtent3d,
    ) {
        self.queue().write_texture(copy_texture.to(), data.read_bytes(), data_layout.to(), extent.to())
    }

    fn write_buffer<T: ReadBytes>(&self, buffer:&wgpu::Buffer, offset:u64, data:T) {
        self.queue().write_buffer(buffer, offset, data.read_bytes());
    }

    fn staging_view<'a>(&'a self, buffer:&'a wgpu::Buffer, range: impl RangeBounds<u64>) -> Option<wgpu::QueueWriteBufferView<'a>> {

        let offset = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => 0,
        };

        let size = match range.end_bound() {
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => buffer.size(),
        }.checked_sub(offset)?;

        self.queue().write_buffer_with(buffer, offset, wgpu::BufferSize::new(size)?)
    }
}


pub trait WgxDeviceQueue: WgxDevice + WgxQueue {

    fn texture_with_data<T: ReadBytes>(&self, descriptor: &TexDsc, data: T) -> wgpu::Texture {
        self.device().create_texture_with_data(
            self.queue(), &descriptor.into(), TextureDataOrder::default(), data.read_bytes(),
        )
    }

    // with CommandEncoder

    fn with_encoder<T: ImplicitControlFlow>(&self, handler: impl FnOnce(&mut wgpu::CommandEncoder) -> T) -> T
    {
        let mut encoder = self.command_encoder();
        let res = handler(&mut encoder);
        if res.should_continue() {
            self.queue().submit([encoder.finish()]);
        }
        res
    }
}


// impl

impl<T: WgxDevice + WgxQueue> WgxDeviceQueue for T {}

impl WgxDevice for Wgx { fn device(&self) -> &wgpu::Device { &self.device } }
impl WgxQueue for Wgx { fn queue(&self) -> &wgpu::Queue { &self.queue } }

impl WgxDevice for wgpu::Device { fn device(&self) -> &wgpu::Device { self } }
impl WgxQueue for wgpu::Queue { fn queue(&self) -> &wgpu::Queue { self } }

impl WgxDevice for (wgpu::Device, wgpu::Queue) { fn device(&self) -> &wgpu::Device { &self.0 } }
impl WgxQueue for (wgpu::Device, wgpu::Queue) { fn queue(&self) -> &wgpu::Queue { &self.1 } }

impl WgxDevice for (&wgpu::Device, &wgpu::Queue) { fn device(&self) -> &wgpu::Device { self.0 } }
impl WgxQueue for (&wgpu::Device, &wgpu::Queue) { fn queue(&self) -> &wgpu::Queue { self.1 } }