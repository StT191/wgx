#![allow(unused)]

use std::time::{Duration, Instant};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use wgfx::*;

fn main() {

    let (
        event_loop, window, surface, adapter, device, mut queue
    ) = init_all();

    window.set_title("WgFx");


    // shaders
    let vs_module = load_shader_glsl(&device, "shaders/main.vert", ShaderType::Vertex);
    let fs_module = load_shader_glsl(&device, "shaders/main.frag", ShaderType::Fragment);


    // bind group
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[]
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout, bindings: &[],
    });


    // vertices
    let vertex_desc = wgpu::VertexBufferDescriptor {
        stride: (4+4) as wgpu::BufferAddress,
        step_mode: wgpu::InputStepMode::Vertex,
        attributes: &[wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float2,
            offset: 0,
            shader_location: 0
        }],
    };

    let data = [
        (-0.25, -0.5), (-0.5, -0.5), (-0.5, 0.5),
        (0.25, -0.5), (0.5, -0.5), (0.5, 0.5),
    ];

    let vertices1 = device.create_buffer_mapped::<(f32, f32)>(
        data.len(),
        wgpu::BufferUsage::VERTEX
    ).fill_from_slice(&data[..]);


    let render_pipeline = create_render_pipeline(
        &device, &[&bind_group_layout], &vs_module, &fs_module, &[vertex_desc]
    );

    // swap chain
    let mut sc_desc = create_swap_chain_descriptor(&window);
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);


    // frames
    let frame_time = Duration::from_nanos(1_000_000_000 / 60);
    let mut time = Instant::now();

    // event loop
    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                *control_flow = ControlFlow::Exit;
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
            },


            Event::RedrawRequested(_) => {

                let frame_draw = draw_frame_command(
                    &device, &mut swap_chain, &render_pipeline,
                    wgpu::Color::GREEN,
                    &[(&vertices1, 0)],
                    4,
                    &bind_group,
                );

                queue.submit(&[frame_draw]);

            },

            Event::MainEventsCleared => {
                let elapsed = time.elapsed();
                if elapsed < frame_time {
                    std::thread::sleep(frame_time - elapsed);
                }
                time = Instant::now();
            },

            _ => {}
        }
    });
}
