#![allow(unused)]

// imports
use std::{time::{Instant}, include_str};

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};

use wgx::*;


// main
fn main() {

    const DEPTH_TESTING:bool = true;
    const ALPHA_BLENDING:bool = false;
    const MSAA:u32 = 8;


    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((800, 600)));
    window.set_title("WgFx");

    let mut gx = Wgx::new(Some(&window));
    let mut target = gx.surface_target((800, 600), DEPTH_TESTING, MSAA).expect("render target failed");


    // global pipeline
    let vs = gx.load_glsl(include_str!("../../shaders/pass_plain.vert"), ShaderType::Vertex);
    let fs = gx.load_glsl(include_str!("../../shaders/color_flat.frag"), ShaderType::Fragment);


    let vertex_desc = vertex_desc![0 => Float3];

    let layout = gx.binding(&[
        binding!(0, FRAGMENT, UniformBuffer, 16),
    ]);

    let pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, &vs, &fs,
        vertex_desc, Primitive::TriangleStrip, &layout
    );

    // first render

    // colors
    let color_buffer = gx.buffer_from_data(BuffUse::UNIFORM, &[
        Color::from([1.0, 0.0, 0.0]).f32()
    ]);

    let binding = gx.bind(&layout, &[
        bind!(0, Buffer, color_buffer.slice(..)),
    ]);


    // vertices
    let data:[(f32, f32, f32); 4] = [
        (-0.5, -0.5, 0.0),
        ( 0.5, -0.5, 0.0),
        (-0.5,  0.5, 0.0),
        ( 0.5,  0.5, 0.0),
    ];
    let vertices = gx.buffer_from_data(BuffUse::VERTEX, &data[..]);



    // event loop

    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                *control_flow = ControlFlow::Exit;
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                target.update(&gx, (size.width, size.height));
            },

            Event::WindowEvent {
                event:WindowEvent::KeyboardInput{
                    input: winit::event::KeyboardInput {
                        virtual_keycode:Some(winit::event::VirtualKeyCode::R), ..
                    }, ..
                }, ..
            } => {
                window.request_redraw();
            },

            Event::RedrawRequested(_) => {

                let then = Instant::now();

                target.with_encoder_frame(&gx, |encoder, attachment| {
                    encoder.draw(attachment,
                        Some(Color::GREEN),
                        &[
                            (&pipeline, &binding, vertices.slice(..), 0..data.len() as u32),
                        ],
                    );
                });


                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}
