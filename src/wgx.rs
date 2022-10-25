
#[cfg(feature = "iced")]
use iced_wgpu::{Renderer, Backend, Settings, Antialiasing};
use std::num::NonZeroU32;
use arrayvec::ArrayVec;
use wgpu::util::DeviceExt;
use raw_window_handle::HasRawWindowHandle;
use crate::{*, byte_slice::AsByteSlice, error::*};


// wgx

pub struct Wgx {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}


impl Wgx {

    pub async fn new<W: HasRawWindowHandle>(window:Option<&W>, features:wgpu::Features, limits:wgpu::Limits,)
        -> Res<(Self, Option<wgpu::Surface>)>
    {

        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let surface = if let Some(window) = window {
           unsafe { Some(instance.create_surface(window)) }
        }
        else { None };


        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: surface.as_ref(),
        }).await.ok_or("couldn't get adapter")?;


        #[cfg(target_family = "wasm")] let limits = limits.using_resolution(adapter.limits());

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {label: None, features, limits}, None,
        ).await.convert()?;

        Ok((Self { instance, adapter, device, queue }, surface))
    }


    // texture
    pub fn texture(&self, descriptor:&TexDsc) -> wgpu::Texture {
        self.device.create_texture(descriptor)
    }

    pub fn texture_2d(&self, size:(u32, u32), sample_count:u32, format:wgpu::TextureFormat, usage:TexUse)
        -> wgpu::Texture
    {
        self.texture(&TexDsc::new_2d(size, sample_count, format, usage))
    }

    pub fn texture_2d_bound(&self, size:(u32, u32), sample_count:u32, format:wgpu::TextureFormat, usage:TexUse)
        -> wgpu::Texture
    {
        self.texture_2d(size, sample_count, format, TexUse::TEXTURE_BINDING | usage)
    }

    pub fn texture_2d_attached(&self, size:(u32, u32), sample_count:u32, format:wgpu::TextureFormat, usage:TexUse)
        -> wgpu::Texture
    {
        self.texture_2d(size, sample_count, format, TexUse::RENDER_ATTACHMENT | usage)
    }

    pub fn texture_with_data<T: AsByteSlice<U>, U>(&self, descriptor: &TexDsc, data: T) -> wgpu::Texture {
        self.device.create_texture_with_data(&self.queue, descriptor, data.as_byte_slice())
    }

    pub fn texture_2d_with_data<T: AsByteSlice<U>, U>(
        &self, size:(u32, u32), sample_count:u32, format:wgpu::TextureFormat, usage:TexUse, data: T
    ) -> wgpu::Texture {
        self.texture_with_data(&TexDsc::new_2d(size, sample_count, format, usage), data)
    }

    pub fn write_texture<T: AsByteSlice<U>, U>(&self, texture:&wgpu::Texture, (x, y, w, h):(u32, u32, u32, u32), data:T) {
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All
            },
            data.as_byte_slice(),
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: NonZeroU32::new(4 * w), rows_per_image: NonZeroU32::new(h) },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        )
    }

    pub fn sampler(&self, descriptor: &wgpu::SamplerDescriptor) -> wgpu::Sampler {
        self.device.create_sampler(descriptor)
    }

    pub fn default_sampler(&self) -> wgpu::Sampler {
        self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            border_color: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None, // Some(wgpu::CompareFunction::LessEqual),
            anisotropy_clamp: None, // NonZeroU8::new(16),
        })
    }


    // buffer

    pub fn buffer(&self, usage:BufUse, size:u64, mapped_at_creation:bool) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {usage, size, mapped_at_creation, label: None})
    }

    pub fn buffer_from_data<T: AsByteSlice<U>, U>(&self, usage:BufUse, data:T) -> wgpu::Buffer {
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            usage, contents: data.as_byte_slice(), label: None
        })
    }

    pub fn write_buffer<T: AsByteSlice<U>, U>(&self, buffer:&wgpu::Buffer, offset:u64, data:T) {
        self.queue.write_buffer(buffer, offset, data.as_byte_slice());
    }


    // shader

    pub fn load_wgsl(&self, code:&str) -> wgpu::ShaderModule {
        let source = wgpu::ShaderSource::Wgsl(code.into());
        self.device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source })
    }


    // bind group

    pub fn layout(&self, entries:&[wgpu::BindGroupLayoutEntry]) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries, label: None
        })
    }

    pub fn bind(&self, layout:&wgpu::BindGroupLayout, entries:&[wgpu::BindGroupEntry]) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout, entries, label: None
        })
    }


    // iced_backend
    #[cfg(feature = "iced")]
    pub fn iced_renderer(&self, mut settings:Settings, format:wgpu::TextureFormat, msaa: Option<u32>) -> Renderer {
        if let Some(msaa) = msaa {
            settings.antialiasing = match msaa {
                2 => Some(Antialiasing::MSAAx2),
                4 => Some(Antialiasing::MSAAx4),
                8 => Some(Antialiasing::MSAAx8),
                16 => Some(Antialiasing::MSAAx16),
                _ => None,
            }
        }
        Renderer::new(Backend::new(&self.device, settings, format))
    }


    // command encoder

    pub fn with_encoder<C: ImplicitControlflow>(&self, handler: impl FnOnce(&mut wgpu::CommandEncoder) -> C)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let controlflow = handler(&mut encoder);
        if controlflow.should_continue() {
            self.queue.submit([encoder.finish()]);
        }
    }


    // render bundle

    pub fn render_bundle_encoder(&self, formats: &[Option<wgpu::TextureFormat>], depth_testing:bool, msaa:u32)
        -> wgpu::RenderBundleEncoder
    {
        self.device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: formats,
            depth_stencil: if depth_testing { Some(wgpu::RenderBundleDepthStencil {
                format: DEPTH, depth_read_only: false, stencil_read_only: false,
            })} else { None },
            sample_count: msaa,
            multiview: None,
        })
    }


    // compute pipeline

    pub fn compute_pipeline(
        &self, (module, entry_point):(&wgpu::ShaderModule, &str),
        layout:Option<(&[wgpu::PushConstantRange], &[&wgpu::BindGroupLayout])>
    ) -> wgpu::ComputePipeline {

        let layout = if let Some(layout) = layout {
            Some(self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None, push_constant_ranges: layout.0, bind_group_layouts: layout.1
            }))
        }
        else { None };

        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: layout.as_ref(),
            module, entry_point,
        })
    }


    // render pipeline

    pub fn render_pipeline<const S: usize>(
        &self,
        depth_testing: bool, msaa: u32,
        layout: Option<(&[wgpu::PushConstantRange], &[&wgpu::BindGroupLayout])>,
        buffers: &[wgpu::VertexBufferLayout],
        (module, entry_point, topology): (&wgpu::ShaderModule, &str, wgpu::PrimitiveTopology),
        fragment: Option<(&wgpu::ShaderModule, &str, &[(wgpu::TextureFormat, Option<BlendState>); S])>,
    ) -> wgpu::RenderPipeline {

        // cache temporar values
        let pipeline_layout;
        let targets: ArrayVec<_, S>;


        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

            label: None,

            layout: if let Some((push_constant_ranges, bind_group_layouts)) = layout {

                pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None, push_constant_ranges, bind_group_layouts,
                });

                Some(&pipeline_layout)
            }
            else { None },

            vertex: wgpu::VertexState { module, entry_point, buffers },

            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },

            fragment: if let Some((module, entry_point, formats)) = fragment {

                targets = formats.iter().map(|(format, blend)| Some(wgpu::ColorTargetState {
                    format: *format,
                    blend: *blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })).collect();

                Some(wgpu::FragmentState { module, entry_point, targets: &targets })
            }
            else {None},

            depth_stencil: if depth_testing { Some(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }) }
            else { None },

            multisample: wgpu::MultisampleState {
                count: msaa, mask: !0, alpha_to_coverage_enabled: false,
            },

            multiview: None,

        })
    }
}

