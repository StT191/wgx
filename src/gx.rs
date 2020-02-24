

use glsl_to_spirv::ShaderType;


// some settings constants
const TEXTUREFORMAT:wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;


pub struct Gx {
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
}


impl Gx {

    // initialize
    pub fn new(window:&winit::window::Window) -> Self {

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
            format: TEXTUREFORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Vsync,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self { surface, device, sc_desc, swap_chain, queue }
    }

    // main methods

    pub fn resize(&mut self, width:u32, height:u32) {
        self.sc_desc.width = width;
        self.sc_desc.height = height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn draw_frame(
        &mut self, color:wgpu::Color,
        draws:&[(&wgpu::RenderPipeline, &wgpu::Buffer, std::ops::Range<u32>, &wgpu::BindGroup)]
    ) {
        let frame = self.swap_chain.get_next_texture();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: color,
                }],
                depth_stencil_attachment: None,
            });

            for (render_pipeline, vertices, range, bind_group) in draws {

                rpass.set_pipeline(render_pipeline);
                rpass.set_vertex_buffers(0, &[(vertices, 0)]);
                rpass.set_bind_group(0, bind_group, &[]);

                rpass.draw(range.clone(), 0..1);
            }

        }

        self.queue.submit(&[encoder.finish()]);
    }


    // creation methods

    pub fn vertex_mapped<T:'static+Copy>(&self, size:usize) -> wgpu::CreateBufferMapped<T> {
        self.device.create_buffer_mapped::<T>(size, wgpu::BufferUsage::VERTEX)
    }


    pub fn load_glsl(&self, path:&str, ty:ShaderType) -> wgpu::ShaderModule {
        let code = std::fs::read_to_string(path).unwrap();
        let shader_spirv = glsl_to_spirv::compile(&code, ty).unwrap();
        let shader = wgpu::read_spirv(shader_spirv).unwrap();
        self.device.create_shader_module(&shader)
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
        &self,
        vs_module:&wgpu::ShaderModule, fs_module:&wgpu::ShaderModule,
        vertex_layout:wgpu::VertexBufferDescriptor, bind_group_layout:&wgpu::BindGroupLayout,
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

            primitive_topology: wgpu::PrimitiveTopology::TriangleList,

            color_states: &[wgpu::ColorStateDescriptor {
                format: TEXTUREFORMAT,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],

            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[vertex_layout],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }

}