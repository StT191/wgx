#![allow(unused)]

use std::{time::{Instant, Duration}};
use futures::executor::block_on;
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
};
use wgx::{*, cgmath::*};


fn main() {

    const DEPTH_TESTING:bool = false;
    const MSAA:u32 = 1;
    const ALPHA_BLENDING:bool = false;


    let (width, height) = (1280, 900);

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((width, height)));
    window.set_title("WgFx");

    let mut gx = block_on(Wgx::new(Some(&window), Features::PUSH_CONSTANTS, limits!{max_push_constant_size: 4})).unwrap();
    let mut target = gx.surface_target((width, height), DEPTH_TESTING, MSAA).unwrap();


    // pipeline
    let shader = gx.load_wgsl(include_wgsl_module!("./shaders/sdf-balls.wgsl"));


    let layout = gx.layout(&[
        binding!(0, Shader::VERTEX_FRAGMENT, UniformBuffer, 12),
        binding!(1, Shader::FRAGMENT, UniformBuffer, 4),
        // binding!(2, Shader::FRAGMENT, UniformBuffer, 4),
    ]);


    let pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, (&shader, "vs_main"), (&shader, "fs_main"),
        &[vertex_desc!(Vertex, 0 => Float32x2)],
        Primitive::TriangleList, Some((push_constants![0..4 => Shader::FRAGMENT], &[&layout]))
    );

    // vertices
    let vertex_data = [
        [-1.0, -1.0f32], [ 1.0, -1.0f32], [ 1.0,  1.0f32],
        [-1.0, -1.0f32], [ 1.0,  1.0f32], [-1.0,  1.0f32],
    ];
    let vertices = gx.buffer_from_data(BufUse::VERTEX, &vertex_data[..]);


    // data
    // const DA:f32 = 3.0;
    const DT:f32 = 0.01;

    let (mut width, mut height) = (width as f32, height as f32);
    let (mut w, mut h) = (1.0, 1.0);

    let time = Instant::now();


    // buffer
    let mut viewport_buffer = gx.buffer_from_data(BufUse::UNIFORM | BufUse::COPY_DST, &[
        width as f32, height as f32, width as f32 / height as f32
    ]);
    let mut scale_buffer = gx.buffer_from_data(BufUse::UNIFORM | BufUse::COPY_DST, &[1.0 as f32]);
    // let mut t_buffer = gx.buffer_from_data(BufUse::UNIFORM | BufUse::COPY_DST, &[time.elapsed().as_secs_f32()]);

    // binding
    let binding = gx.bind(&layout, &[
        bind!(0, Buffer, &viewport_buffer),
        bind!(1, Buffer, &scale_buffer),
        // bind!(2, Buffer, &t_buffer),
    ]);

    // render bundles
    let bundles = [target.render_bundle(&gx, |rpass| {
        rpass.set_pipeline(&pipeline);
        rpass.set_bind_group(0, &binding, &[]);
        rpass.set_vertex_buffer(0, vertices.slice(..));
        rpass.draw(0..vertex_data.len() as u32, 0..1);
    })];


    // frame rate and counter

    let mut frame_timer = ticks::TickTimer::from_ticks_per_sec(40.0, 1.0);
    // let mut frame_counter = ticks::TickCounter::from_secs(5.0);


    // event loop
    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::WaitUntil(frame_timer.next_tick); // next frame
        // *control_flow = ControlFlow::Wait;

        match event {

            Event::NewEvents(_) => {
                if (frame_timer.elapsed()) {
                    window.request_redraw(); // request frame
                }
            },

            Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                *control_flow = ControlFlow::Exit;
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                target.update(&gx, (size.width, size.height));

                width = size.width as f32;
                height = size.height as f32;

                // write buffer
                gx.write_buffer(&mut viewport_buffer, 0, &[width, height, width / height]);
            },

            Event::WindowEvent { event:WindowEvent::KeyboardInput { input: KeyboardInput {
                virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
            }, ..}, ..} => {

                let mut update = true;

                match keycode {
                    // VirtualKeyCode::I => { rot_matrix = Matrix4::from_angle_x(Deg( DA)) * rot_matrix; },
                    // VirtualKeyCode::K => { rot_matrix = Matrix4::from_angle_x(Deg(-DA)) * rot_matrix; },
                    // VirtualKeyCode::J => { rot_matrix = Matrix4::from_angle_y(Deg( DA)) * rot_matrix; },
                    // VirtualKeyCode::L => { rot_matrix = Matrix4::from_angle_y(Deg(-DA)) * rot_matrix; },
                    // VirtualKeyCode::U => { rot_matrix = Matrix4::from_angle_z(Deg( DA)) * rot_matrix; },
                    // VirtualKeyCode::O => { rot_matrix = Matrix4::from_angle_z(Deg(-DA)) * rot_matrix; },

                    VirtualKeyCode::W => { h += DT; },
                    VirtualKeyCode::S => { h -= DT; },
                    // VirtualKeyCode::A => { w -= DT; },
                    // VirtualKeyCode::D => { w += DT; },

                    VirtualKeyCode::R => {
                        // rot_matrix = Matrix4::identity();
                        w = 0.4;
                        h = 0.4;
                    },

                    _ => { update = false; }
                } {
                    if update {
                        gx.write_buffer(&mut scale_buffer, 0, &[h]);
                        // window.request_redraw();
                    }
                }
            },

            Event::RedrawRequested(_) => {

                // next frame time
                let shift = frame_timer.next();

                /*if (shift.was_next_tick) {
                    frame_counter.add_delta_time(shift.delta_time);
                }*/

                *control_flow = ControlFlow::WaitUntil(frame_timer.next_tick);

                // draw
                // gx.write_buffer(&mut t_buffer, 0, &[time.elapsed().as_secs_f32()]);

                target.with_encoder_frame(&gx, |encoder, attachment| {
                    // encoder.render_bundles(attachment, Some(Color::BLACK), &bundles);

                    encoder.with_render_pass(attachment, Some(Color::BLACK), |mut rpass| {

                        // rpass.execute_bundles(bundles.iter());

                        rpass.set_pipeline(&pipeline);
                        rpass.set_bind_group(0, &binding, &[]);
                        rpass.set_vertex_buffer(0, vertices.slice(..));
                        rpass.set_push_constants(Shader::FRAGMENT, 0, &time.elapsed().as_secs_f32().to_ne_bytes());
                        rpass.draw(0..vertex_data.len() as u32, 0..1);
                    });

                }).expect("frame error");


                // statistics
                /*frame_counter.add_one();

                if let Some(count) = frame_counter.tick_count() {
                    println!("{:?}", count);
                }*/
            },

            _ => {}
        }
    });
}