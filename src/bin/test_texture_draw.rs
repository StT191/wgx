#![allow(unused)]

// imports
use std::{time::{Instant}, include_str};

use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};

use wgx::*;


// main
fn main() {

    const DEPTH_TESTING:bool = false;
    const ALPHA_BLENDING:bool = true;
    const MSAA:u32 = 1;


    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();
    window.set_title("WgFx");


    let mut gx = Gx::new(&window, DEPTH_TESTING, MSAA);


    // global pipeline
    let vs = gx.load_glsl(include_str!("../../shaders/pass_texcoord.vert"), ShaderType::Vertex);
    let fs = gx.load_glsl(include_str!("../../shaders/texture_flat.frag"), ShaderType::Fragment);

    let layout = gx.binding(&[
        binding!(0, FRAGMENT, SampledTexture),
        binding!(1, FRAGMENT, Sampler)
    ]);

    let pipeline = gx.render_pipeline(
        TexOpt::Output, DEPTH_TESTING, ALPHA_BLENDING, MSAA, &vs, &fs, vertex_desc![0 => Float3, 1 => Float2],
        Primitive::TriangleList, &layout
    );

    let sampler = gx.sampler();

    // colors
    let color_texture = gx.texture(2, 1, 1, TexUse::SAMPLED | TexUse::COPY_DST, TexOpt::Texture);
    gx.write_texture(&color_texture, (0, 0, 2, 1), &[
        (255u8, 0u8, 0u8, 255u8), (255, 0, 0, 255),
    ]);
    let color_texture_view = color_texture.create_default_view();


    // draw texture
    let size = window.inner_size();

    let draw_pipeline = gx.render_pipeline(
        TexOpt::Texture, false, false, 8, &vs, &fs, vertex_desc![0 => Float3, 1 => Float2],
        Primitive::LineStrip, &layout
    );

    let draw_texture = gx.texture(size.width, size.height, 1, TexUse::SAMPLED | TexUse::OUTPUT_ATTACHMENT, TexOpt::Texture);

    let draw_msaa_texture = gx.msaa_texture(size.width, size.height, 8, TexOpt::Texture);

    let draw_texture_view = draw_texture.create_default_view();
    let draw_msaa_texture_view = draw_msaa_texture.create_default_view();

    // draw_vertices
    const A:usize = 4;

    let data:[((f32, f32, f32), (f32, f32)); A] = [
        (( 0.5,  0.5, 0.0), (1.0, 1.0)),
        ((-0.5,  0.5, 0.0), (1.0, 1.0)),
        ((-0.5, -0.5, 0.0), (1.0, 1.0)),
        (( 0.5,  0.5, 0.0), (1.0, 1.0)),
    ];

    let draw_vertices = gx.buffer_from_data(BuffUse::VERTEX, &data[..]);

    let binding = gx.bind(&layout, &[
        bind!(0, TextureView, &color_texture_view),
        bind!(1, Sampler, &sampler),
    ]);


    // first render
    gx.with_encoder(|encoder, gx| {
        encoder.draw((&draw_texture_view, None, Some(&draw_msaa_texture_view)), Some(Color::TRANSPARENT), &[
            (&draw_pipeline, &binding, draw_vertices.slice(..), 0..A as u32),
        ]);
    });


    // real draw
    let binding = gx.bind(&layout, &[
        bind!(0, TextureView, &draw_texture_view),
        bind!(1, Sampler, &sampler),
    ]);


    let data = [
        ((-1.0f32, -1.0f32, 0.0f32), (0.0f32, 0.0f32)),
        (( 1.0, -1.0, 0.0), (1.0, 0.0)),
        (( 1.0,  1.0, 0.0), (1.0, 1.0)),

        (( 1.0,  1.0, 0.0), (1.0, 1.0)),
        ((-1.0,  1.0, 0.0), (0.0, 1.0)),
        ((-1.0, -1.0, 0.0), (0.0, 0.0)),
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
