#![allow(unused)]

// imports
use std::{time::{Instant}};

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::*,
};

use wgx::*;
use cgmath::*;


// main
fn main() {

    const DEPTH_TESTING:bool = true;
    const ALPHA_BLENDING:bool = false;
    const MSAA:u32 = 4;


    let event_loop = EventLoop::new();


    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((800, 500)));
    window.set_title("WgFx");


    let mut gx = Wgx::new(Some(&window), Features::empty(), limits!{});
    let mut target = gx.surface_target((800, 500), DEPTH_TESTING, MSAA).expect("render target failed");


    // global pipeline
    let shader = gx.load_wgsl(include_str!("../shaders/projection_texture.wgsl"));

    let pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, (&shader, "vs_main"), (&shader, "fs_main"),
        &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        Primitive::TriangleList, None,
    );


    let texture = gx.texture((2, 1), 1, TexUse::COPY_DST | TexUse::COPY_SRC  | TexUse::TEXTURE_BINDING, TEXTURE);

    gx.write_texture(&texture, (0, 0, 2, 1), &[
        [255, 0, 0, 255u8], // r
        [0, 0, 255, 50], // b
    ]);


    let d = 1.0 * 0.1;
    let a = -1.0 * 0.1;
    let v = -3.0 * 0.1;

    let c = [
        ([-1.0, 0.1, 0.0f32], [1.0, 0.0f32]), // tl
        ([1.0, 0.1, 0.0], [1.0, 0.0]), // tr
        ([1.0, -0.2, 0.0], [0.0, 0.0]), // br
        ([-1.0, -0.2, 0.0], [1.0, 0.0]), // bl

        ([-0.25, 0.5, a], [0.0, 0.0]), // tl
        ([0.25, 0.5, a], [0.0, 0.0]), // tr
        ([0.25, -0.5, a], [1.0, 0.0]), // br
        ([-0.25, -0.5, a], [0.0, 0.0]), // bl

        ([-0.25+d, 0.5+d, v], [1.0, 0.0]), // tl
        ([0.25+d, 0.5+d, v], [1.0, 0.0]), // tr
        ([0.25+d, -0.5+d, v], [0.0, 0.0]), // br
        ([-0.25+d, -0.5+d, v], [1.0, 0.0]), // bl
    ];


    // vertices
    let data = [
        c[0+0], c[1+0], c[2+0], c[2+0], c[3+0], c[0+0],
        c[0+4], c[1+4], c[2+4], c[2+4], c[3+4], c[0+4],
        c[0+8], c[1+8], c[2+8], c[2+8], c[3+8], c[0+8],
    ];
    let vertices = gx.buffer_from_data(BuffUse::VERTEX, &data[..]);



    // texture + sampler + projection

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = gx.sampler();


    // projection matrixes

    let mut projection =
        // Matrix4::identity()
        window_fov_projection(30.0, 800.0, 500.0) *
        // Matrix4::from_translation((0.0, 0.0, 0.5).into()) *
        // Matrix4::from_nonuniform_scale(1.0, 1.0, -1.0)
        // Matrix4::from_angle_y(Deg(45.0)) *
        // Matrix4::from_translation((0.0, 0.0, -400.0).into()) *
        // Matrix4::from_scale(0.9 * 1.0)
        Matrix4::from_scale(400.0)
    ;


    let pj_buffer = gx.buffer_from_data(BuffUse::UNIFORM | BuffUse::COPY_DST, AsRef::<[f32; 16]>::as_ref(&projection));


    let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
        bind!(0, Buffer, &pj_buffer, 0, None),
        bind!(1, TextureView, &texture_view),
        bind!(2, Sampler, &sampler),
    ]);


    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;

    let mut rx = 0.0;
    let mut ry = 0.0;
    let mut rz = 0.0;


    const DS:f32 = 10.0;
    const DA:f32 = 10.0;

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

            Event::WindowEvent { event:WindowEvent::KeyboardInput { input: KeyboardInput {
                virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
            }, ..}, ..} => {
                let mut redraw = true;
                match keycode {
                    VirtualKeyCode::I => { z -= DS; },
                    VirtualKeyCode::K => { z += DS; },
                    VirtualKeyCode::J => { x -= DS; },
                    VirtualKeyCode::L => { x += DS; },
                    VirtualKeyCode::U => { y += DS; },
                    VirtualKeyCode::O => { y -= DS; },

                    VirtualKeyCode::A => { ry -= DA; },
                    VirtualKeyCode::D => { ry += DA; },
                    VirtualKeyCode::W => { rx -= DA; },
                    VirtualKeyCode::S => { rx += DA; },
                    VirtualKeyCode::Q => { rz += DA; },
                    VirtualKeyCode::E => { rz -= DA; },

                    VirtualKeyCode::R => {
                        x = 0.0; y = 0.0; z = 0.0;
                        rx = 0.0; ry = 0.0; rz = 0.0;
                    },
                    _ => { redraw = false; }
                } {
                    if redraw { window.request_redraw(); }
                }
            },

            Event::RedrawRequested(_) => {

                let then = Instant::now();

                let (width, height) = target.size();

                projection =
                    window_fov_projection(30.0, width as f32, height as f32) *
                    Matrix4::from_translation((x, y, z).into()) *
                    Matrix4::from_angle_z(Deg(rz)) *
                    Matrix4::from_angle_y(Deg(ry)) *
                    Matrix4::from_angle_x(Deg(rx)) *
                    Matrix4::from_scale(width as f32 / 4.0)
                ;

                gx.write_buffer(&pj_buffer, 0, AsRef::<[f32; 16]>::as_ref(&projection));


                target.with_encoder_frame(&gx, |encoder, attachment| {

                    encoder.with_render_pass(attachment, Some(Color::GREEN), |mut rpass| {
                        rpass.set_pipeline(&pipeline);
                        rpass.set_bind_group(0, &binding, &[]);
                        rpass.set_vertex_buffer(0, vertices.slice(..));
                        rpass.draw(0..data.len() as u32, 0..1);
                    });

                }).expect("frame error");


                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}
