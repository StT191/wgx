#![allow(unused)]

// imports
use std::{time::{Instant}};

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};

use wgx::*;


// main
fn main() {

    const DEPTH_TESTING:bool = true;
    const MSAA:u32 = 4;
    const ALPHA_BLENDING:bool = true;


    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((600, 600)));
    window.set_title("WgFx");


    let mut gx = Wgx::new(Some(&window));
    let mut target = gx.surface_target((600, 600), DEPTH_TESTING, MSAA).expect("render target failed");


    // global pipeline
    let vs = gx.load_glsl(include_str!("../../shaders/pass_texC.vert"), ShaderType::Vertex);
    let fs = gx.load_glsl(include_str!("../../shaders/texture_flat.frag"), ShaderType::Fragment);


    let layout = gx.binding(&[
        binding!(0, FRAGMENT, SampledTexture),
        binding!(1, FRAGMENT, Sampler)
    ]);


    let pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, &vs, &fs,
        vertex_desc![0 => Float32x3, 1 => Float32x2],
        Primitive::TriangleList, &layout
    );

    // first render

    // colors
    let texture = gx.texture((2, 1), 1, TexUse::COPY_DST | TexUse::COPY_SRC  | TexUse::TEXTURE_BINDING, TEXTURE);

    gx.write_texture(&texture, (0, 0, 2, 1), &[
        (255u8, 0u8, 0u8, 255u8), (0, 0, 255, 50),
    ]);


    /*gx.with_encoder(|encoder, gx| {
        let buff = gx.buffer_from_data::<(u8, u8, u8, u8)>(BufferUsage::COPY_SRC, &[
            (255, 0, 0, 255), (0, 0, 255, 50),
        ]);

        buffer_to_texture(encoder, &buff, (2, 1, 0), &texture, (0, 0, 2, 1));
    });*/


    // vertices
    let data = [
        ([-0.25, -0.5, 0.35f32], [0.0, 0.0f32]),
        ([0.0, -0.5, 0.35], [1.0, 0.0]),
        ([-1.0, 0.5, 0.1], [0.0, 0.0]),

        ([0.25, -0.5, 0.1], [0.0, 0.0]),
        ([0.5, -0.5, 0.1], [1.0, 0.0]),
        ([-1.0, 0.5, 0.6], [0.0, 0.0]),

        ([-0.75, -0.5, 0.1], [0.0, 0.0]),
        ([-1.0, -0.5, 0.1], [1.0, 0.0]),
        ([-0.3, 0.5, 0.312], [1.0, 0.0]),
    ];
    let vertices = gx.buffer_from_data(BuffUse::VERTEX, &data[..]);


    // texture + sampler

    let texture_view = texture.create_default_view();
    let sampler = gx.sampler();

    let binding = gx.bind(&layout, &[
        bind!(0, TextureView, &texture_view),
        bind!(1, Sampler, &sampler),
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
}
