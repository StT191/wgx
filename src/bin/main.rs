#![allow(unused)]

// imports
use std::time::{Duration, Instant};

use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};

use wgfx::*;


// main
fn main() {

    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();
    window.set_title("WgFx");

    let mut gx = Gx::new(&window);


    // pipeline
    let layout = gx.binding(&[]);

    let render_pipeline = gx.render_pipeline(
        &gx.load_glsl("shaders/main.vert", ShaderType::Vertex),
        &gx.load_glsl("shaders/main.frag", ShaderType::Fragment),
        vertex_desc![0 => Float2],
        &layout
    );


    // vertex data
    let data = [
        (-0.25, -0.5), (-0.5, -0.5), (-0.5, 0.5),
        (0.25, -0.5), (0.5, -0.5), (0.5, 0.5),
    ];

    let vertices1 = gx.vertex_mapped::<(f32, f32)>(3).fill_from_slice(&data[0..3]);
    let vertices2 = gx.vertex_mapped::<(f32, f32)>(3).fill_from_slice(&data[3..6]);


    // bind data
    let bound = gx.bind(&layout, &[]);



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
                gx.resize(size.width, size.height);
            },

            Event::RedrawRequested(_) => {
                gx.draw_frame(
                    wgpu::Color::GREEN,
                    &[
                        (&render_pipeline, &vertices1, 0..3, &bound),
                        (&render_pipeline, &vertices2, 0..3, &bound),
                    ],
                );
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
