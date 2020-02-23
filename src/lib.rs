#![allow(unused)]

// pub mod flex_refs;

use winit::{
    event,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub use glsl_to_spirv::ShaderType;


// some settings constants
const TEXTUREFORMAT:wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;



// init
pub fn init_all() -> (EventLoop<()>, Window, wgpu::Surface, wgpu::Adapter, wgpu::Device, wgpu::Queue) {

    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();

    let surface = wgpu::Surface::create(&window);

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

    (event_loop, window, surface, adapter, device, queue)
}

pub fn create_swap_chain_descriptor(window:&Window) -> wgpu::SwapChainDescriptor {
    let size = window.inner_size();
    wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: TEXTUREFORMAT,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Vsync,
    }
}


pub fn load_shader_glsl(device:&wgpu::Device, path:&str, ty:ShaderType) -> wgpu::ShaderModule {
    let code = std::fs::read_to_string(path).unwrap();
    let shader_spirv = glsl_to_spirv::compile(&code, ty).unwrap();
    let shader = wgpu::read_spirv(shader_spirv).unwrap();
    device.create_shader_module(&shader)
}


pub fn create_render_pipeline(
    device:&wgpu::Device,
    layouts:&[&wgpu::BindGroupLayout],
    vs_module:&wgpu::ShaderModule, fs_module:&wgpu::ShaderModule,
    vertex_buffers:&[wgpu::VertexBufferDescriptor]
) -> wgpu::RenderPipeline {

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: layouts,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

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
        vertex_buffers,
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}


pub fn draw_frame_command(
    device:&wgpu::Device,
    swap_chain:&mut wgpu::SwapChain,
    render_pipeline:&wgpu::RenderPipeline,
    color:wgpu::Color,
    vertices:&[(&wgpu::Buffer, u64)],
    num: u32,
    bind_group:&wgpu::BindGroup,
) -> wgpu::CommandBuffer {

    let frame = swap_chain.get_next_texture();

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

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

        rpass.set_pipeline(render_pipeline);
        rpass.set_vertex_buffers(0, vertices);
        rpass.set_bind_group(0, bind_group, &[]);

        rpass.draw(0..num, 0..1);
    }

    encoder.finish()
}