
use glsl_to_spirv::ShaderType;
use futures::executor::block_on;
use std::io::{Read, Seek};
// use core::num::NonZeroU8;
use wgpu::util::DeviceExt;
use raw_window_handle::HasRawWindowHandle;
use crate::byte_slice::AsByteSlice;
use crate::*;



// Default Texture Formats

pub const OUTPUT:wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const TEXTURE:wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

// wgx

pub struct Wgx {
    surface: Option<wgpu::Surface>,
    pub(super) device: wgpu::Device,
    queue: wgpu::Queue,
}


impl Wgx {

    pub fn new<W: HasRawWindowHandle>(window:Option<&W>) -> Self {

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let surface = if let Some(window) = window {
           unsafe { Some(instance.create_surface(window)) }
        }
        else { None };


        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: surface.as_ref(),
        })).unwrap();


        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None,
        )).unwrap();


        Self { surface, device, queue }
    }


    pub fn surface_target(&mut self, size:(u32, u32), depth_testing:bool, msaa:u32)
        -> Result<SurfaceTarget, String>
    {
        let surface = self.surface.take().ok_or("no surface".to_string())?;

        SurfaceTarget::new(self, surface, size, depth_testing, msaa)
    }


    // texture

    pub fn texture(&self,
        (width, height):(u32, u32), sample_count:u32, usage:wgpu::TextureUsage, format:wgpu::TextureFormat,
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            usage, label: None, mip_level_count: 1, sample_count, dimension: wgpu::TextureDimension::D2,
            size: wgpu::Extent3d {width, height, depth: 1}, format,
        })
    }

    pub fn depth_texture(&self, (width, height):(u32, u32), msaa:u32) -> wgpu::Texture {
        self.texture((width, height), msaa, wgpu::TextureUsage::OUTPUT_ATTACHMENT, DEPTH)
    }

    pub fn msaa_texture(&self, (width, height):(u32, u32), msaa:u32, format:wgpu::TextureFormat) -> wgpu::Texture {
        self.texture((width, height), msaa, wgpu::TextureUsage::OUTPUT_ATTACHMENT, format)
    }

    pub fn sampler(&self) -> wgpu::Sampler {
        self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
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
            wgpu::TextureCopyViewBase { texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z: 0 } },
            data.as_byte_slice(),
            wgpu::TextureDataLayout { offset: 0, bytes_per_row: 4 * w, rows_per_image: h },
            wgpu::Extent3d { width: w, height: h, depth: 1 },
        )
    }


    // buffer

    pub fn buffer(&self, usage:wgpu::BufferUsage, size:u64, mapped_at_creation:bool) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {usage, size, mapped_at_creation, label: None})
    }

    pub fn buffer_from_data<T:Sized>(&self, usage:wgpu::BufferUsage, data:&[T]) -> wgpu::Buffer {
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            usage, contents: data.as_byte_slice(), label: None
        })
    }


    // shader

    pub fn load_spirv<R:Read+Seek>(&self, mut shader_spirv:R) -> wgpu::ShaderModule {
        let mut data = Vec::new();
        let _ = shader_spirv.read_to_end(&mut data);
        let shader = wgpu::util::make_spirv(&data);
        self.device.create_shader_module(shader)
    }

    pub fn load_glsl(&self, code:&str, ty:ShaderType) -> wgpu::ShaderModule {
        self.load_spirv(glsl_to_spirv::compile(&code, ty).unwrap())
    }

    pub fn load_wgsl(&self, code:&str) -> wgpu::ShaderModule {
        self.device.create_shader_module(wgpu::ShaderModuleSource::Wgsl(std::borrow::Cow::Borrowed(code)))
    }


    // bind group

    pub fn binding(&self, entries: &[wgpu::BindGroupLayoutEntry]) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries, label: None
        })
    }

    pub fn bind(&self, layout:&wgpu::BindGroupLayout, entries: &[wgpu::BindGroupEntry]) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout, entries, label: None
        })
    }


    // command encoder

    pub fn with_encoder<'a, F>(&self, handler: F) where F: 'a + FnOnce(&mut wgpu::CommandEncoder)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        handler(&mut encoder);
        self.queue.submit(Some(encoder.finish()));
    }


    // render_pipeline

    pub fn render_pipeline(
        &self, format:wgpu::TextureFormat, depth_testing:bool, msaa:u32, alpha_blend:bool,
        vs_module:&wgpu::ShaderModule, fs_module:&wgpu::ShaderModule,
        vertex_layout:wgpu::VertexBufferDescriptor, topology:wgpu::PrimitiveTopology,
        bind_group_layout:&wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None, push_constant_ranges: &[],
            bind_group_layouts: &[bind_group_layout],
        });

        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

            label: None,

            layout: Some(&pipeline_layout),

            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: vs_module,
                entry_point: "main",
            },

            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: fs_module,
                entry_point: "main",
            }),

            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),

            primitive_topology: topology,

            color_states: &[wgpu::ColorStateDescriptor {

                format,

                color_blend: if alpha_blend { wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                }} else { wgpu::BlendDescriptor::REPLACE },

                alpha_blend: if alpha_blend { wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Max,
                }} else { wgpu::BlendDescriptor::REPLACE },

                write_mask: wgpu::ColorWrite::ALL,
            }],

            depth_stencil_state: if depth_testing { Some(wgpu::DepthStencilStateDescriptor {
                format: DEPTH,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilStateDescriptor::default()
            }) } else { None },

            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[vertex_layout]
            },

            sample_count: msaa,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }
}

