

use glsl_to_spirv::ShaderType;

use zerocopy::{FromBytes, AsBytes};

// some settings constants
pub const OUTPUT_FORMAT:wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
// pub const TEXTURE_FORMAT:wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const TEXTURE_FORMAT:wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;


#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, FromBytes, AsBytes)]
#[repr(C)]
pub struct Color (pub u8, pub u8, pub u8, pub u8);

impl<T: Into<u8>> From<(T, T, T, T)> for Color{
    fn from(data:(T, T, T, T)) -> Self { Self(data.0.into(), data.1.into(), data.2.into(), data.3.into()) }
}


pub fn pass_render(
    encoder:&mut wgpu::CommandEncoder,
    attachment:&wgpu::TextureView,
    depth_attachment:Option<&wgpu::TextureView>,
    color:wgpu::Color,
    draws:&[(&wgpu::RenderPipeline, &wgpu::Buffer, std::ops::Range<u32>, &wgpu::BindGroup)]
) {
    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment,
            resolve_target: None,
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


pub fn buffer_to_texture(
    encoder:&mut wgpu::CommandEncoder,
    buffer:&wgpu::Buffer, (bf_w, bf_h, offset):(u32, u32, u64),
    texture:&wgpu::Texture, (x, y, array_layer, w, h):(f32, f32, u32, u32, u32)
) {
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer, offset, row_pitch: 4 * bf_w, image_height: bf_h,
        },
        wgpu::TextureCopyView {
            texture, mip_level: 0, array_layer, origin: wgpu::Origin3d { x, y, z: 0.0, }
        },
        wgpu::Extent3d {width: w, height: h, depth: 1},
    );
}

pub fn texture_to_buffer(
    encoder:&mut wgpu::CommandEncoder,
    texture:&wgpu::Texture, (x, y, array_layer, w, h):(f32, f32, u32, u32, u32),
    buffer:&wgpu::Buffer, (bf_w, bf_h, offset):(u32, u32, u64)
) {
    encoder.copy_texture_to_buffer(
        wgpu::TextureCopyView {
            texture, mip_level: 0, array_layer, origin: wgpu::Origin3d { x, y, z: 0.0, }
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
    depth_texture: Option<wgpu::Texture>,
    depth_texture_view: Option<wgpu::TextureView>,
}


impl Gx {

    // initialize
    pub fn new(window:&winit::window::Window, deph_testing:bool) -> Self {

        let surface = wgpu::Surface::create(window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                backends: wgpu::BackendBit::PRIMARY,
            },
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
            depth_texture: None, depth_texture_view: None
        };

        if deph_testing {
            new.resize(size.width, size.height, true)
        }

        new
    }

    // main methods

    pub fn resize(&mut self, width:u32, height:u32, deph_testing:bool) {
        self.sc_desc.width = width;
        self.sc_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        if deph_testing {
            let depth_texture = self.texture(width, height, 1, wgpu::TextureUsage::OUTPUT_ATTACHMENT, true);

            self.depth_texture_view = Some(depth_texture.create_default_view());
            self.depth_texture = Some(depth_texture);
        }
    }

    // encoding, rendering

    pub fn with_encoder<'a, F>(&mut self, mut handler: F)
        where F: 'a + FnMut(&mut wgpu::CommandEncoder, &mut Gx)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        handler(&mut encoder, self);

        self.queue.submit(&[encoder.finish()]);
    }


    pub fn with_encoder_frame<'a, F>(&mut self, mut handler: F)
        where F: 'a + FnMut(&mut wgpu::CommandEncoder, &wgpu::SwapChainOutput, Option<&wgpu::TextureView>)
    {
        let frame = self.swap_chain.get_next_texture();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        handler(&mut encoder, &frame, self.depth_texture_view.as_ref());

        self.queue.submit(&[encoder.finish()]);
    }

    // creation methods

    pub fn buffer(&self, usage:wgpu::BufferUsage, size:u64) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {usage, size})
    }

    pub fn buffer_mapped<T:'static+Copy>(&self, usage:wgpu::BufferUsage, size:usize) -> wgpu::CreateBufferMapped<T> {
        self.device.create_buffer_mapped::<T>(size, usage)
    }

    pub fn buffer_from_data<T:'static+Copy>(&self, usage:wgpu::BufferUsage, data:&[T]) -> wgpu::Buffer {
        self.device.create_buffer_mapped::<T>(data.len(), usage).fill_from_slice(data)
    }

    pub fn load_glsl(&self, path:&str, ty:ShaderType) -> wgpu::ShaderModule {
        let code = std::fs::read_to_string(path).unwrap();
        let shader_spirv = glsl_to_spirv::compile(&code, ty).unwrap();
        let shader = wgpu::read_spirv(shader_spirv).unwrap();
        self.device.create_shader_module(&shader)
    }

    pub fn texture(&self,
        width:u32, height:u32, array_layer_count:u32,
        usage:wgpu::TextureUsage, is_deph_texture:bool
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {width, height, depth: 1},
            array_layer_count,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if is_deph_texture { DEPTH_FORMAT } else { TEXTURE_FORMAT },
            usage,
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
        &self, use_texture_format:bool, depth_testing:bool, alpha_blend:bool,
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
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }

}