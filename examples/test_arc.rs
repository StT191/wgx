#![allow(unused)]

// imports
use cgmath::*;
use std::{time::{Instant}};

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
};

use wgx::*;


// main
fn main() {

    const DEPTH_TESTING:bool = false;
    const MSAA:u32 = 4;
    const ALPHA_BLENDING:bool = true;


    let (width, height) = (1000, 1000);

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((width, height)));
    window.set_title("WgFx");

    let mut gx = Wgx::new(Some(&window), 0, None);
    let mut target = gx.surface_target((width, height), DEPTH_TESTING, MSAA).expect("render target failed");


    // pipeline
    let shader = gx.load_wgsl(include_str!("../shaders/arc.wgsl"));

    let layout = gx.binding(&[
        binding!(0, VERTEX, UniformBuffer, 64),
        binding!(1, VERTEX, UniformBuffer, 64),
        // binding!(1, VERTEX, UniformBuffer, 8),
    ]);

    let pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, (&shader, "vs_main"), (&shader, "fs_main"),
        &[vertex_desc!(Vertex, 0 => Float32x2, 1 => Float32x4)],
        Primitive::TriangleList, &[], &[&layout]
    );

    let color = Color::RED.f32();


    // path
    let q = f32::sqrt(2.0);
    let z = 0.0;

    // corners
    let c = [
        ([ z,  z], color),
        ([-q,  z], color),
        ([ z, -q], color),
        ([ q,  z], color),
        ([ z,  q], color),
    ];

    // vertices
    // let data = [
    //     c[0], c[1], c[2],
    //     c[0], c[2], c[3],
    // ];
    let data = [
        c[1], c[0], c[2],
        c[2], c[0], c[3],
        c[3], c[0], c[4],
        c[4], c[0], c[1],
    ];
    let vertices = gx.buffer_from_data(BuffUse::VERTEX, &data[..]);


    // projection
    let fill_matrix = Matrix4::<f32>::identity();

    let mut clip_buffer = gx.buffer_from_data(BuffUse::UNIFORM | BuffUse::COPY_DST, AsRef::<[f32; 16]>::as_ref(&fill_matrix));
    let mut pix_buffer = gx.buffer_from_data(BuffUse::UNIFORM | BuffUse::COPY_DST, AsRef::<[f32; 16]>::as_ref(&fill_matrix));


    // binding
    let binding = gx.bind(&layout, &[
        bind!(0, Buffer, &clip_buffer),
        bind!(1, Buffer, &pix_buffer),
    ]);


    // render bundles
    let bundles = [target.render_bundle(&gx, |mut rpass| {
        rpass.set_pipeline(&pipeline);
        rpass.set_bind_group(0, &binding, &[]);
        rpass.set_vertex_buffer(0, vertices.slice(..));
        rpass.draw(0..data.len() as u32, 0..1);
    })];


    // matrix
    const DA:f32 = 3.0;
    const DT:f32 = 0.01;

    let (mut width, mut height) = (width as f32, height as f32);
    let (mut w, mut h) = (0.33, 0.33);

    let mut obj_scale_matrix = Matrix4::from_nonuniform_scale(w*width, h*height, 1.0);

    let mut rot_matrix = Matrix4::<f32>::identity();

    let projection =
        window_fov_projection(45.0, width, height)
        // flat_window_projection(width, height, 0.0) *
        // Matrix4::from_translation((width/2.0, height/2.0, 0.0).into())
    ;

    let mut clip_matrix = projection * rot_matrix * obj_scale_matrix;

    let mut pixel_matrix = Matrix4::from_nonuniform_scale(width/2.0, height/2.0, 1.0) * clip_matrix;


    // event loop
    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                *control_flow = ControlFlow::Exit;
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                target.update(&gx, (size.width, size.height));

                width = size.width as f32;
                height = size.height as f32;

                // projection
                obj_scale_matrix = Matrix4::from_nonuniform_scale(w*width, h*height, 1.0);

                clip_matrix = projection * rot_matrix * obj_scale_matrix;
                pixel_matrix = Matrix4::from_nonuniform_scale(width/2.0, height/2.0, 1.0) * clip_matrix;

                gx.write_buffer(&mut clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
                gx.write_buffer(&mut pix_buffer, 0, AsRef::<[f32; 16]>::as_ref(&pixel_matrix));
            },

            Event::WindowEvent { event:WindowEvent::KeyboardInput { input: KeyboardInput {
                virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
            }, ..}, ..} => {
                let mut redraw = true;
                match keycode {
                    VirtualKeyCode::I => { rot_matrix = Matrix4::from_angle_x(Deg( DA)) * rot_matrix; },
                    VirtualKeyCode::K => { rot_matrix = Matrix4::from_angle_x(Deg(-DA)) * rot_matrix; },
                    VirtualKeyCode::J => { rot_matrix = Matrix4::from_angle_y(Deg( DA)) * rot_matrix; },
                    VirtualKeyCode::L => { rot_matrix = Matrix4::from_angle_y(Deg(-DA)) * rot_matrix; },
                    VirtualKeyCode::U => { rot_matrix = Matrix4::from_angle_z(Deg( DA)) * rot_matrix; }, // on flat pojection
                    VirtualKeyCode::O => { rot_matrix = Matrix4::from_angle_z(Deg(-DA)) * rot_matrix; }, // on flat pojection

                    VirtualKeyCode::W => { h += DT; },
                    VirtualKeyCode::S => { h -= DT; },
                    VirtualKeyCode::A => { w -= DT; },
                    VirtualKeyCode::D => { w += DT; },

                    VirtualKeyCode::R => {
                        rot_matrix = Matrix4::identity();
                        w = 0.33;
                        h = 0.33;
                    },

                    _ => { redraw = false; }
                } {
                    if redraw {
                        obj_scale_matrix = Matrix4::from_nonuniform_scale(w*width, h*height, 1.0);

                        clip_matrix = projection * rot_matrix * obj_scale_matrix;
                        pixel_matrix = Matrix4::from_nonuniform_scale(width/2.0, height/2.0, 1.0) * clip_matrix;

                        gx.write_buffer(&mut clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
                        gx.write_buffer(&mut pix_buffer, 0, AsRef::<[f32; 16]>::as_ref(&pixel_matrix));

                        window.request_redraw();
                    }
                }
            },

            Event::RedrawRequested(_) => {

                let then = Instant::now();

                target.with_encoder_frame(&gx, |encoder, attachment| {
                    encoder.render_bundles(attachment, Some(Color::GREEN), &bundles);
                }).expect("frame error");

                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}
