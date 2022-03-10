
#[cfg(feature = "spirv")]
use glsl_to_spirv::ShaderType;

#[cfg(feature = "spirv")]
use std::io::{Read, Seek};

#[cfg(feature = "iced")]
use iced_wgpu::{Renderer, Backend, Settings};

use futures::executor::block_on;
use std::num::NonZeroU32;

use wgpu::util::DeviceExt;
use raw_window_handle::HasRawWindowHandle;
use crate::byte_slice::AsByteSlice;
use crate::{*, error::*};



// Default Texture Formats

pub const OUTPUT:wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const TEXTURE:wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

// wgx

pub struct Wgx {
    pub instance: wgpu::Instance,
    pub(super) adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface>,
}


impl Wgx {

    pub fn new<W: HasRawWindowHandle>(
        window:Option<&W>, features:wgpu::Features, limits:wgpu::Limits,
    ) -> Res<Self> {

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        let surface = if let Some(window) = window {
           unsafe { Some(instance.create_surface(window)) }
        }
        else { None };


        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: surface.as_ref(),
        })).ok_or("couldn't get device")?;


        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {label: None, features, limits}, None,
        )).map_err(error)?;

        Ok(Self { instance, adapter, device, queue, surface })
    }


    pub fn surface_target(&mut self, size:(u32, u32), depth_testing:bool, msaa:u32) -> Res<SurfaceTarget>
    {
        let surface = self.surface.take().ok_or("no surface")?;

        SurfaceTarget::new(self, surface, size, depth_testing, msaa)
    }


    // texture

    pub fn texture(&self,
        (width, height):(u32, u32), sample_count:u32, usage:wgpu::TextureUsages, format:wgpu::TextureFormat,
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            usage, label: None, mip_level_count: 1, sample_count, dimension: wgpu::TextureDimension::D2,
            size: wgpu::Extent3d {width, height, depth_or_array_layers: 1}, format,
        })
    }

    pub fn depth_texture(&self, (width, height):(u32, u32), msaa:u32) -> wgpu::Texture {
        self.texture((width, height), msaa, wgpu::TextureUsages::RENDER_ATTACHMENT, DEPTH)
    }

    pub fn msaa_texture(&self, (width, height):(u32, u32), msaa:u32, format:wgpu::TextureFormat) -> wgpu::Texture {
        self.texture((width, height), msaa, wgpu::TextureUsages::RENDER_ATTACHMENT, format)
    }

    pub fn sampler(&self) -> wgpu::Sampler {
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


    // buffer

    pub fn buffer(&self, usage:wgpu::BufferUsages, size:u64, mapped_at_creation:bool) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {usage, size, mapped_at_creation, label: None})
    }

    pub fn buffer_from_data<T: AsByteSlice<U>, U>(&self, usage:wgpu::BufferUsages, data:T) -> wgpu::Buffer {
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            usage, contents: data.as_byte_slice(), label: None
        })
    }

    pub fn write_buffer<T: AsByteSlice<U>, U>(&self, buffer:&wgpu::Buffer, offset:u64, data:T) {
        self.queue.write_buffer(buffer, offset, data.as_byte_slice());
    }


    // shader

    #[cfg(feature = "spirv")]
    pub fn load_spirv<R:Read+Seek>(&self, mut shader_spirv:R) -> wgpu::ShaderModule {
        let mut data = Vec::new();
        let _ = shader_spirv.read_to_end(&mut data);
        let source = wgpu::util::make_spirv(&data);
        self.device.create_shader_module(&wgpu::ShaderModuleDescriptor { label: None, source })
        /*let source = wgpu::util::make_spirv_raw(&data);
        unsafe { self.device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV { label: None, source }) }*/
    }

    #[cfg(feature = "spirv")]
    pub fn load_glsl(&self, code:&str, ty:ShaderType) -> Res<wgpu::ShaderModule> {
        self.load_spirv(glsl_to_spirv::compile(&code, ty)?)
    }

    pub fn load_wgsl(&self, code:&str) -> wgpu::ShaderModule {
        let source = wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(code));
        self.device.create_shader_module(&wgpu::ShaderModuleDescriptor { label: None, source })
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
    pub fn iced_renderer(&self, settings:Settings, format:wgpu::TextureFormat) -> Renderer {
        Renderer::new(Backend::new(&self.device, settings, format))
    }


    // command encoder

    pub fn with_encoder(&self, handler: impl FnOnce(&mut wgpu::CommandEncoder))
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        handler(&mut encoder);
        self.queue.submit([encoder.finish()]);
    }


    // render bundle

    pub fn render_bundle_encoder(&self, formats: &[wgpu::TextureFormat], depth_testing:bool, msaa:u32)
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

    pub fn render_pipeline(
        &self, format:wgpu::TextureFormat, depth_testing:bool, msaa:u32, alpha_blend:bool,
        (vs_module, vs_entry_point):(&wgpu::ShaderModule, &str), (fs_module, fs_entry_point):(&wgpu::ShaderModule, &str),
        vertex_layouts:&[wgpu::VertexBufferLayout], topology:wgpu::PrimitiveTopology,
        layout:Option<(&[wgpu::PushConstantRange], &[&wgpu::BindGroupLayout])>
    ) -> wgpu::RenderPipeline {

        let layout = if let Some(layout) = layout {
            Some(self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None, push_constant_ranges: layout.0, bind_group_layouts: layout.1
            }))
        }
        else { None };

        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

            label: None,

            layout: layout.as_ref(),

            vertex: wgpu::VertexState {
                module: vs_module,
                entry_point: vs_entry_point,
                buffers: vertex_layouts,
            },

            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },

            fragment: Some(wgpu::FragmentState {
                module: fs_module,
                entry_point: fs_entry_point,

                targets: &[wgpu::ColorTargetState {

                    format,

                    blend: if alpha_blend { Some(wgpu::BlendState {

                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },

                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Max,
                        }

                    })} else { None },

                    write_mask: wgpu::ColorWrites::ALL,
                }]
            }),

            depth_stencil: if depth_testing { Some(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }) } else { None },

            multisample: wgpu::MultisampleState {
                count: msaa,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },

            multiview: None,

        })
    }
}

