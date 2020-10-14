
use glsl_to_spirv::ShaderType;
use futures::executor::block_on;
use std::io::{Read, Seek};

use wgpu::util::DeviceExt;

use crate::byte_slice::AsByteSlice;


// some settings constants
pub const OUTPUT_FORMAT:wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
// pub const TEXTURE_FORMAT:wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const TEXTURE_FORMAT:wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;


// TextureFormat enum
#[derive(Debug)]
pub enum TexOpt { Output, Texture, Depth }


pub fn pass_render(
    encoder:&mut wgpu::CommandEncoder,
    attachment:&wgpu::TextureView,
    depth_attachment:Option<&wgpu::TextureView>,
    mssa_attachment:Option<&wgpu::TextureView>, // antialiasing multisampled texture_attachment
    color:wgpu::Color,
    draws:&[(&wgpu::RenderPipeline, &wgpu::BindGroup, wgpu::BufferSlice, std::ops::Range<u32>)]
) {
    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: if let Some(ms_at) = mssa_attachment { ms_at } else { attachment },
            resolve_target: if mssa_attachment.is_some() { Some(attachment) } else { None },
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color),
                store: true
            }
        }],
        depth_stencil_attachment: if let Some(depth_attachment) = depth_attachment {
          Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
            attachment: depth_attachment,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: true
            }),
            stencil_ops: None,
            /*stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                store: true
            }),*/
        })} else { None },
    });

    /*let mut l_render_pipeline = None;
    let mut l_bind_group = None;
    let mut l_vertices = None;*/

    for (render_pipeline, bind_group, vertices, range) in draws {

        // if l_render_pipeline != Some(render_pipeline) {
        rpass.set_pipeline(render_pipeline);
            /*l_render_pipeline = Some(render_pipeline);
        }*/

        // if l_bind_group != Some(bind_group) {
        rpass.set_bind_group(0, bind_group, &[]);
            /*l_bind_group = Some(bind_group);
        }*/

        // if l_vertices != Some(vertices) {
        rpass.set_vertex_buffer(0, *vertices);
            /*l_vertices = Some(vertices);
        }*/

        rpass.draw(range.clone(), 0..1);
    }
}


pub fn buffer_to_buffer(
    encoder:&mut wgpu::CommandEncoder,
    src_buffer:&wgpu::Buffer, src_offset:wgpu::BufferAddress,
    dst_buffer:&wgpu::Buffer, dst_offset:wgpu::BufferAddress,
    size:wgpu::BufferAddress,
) {
    encoder.copy_buffer_to_buffer(src_buffer, src_offset, dst_buffer, dst_offset, size);
}


pub fn buffer_to_texture(
    encoder:&mut wgpu::CommandEncoder,
    buffer:&wgpu::Buffer, (bf_w, bf_h, offset):(u32, u32, u64),
    texture:&wgpu::Texture, (x, y, w, h):(u32, u32, u32, u32)
) {
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyViewBase {
            buffer,
            layout: wgpu::TextureDataLayout {
                offset, bytes_per_row: 4 * bf_w, rows_per_image: bf_h,
            }
        },
        wgpu::TextureCopyViewBase {
            texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z: 0, }
        },
        wgpu::Extent3d {width: w, height: h, depth: 1},
    );
}



pub fn texture_to_buffer(
    encoder:&mut wgpu::CommandEncoder,
    texture:&wgpu::Texture, (x, y, w, h):(u32, u32, u32, u32),
    buffer:&wgpu::Buffer, (bf_w, bf_h, offset):(u32, u32, u64)
) {
    encoder.copy_texture_to_buffer(
        wgpu::TextureCopyViewBase {
            texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z: 0, }
        },
        wgpu::BufferCopyViewBase {
            buffer,
            layout:  wgpu::TextureDataLayout {
                offset, bytes_per_row: 4 * bf_w, rows_per_image: bf_h,
            }
        },
        wgpu::Extent3d {width: w, height: h, depth: 1},
    );
}



pub struct Gx {
    // instance: wgpu::Instance,
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    queue: wgpu::Queue,

    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    pub depth_testing: bool, // changing needs call to update
    depth_texture: Option<wgpu::Texture>,
    depth_texture_view: Option<wgpu::TextureView>,

    pub msaa: u32, // antialiasing // changing needs call to update
    msaa_texture: Option<wgpu::Texture>,
    msaa_texture_view: Option<wgpu::TextureView>,
}


