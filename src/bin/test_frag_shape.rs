#![allow(unused)]

// imports
use cgmath::*;
use std::{time::{Instant}};

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};

use wgx::*;


// main
fn main() {

    const DEPTH_TESTING:bool = false;
    const MSAA:u32 = 1;
    const ALPHA_BLENDING:bool = true;


    let (width, height) = (700, 700);

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((width, height)));
    window.set_title("WgFx");

    let mut gx = Wgx::new(Some(&window));
    let mut target = gx.surface_target((width, height), DEPTH_TESTING, MSAA).expect("render target failed");


    // pipeline
    let vs = gx.load_glsl(include_str!("../../shaders/proj_2d_vNorm_passCol.vert"), ShaderType::Vertex);
    let fs = gx.load_glsl(include_str!("../../shaders/circle_flat.frag"), ShaderType::Fragment);

    let layout = gx.binding(&[
        binding!(0, VERTEX, UniformBuffer, 64),
        binding!(1, FRAGMENT, UniformBuffer, 8),
        // binding!(1, VERTEX, UniformBuffer, 8),
    ]);

    let pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, &vs, &fs,
        vertex_desc![0 => Float32x2, 1 => Float32x4],
        Primitive::TriangleList, &layout
    );

    let color = Color::RED.f32();

    // corners
    let c = [
        ([-1.0, -1.0f32], color),
        ([ 1.0, -1.0],    color),
        ([ 1.0,  1.0],    color),
        ([-1.0,  1.0],    color),
    ];

    // vertices
    let data = [
        c[0], c[1], c[2],
        c[0], c[2], c[3],
    ];
    let vertices = gx.buffer_from_data(BuffUse::VERTEX, &data[..]);


    // projection
    let projection = Matrix4::<f32>::identity();

    let mut pj_buffer = gx.buffer_from_data(BuffUse::UNIFORM | BuffUse::COPY_DST, AsRef::<[f32; 16]>::as_ref(&projection));
    // let mut tf_buffer = gx.buffer_from_data(BuffUse::UNIFORM | BuffUse::COPY_DST, AsRef::<[f32; 16]>::as_ref(&projection));

    let mut dim_buffer = gx.buffer_from_data(BuffUse::UNIFORM | BuffUse::COPY_DST, &[1.0, 1.0]);

    // binding
    let binding = gx.bind(&layout, &[
        bind!(0, Buffer, &pj_buffer, 0, None),
        // bind!(1, Buffer, tf_buffer.slice(..)),
        bind!(1, Buffer, &dim_buffer, 0, None),
    ]);


    // event loop
    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                *control_flow = ControlFlow::Exit;
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                target.update(&gx, (size.width, size.height));

                let (width, height) = (size.width as f32, size.height as f32);

                let (w, h) = (width/4.0, height/4.0);


                let scale:Matrix4::<f32> =
                    /*Matrix4::from_angle_x(Deg(-60.0)) *
                    Matrix4::from_angle_y(Deg(60.0)) **/
                    // Matrix4::from_angle_z(Deg(-45.0)) *
                    Matrix4::from_nonuniform_scale(1.0, 0.3, 1.0) *
                    Matrix4::from_nonuniform_scale(w, h, 1.0)
                ;


                let dim_x = (
                    scale *
                    Vector4::<f32>::new(1.0, 0.0, 0.0, 0.0)
                ).magnitude();

                let dim_y = (
                    scale *
                    Vector4::<f32>::new(0.0, 1.0, 0.0, 0.0)
                ).magnitude();


                // projection
                let projection =
                    window_fov_projection(30.0, width, height) *
                    scale
                ;

                gx.write_buffer(&mut pj_buffer, 0, AsRef::<[f32; 16]>::as_ref(&projection));
                gx.write_buffer(&mut dim_buffer, 0, &[dim_x, dim_y]);
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
                    encoder.draw(
                        attachment,
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

    let rx = false;
}
