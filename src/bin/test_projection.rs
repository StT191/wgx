#![allow(unused)]

// imports
use std::{time::{Instant}, include_str, convert::AsRef};

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};

use wgx::*;
use cgmath::*;


// main
fn main() {

    const DEPTH_TESTING:bool = true;
    const ALPHA_BLENDING:bool = false;
    const MSAA:u32 = 8;


    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();
    window.set_title("WgFx");
    window.set_inner_size(PhysicalSize::<u32>::from((600.0, 600.0)));


    let mut gx = Gx::new(&window, DEPTH_TESTING, MSAA);


    // global pipeline
    let vs = gx.load_glsl(include_str!("../../shaders/projection_texcoord.vert"), ShaderType::Vertex);
    let fs = gx.load_glsl(include_str!("../../shaders/texture_flat.frag"), ShaderType::Fragment);

    let vertex_desc = vertex_desc![0 => Float3, 1 => Float2];

    let layout = gx.binding(&[
        binding!(0, FRAGMENT, SampledTexture),
        binding!(1, FRAGMENT, Sampler),
        binding!(2, VERTEX, UniformBuffer, 64),
    ]);

    let pipeline = gx.render_pipeline(
        TexOpt::Output, DEPTH_TESTING, ALPHA_BLENDING, MSAA, &vs, &fs,
        vertex_desc, Primitive::TriangleList, &layout
    );

    // first render

    // colors
    let texture = gx.texture(2, 1, 1, TexUse::COPY_DST | TexUse::COPY_SRC  | TexUse::SAMPLED, TexOpt::Texture);

    gx.write_texture(&texture, (0, 0, 2, 1), &[
        [255, 0, 0, 255u8], // r
        [0, 0, 255, 50], // b
    ]);


    /*gx.with_encoder(|encoder, gx| {
        let buff = gx.buffer_from_data::<(u8, u8, u8, u8)>(BufferUsage::COPY_SRC, &[
            (255, 0, 0, 255), (0, 0, 255, 50),
        ]);

        buffer_to_texture(encoder, &buff, (2, 1, 0), &texture, (0, 0, 2, 1));
    });*/

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

    let projection =
        gx.projection *
        Matrix4::from_translation((0.0, 0.0, 0.0).into()) *
        // Matrix4::from_angle_z(Deg(30.0)) *
        Matrix4::from_angle_y(Deg(45.0)) *
        Matrix4::from_translation((0.0, 0.0, -400.0).into()) *
        // Matrix4::from_angle_x(Deg(20.0)) *
        Matrix4::from_scale(1000.0 * 1.0)
    ;


    let pj_buffer = gx.buffer_from_data(BuffUse::UNIFORM, AsRef::<[f32; 16]>::as_ref(&projection));


    let binding = gx.bind(&layout, &[
        bind!(0, TextureView, &texture_view),
        bind!(1, Sampler, &sampler),
        bind!(2, Buffer, pj_buffer.slice(..)),
    ]);



    // event loop

    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                *control_flow = ControlFlow::Exit;
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                gx.update(size.width, size.height);
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


                gx.with_encoder_frame(|encoder, gx| {
                    gx.draw(encoder,
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