impl Gx {

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
        };

        if depth_testing || msaa > 1 {
            gx.update(size.width, size.height)
        }

        gx
    }

    // main methods

    pub fn update(&mut self, width:u32, height:u32) {
        self.sc_desc.width = width;
        self.sc_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        if self.depth_testing {
            let depth_texture = self.texture(
                width, height, self.msaa, wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                TexOpt::Depth, Some("DEPTH TEXTURE")
            );

            self.depth_texture_view = Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()/*&wgpu::TextureViewDescriptor {
                label: Some("DEPTH TEXTURE"),
                format: Some(DEPTH_FORMAT),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            }*/));

            self.depth_texture = Some(depth_texture);
        }

        if self.msaa > 1 {
            let msaa_texture = self.texture(
                width, height, self.msaa, wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                TexOpt::Output, Some("MSSA TEXTURE")
            );

            self.msaa_texture_view = Some(msaa_texture.create_view(&wgpu::TextureViewDescriptor::default()/*&wgpu::TextureViewDescriptor {
                label: Some("MSSA TEXTURE"),
                format: Some(TEXTURE_FORMAT),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::StencilOnly, // All
                base_mip_level: 0,
                level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            }*/));

            self.msaa_texture = Some(msaa_texture);
        }
    }

    // encoding, rendering

    pub fn with_encoder<'a, F>(&mut self, handler: F)
        where F: 'a + FnOnce(&mut wgpu::CommandEncoder, &mut Gx)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        handler(&mut encoder, self);

        self.queue.submit(Some(encoder.finish()));
    }


    pub fn with_encoder_frame<'a, F>(&mut self, handler: F) -> Result<(), wgpu::SwapChainError>
        where F: 'a + FnOnce(
            &mut wgpu::CommandEncoder, &wgpu::SwapChainFrame,
            Option<&wgpu::TextureView>, Option<&wgpu::TextureView>
        )
    {
        let frame = self.swap_chain.get_current_frame()?;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        handler(&mut encoder, &frame, self.depth_texture_view.as_ref(), self.msaa_texture_view.as_ref());

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }


    pub fn pass_render(&mut self, color:wgpu::Color,
        draws:&[(&wgpu::RenderPipeline, &wgpu::BindGroup, wgpu::BufferSlice, std::ops::Range<u32>)]
    ) -> Result<(), wgpu::SwapChainError>
    {
        self.with_encoder_frame(|encoder, frame, deph_view, msaa| {
            pass_render(encoder, &frame.output.view, deph_view, msaa, color, draws);
        })
    }

    // creation methods

    pub fn buffer(&self, usage:wgpu::BufferUsage, size:u64, mapped_at_creation:bool) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {usage, size, mapped_at_creation, label: None})
    }

    /*pub fn buffer_mapped(&self, usage:wgpu::BufferUsage, size:u64) -> wgpu::CreateBufferMapped {
        self.device.create_buffer_mapped(&wgpu::BufferDescriptor {usage, size, label: None, mapped_at_creation: true})
    }*/

    /*pub fn buffer_from_data<T:Sized+Copy>(&self, usage:wgpu::BufferUsage, data:&[T]) -> wgpu::Buffer {

        let size = data.len() * size_of::<T>();
        let buffer_mapped = self.buffer_mapped(usage, size as u64);

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr() as *const u8, buffer_mapped.data.as_mut_ptr(), size);
        }


    }*/

    pub fn buffer_from_data<T:Sized>(&self, usage:wgpu::BufferUsage, data:&[T]) -> wgpu::Buffer {
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            usage, contents: data.as_byte_slice(), label: None
        })
    }

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


    pub fn texture(&self,
        width:u32, height:u32, sample_count:u32, usage:wgpu::TextureUsage, format:TexOpt, label:Option<&str>
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {width, height, depth: 1},
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: match format {
                TexOpt::Output => OUTPUT_FORMAT,
                TexOpt::Texture => TEXTURE_FORMAT,
                TexOpt::Depth => DEPTH_FORMAT
            },
            usage,
            label,
        })
    }


    pub fn write_texture<T:Sized>(&self, texture:&wgpu::Texture, (x, y, w, h):(u32, u32, u32, u32), data:&[T]) {
        self.queue.write_texture(
            wgpu::TextureCopyViewBase {
                texture, mip_level: 0, origin: wgpu::Origin3d { x, y, z: 0 }
            },
            data.as_byte_slice(),
            wgpu::TextureDataLayout {
                offset: 0, bytes_per_row: 4 * w, rows_per_image: h,
            },
            wgpu::Extent3d { width: w, height: h, depth: 1 },
        )
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
            compare: Some(wgpu::CompareFunction::Always),
            anisotropy_clamp: None,
        })
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
        &self, use_texture_format:bool, depth_testing:bool, alpha_blend:bool, msaa:u32,
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
                format: if use_texture_format { TEXTURE_FORMAT } else { OUTPUT_FORMAT },

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
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default()
                /*stencil: wgpu::StencilStateDescriptor {
                    front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                }*/
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