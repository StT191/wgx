
use glsl_to_spirv::ShaderType;
use futures::executor::block_on;
use std::{io::{Read, Seek}, ops::Range};
// use core::num::NonZeroU8;
use cgmath::Matrix4;

use wgpu::util::DeviceExt;

use wgpu_glyph::{GlyphBrush, ab_glyph::{FontArc, InvalidFont}, GlyphBrushBuilder};


use crate::byte_slice::AsByteSlice;
use crate::*;


// gx

pub struct Gx {
    // instance: wgpu::Instance,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,

    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    pub depth_testing: bool, // changing needs call to update
    depth_texture: Option<wgpu::Texture>,
    depth_texture_view: Option<wgpu::TextureView>,

    pub msaa: u32, // antialiasing // changing needs call to update
    msaa_texture: Option<wgpu::Texture>,
    msaa_texture_view: Option<wgpu::TextureView>,

    current_frame: Option<wgpu::SwapChainFrame>,
}


impl Gx {

    // getters

    pub fn device(&self) -> &wgpu::Device { &self.device }
    pub fn width(&self) -> u32 { self.sc_desc.width }
    pub fn height(&self) -> u32 { self.sc_desc.height }


    // initialize

    pub fn new(window:&winit::window::Window, depth_testing:bool, msaa:u32) -> Self {

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        })).unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None,
        )).unwrap();

        let size = window.inner_size();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: OUTPUT_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let mut gx = Self {
            /*instance,*/ surface, device, sc_desc, swap_chain, queue,
            depth_testing, depth_texture: None, depth_texture_view: None,
            msaa, msaa_texture: None, msaa_texture_view: None,
            current_frame: None,
        };

        if depth_testing || msaa > 1 {
            gx.update(size.width, size.height)
        }

        gx
    }


    // texture

    pub fn texture(&self,
        width:u32, height:u32, sample_count:u32, usage:wgpu::TextureUsage, format:TexOpt,
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            usage, label: None, mip_level_count: 1, sample_count, dimension: wgpu::TextureDimension::D2,
            size: wgpu::Extent3d {width, height, depth: 1},
            format: TexOpt::select(format),
        })
    }

    pub fn depth_texture(&self, width:u32, height:u32, msaa:u32) -> wgpu::Texture {
        self.texture(width, height, msaa, wgpu::TextureUsage::OUTPUT_ATTACHMENT, TexOpt::Depth)
    }

    pub fn msaa_texture(&self, width:u32, height:u32, msaa:u32, format:TexOpt) -> wgpu::Texture {
        self.texture(width, height, msaa, wgpu::TextureUsage::OUTPUT_ATTACHMENT, format)
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


    // update

    pub fn update(&mut self, width:u32, height:u32) {
        self.sc_desc.width = width;
        self.sc_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        if self.depth_testing {
            let depth_texture = self.depth_texture(width, height, self.msaa);
            self.depth_texture_view = Some(depth_texture.create_default_view());
            self.depth_texture = Some(depth_texture);
        }

        if self.msaa > 1 {
            let msaa_texture = self.msaa_texture(width, height, self.msaa, TexOpt::Output);
            self.msaa_texture_view = Some(msaa_texture.create_default_view());
            self.msaa_texture = Some(msaa_texture);
        }
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


    // text render

    pub fn glyph_brush(&self, format:TexOpt, font_data:Vec<u8>) -> Result<GlyphBrush<(), FontArc>, InvalidFont>
    {
        let font = FontArc::try_from_vec(font_data)?;
        Ok(GlyphBrushBuilder::using_font(font).build(&self.device, TexOpt::select(format)))
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


    // render_pipeline

    pub fn render_pipeline(
        &self, format:TexOpt, depth_testing:bool, alpha_blend:bool, msaa:u32,
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

                format: TexOpt::select(format),

                color_blend: if alpha_blend { wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                }} else { wgpu::BlendDescriptor::REPLACE },

                alpha_blend: wgpu::BlendDescriptor::REPLACE,

                /*alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Max,
                },*/

                write_mask: wgpu::ColorWrite::ALL,
            }],

            depth_stencil_state: if depth_testing { Some(wgpu::DepthStencilStateDescriptor {
                format: DEPTH_FORMAT,
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


    // encoding, rendering

    pub fn with_encoder<'a, F, T>(&mut self, handler: F) -> T
        where F: 'a + FnOnce(&mut wgpu::CommandEncoder, &mut Gx) -> T
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let result = handler(&mut encoder, self);
        self.queue.submit(Some(encoder.finish()));
        result
    }


    pub fn frame(&self) -> (&wgpu::TextureView, Option<&wgpu::TextureView>, Option<&wgpu::TextureView>) {
        (
            &(self.current_frame.as_ref().expect("no current frame")).output.view,
            self.depth_texture_view.as_ref(),
            self.msaa_texture_view.as_ref()
        )
    }


    pub fn with_encoder_frame<'a, F>(&mut self, handler: F) -> Result<(), wgpu::SwapChainError>
        where F: 'a + FnOnce(&mut wgpu::CommandEncoder, &mut Gx)
    {
        self.current_frame = Some(self.swap_chain.get_current_frame()?);

        self.with_encoder(|mut encoder, gx| { handler(&mut encoder, gx) });

        self.current_frame = None;

        Ok(())
    }


    pub fn draw(
        &self, encoder:&mut wgpu::CommandEncoder, color:Option<Color>,
        draws:&[(&wgpu::RenderPipeline, &wgpu::BindGroup, wgpu::BufferSlice, Range<u32>)]
    ) {
        encoder.draw(self.frame(), color, draws);
    }


    pub fn draw_glyphs(
        &mut self, encoder:&mut wgpu::CommandEncoder, glyphs:&mut GlyphBrush<(), FontArc>, transform: Option<Matrix4<f32>>
    ) -> Result<(), String>
    {
        let (width, height) = (self.sc_desc.width as f32, self.sc_desc.height as f32);
        let depth = f32::max(width, height);

        let mut trfmat =
            Matrix4::from_nonuniform_scale(2.0/width, -2.0/height, 1.0/depth) *
            Matrix4::<f32>::from_translation((0.0, 0.0, depth/2.0).into())
            ;

        if let Some(trf) = transform { trfmat = trfmat * trf };

        glyphs.draw(&self.device, encoder, self.frame().0, trfmat, None)
    }


}

