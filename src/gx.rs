
use glsl_to_spirv::ShaderType;
use std::{io::{Read, Seek}, mem::size_of, ptr};


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
    draws:&[(&wgpu::RenderPipeline, &wgpu::Buffer, std::ops::Range<u32>, &wgpu::BindGroup)]
) {
    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: if let Some(aaatt) = mssa_attachment { aaatt } else { attachment },
            resolve_target: if mssa_attachment.is_some() { Some(attachment) } else { None },
            load_op: wgpu::LoadOp::Clear,
            store_op: wgpu::StoreOp::Store,
            clear_color: color,
        }],
        depth_stencil_attachment: if let Some(depth_attachment) = depth_attachment {
          Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
            attachment: depth_attachment,
            depth_load_op: wgpu::LoadOp::Clear,
            depth_store_op: wgpu::StoreOp::Store,
            stencil_load_op: wgpu::LoadOp::Clear,
            stencil_store_op: wgpu::StoreOp::Store,
            clear_depth: 1.0,
            clear_stencil: 0,
        })} else { None },
    });

    for (render_pipeline, vertices, range, bind_group) in draws {

        rpass.set_pipeline(render_pipeline);
        rpass.set_bind_group(0, bind_group, &[]);
        rpass.set_vertex_buffers(0, &[(vertices, 0)]);

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
    texture:&wgpu::Texture, (x, y, array_layer, w, h):(u32, u32, u32, u32, u32)
) {
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer, offset, row_pitch: 4 * bf_w, image_height: bf_h,
        },
        wgpu::TextureCopyView {
            texture, mip_level: 0, array_layer, origin: wgpu::Origin3d { x, y, z: 0, }
        },
        wgpu::Extent3d {width: w, height: h, depth: 1},
    );
}

pub fn texture_to_buffer(
    encoder:&mut wgpu::CommandEncoder,
    texture:&wgpu::Texture, (x, y, array_layer, w, h):(u32, u32, u32, u32, u32),
    buffer:&wgpu::Buffer, (bf_w, bf_h, offset):(u32, u32, u64)
) {
    encoder.copy_texture_to_buffer(
        wgpu::TextureCopyView {
            texture, mip_level: 0, array_layer, origin: wgpu::Origin3d { x, y, z: 0, }
        },
        wgpu::BufferCopyView {
            buffer, offset, row_pitch: 4 * bf_w, image_height: bf_h,
        },
        wgpu::Extent3d {width: w, height: h, depth: 1},
    );
}



pub struct Gx {
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

        let surface = wgpu::Surface::create(window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
            },
            wgpu::BackendBit::PRIMARY
        ).unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });

        let size = window.inner_size();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: OUTPUT_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Vsync,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let mut new = Self {
            surface, device, sc_desc, swap_chain, queue,
            depth_testing, depth_texture: None, depth_texture_view: None,
            msaa, msaa_texture: None, msaa_texture_view: None,
        };

        if depth_testing || msaa > 1 {
            new.update(size.width, size.height)
        }

        new
    }

    // main methods

    pub fn update(&mut self, width:u32, height:u32) {
        self.sc_desc.width = width;
        self.sc_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        if self.depth_testing {
            let depth_texture = self.texture(width, height, 1, self.msaa, wgpu::TextureUsage::OUTPUT_ATTACHMENT, TexOpt::Depth);

            self.depth_texture_view = Some(depth_texture.create_default_view());
            self.depth_texture = Some(depth_texture);
        }

        if self.msaa > 1 {
            let msaa_texture = self.texture(width, height, 1, self.msaa, wgpu::TextureUsage::OUTPUT_ATTACHMENT, TexOpt::Output);

            self.msaa_texture_view = Some(msaa_texture.create_default_view());
            self.msaa_texture = Some(msaa_texture);
        }
    }

    // encoding, rendering

    pub fn with_encoder<'a, F>(&mut self, handler: F)
        where F: 'a + FnOnce(&mut wgpu::CommandEncoder, &mut Gx)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        handler(&mut encoder, self);

        self.queue.submit(&[encoder.finish()]);
    }


    pub fn with_encoder_frame<'a, F>(&mut self, handler: F) -> Result<(), ()>
        where F: 'a + FnOnce(
            &mut wgpu::CommandEncoder, &wgpu::SwapChainOutput,
            Option<&wgpu::TextureView>, Option<&wgpu::TextureView>
        )
    {
        let frame = self.swap_chain.get_next_texture()?;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        handler(&mut encoder, &frame, self.depth_texture_view.as_ref(), self.msaa_texture_view.as_ref());

        self.queue.submit(&[encoder.finish()]);

        Ok(())
    }

    // creation methods

    pub fn buffer(&self, usage:wgpu::BufferUsage, size:u64) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {usage, size})
    }

    pub fn buffer_mapped(&self, usage:wgpu::BufferUsage, size:usize) -> wgpu::CreateBufferMapped {
        self.device.create_buffer_mapped(size, usage)
    }

    pub fn buffer_from_data<T:Sized+Copy>(&self, usage:wgpu::BufferUsage, data:&[T]) -> wgpu::Buffer {

        let size = data.len() * size_of::<T>();
        let buffer_mapped = self.device.create_buffer_mapped(size, usage);

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr() as *const u8, buffer_mapped.data.as_mut_ptr(), size);
        }

        buffer_mapped.finish()
    }

    pub fn load_spirv<R:Read+Seek>(&self, shader_spirv:R) -> wgpu::ShaderModule {
        let shader = wgpu::read_spirv(shader_spirv).unwrap();
        self.device.create_shader_module(&shader)
    }

    pub fn load_glsl(&self, code:&str, ty:ShaderType) -> wgpu::ShaderModule {
        self.load_spirv(glsl_to_spirv::compile(&code, ty).unwrap())
    }

    pub fn texture(&self,
        width:u32, height:u32, array_layer_count:u32, sample_count:u32,
        usage:wgpu::TextureUsage, format:TexOpt
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {width, height, depth: 1},
            array_layer_count,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: match format {
                TexOpt::Output => OUTPUT_FORMAT,
                TexOpt::Texture => TEXTURE_FORMAT,
                TexOpt::Depth => DEPTH_FORMAT
            },
            usage
        })
    }

    pub fn sampler(&self) -> wgpu::Sampler {
        self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        })
    }


    // bind group
    pub fn binding(&self, bindings: &[wgpu::BindGroupLayoutBinding]) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings
        })
    }

    pub fn bind(&self, layout:&wgpu::BindGroupLayout, bindings: &[wgpu::Binding]) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout, bindings,
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
            bind_group_layouts: &[bind_group_layout],
        });

        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

            layout: &pipeline_layout,

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
            }),

            primitive_topology: topology,

            color_states: &[wgpu::ColorStateDescriptor {
                format: if use_texture_format { TEXTURE_FORMAT } else { OUTPUT_FORMAT },

                color_blend: if alpha_blend { wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add
                }} else { wgpu::BlendDescriptor::REPLACE },

                alpha_blend: wgpu::BlendDescriptor::REPLACE,

                write_mask: wgpu::ColorWrite::ALL,
            }],

            depth_stencil_state: if depth_testing { Some(wgpu::DepthStencilStateDescriptor {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }) } else { None },

            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[vertex_layout],
            sample_count: msaa,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }

}