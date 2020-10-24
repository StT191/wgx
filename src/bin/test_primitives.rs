#![allow(unused)]

// imports
use futures::executor::block_on;

use image;

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
    const MSAA:u32 = 8;


    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();
    window.set_title("WgFx");


    let mut gx = Gx::new(&window, DEPTH_TESTING, MSAA);

    // clear
    // gx.pass_frame_render(Some(Color::GREEN), &[]);


    // global pipeline
    let vs = gx.load_glsl(include_str!("../../shaders/pass_texcoord.vert"), ShaderType::Vertex);
    let fs = gx.load_glsl(include_str!("../../shaders/texture_flat.frag"), ShaderType::Fragment);

    // layout
    let layout = gx.binding(&[
        binding!(0, FRAGMENT, SampledTexture),
        binding!(1, FRAGMENT, Sampler)
    ]);


    // colors
    let color_texture = gx.texture(3, 1, 1, TexUse::SAMPLED | TexUse::COPY_DST, TexOpt::Texture);
    gx.write_texture(&color_texture, (0, 0, 3, 1), &[
        (255u8, 0u8, 0u8, 255u8), (0, 255, 0, 255), (0, 0, 255, 255),
    ]);
    let color_texture_view = color_texture.create_default_view();

    let sampler = gx.sampler();

    // binding
    let binding = gx.bind(&layout, &[
        bind!(0, TextureView, &color_texture_view),
        bind!(1, Sampler, &sampler),
    ]);


    // triangle pipeline
    let t_pipeline = gx.render_pipeline(
        TexOpt::Output, DEPTH_TESTING, ALPHA_BLENDING, MSAA, &vs, &fs, vertex_desc![0 => Float3, 1 => Float2],
        Primitive::TriangleStrip, &layout
    );

    let t_data = [
        (( 0.5f32,  0.5f32, 0.0f32), (0.0f32, 0.0f32)),
        ((-0.5,  0.5, 0.0), (0.0, 0.0)),
        (( 0.5, -0.5, 0.0), (0.0, 0.0)),
        ((-0.5, -0.5, 0.0), (0.0, 0.0)),
    ];

    let t_vertices = gx.buffer_from_data(BuffUse::VERTEX, &t_data[..]);


    // lines pipeline
    let l_pipeline = gx.render_pipeline(
        TexOpt::Output, DEPTH_TESTING, ALPHA_BLENDING, MSAA, &vs, &fs, vertex_desc![0 => Float3, 1 => Float2],
        Primitive::LineStrip, &layout
    );

    let l_data = [
        (( 0.5f32,  0.5f32, 0.0f32), (1.0f32, 0.0f32)),
        ((-0.5,  0.5, 0.0), (1.0, 0.0)),
        ((-0.5, -0.5, 0.0), (1.0, 0.0)),
        (( 0.5, -0.5, 0.0), (1.0, 0.0)),
        (( 0.5,  0.5, 0.0), (1.0, 0.0)),
        (( -1.0, -1.0, 0.0), (1.0, 0.0)),
    ];

    let l_vertices = gx.buffer_from_data(BuffUse::VERTEX, &l_data[..]);


    // picture pipeline
    let img = image::open("/home/stefan/dev/wgx/logo_red.png")
        .expect("failed loading image")
        .into_rgba();

    let (w, h) = (img.width(), img.height());

    let image_texture = gx.texture(w, h, 1, TexUse::SAMPLED | TexUse::COPY_DST, TexOpt::Texture);

    gx.write_texture(&image_texture, (0, 0, w, h), &img.as_raw().as_slice() );

    let image_texture_view = image_texture.create_default_view();


    // binding
    let img_binding = gx.bind(&layout, &[
        bind!(0, TextureView, &image_texture_view),
        bind!(1, Sampler, &sampler),
    ]);


    let i_pipeline = gx.render_pipeline(
        TexOpt::Output, DEPTH_TESTING, ALPHA_BLENDING, MSAA, &vs, &fs, vertex_desc![0 => Float3, 1 => Float2],
        Primitive::TriangleStrip, &layout
    );

    let i_data = [
        (( 0.25f32,  0.25f32, 0.0f32), (1.0f32, 0.0f32)),
        ((-0.25,  0.25, 0.0), (0.0, 0.0)),
        (( 0.25, -0.25, 0.0), (1.0, 1.0)),
        ((-0.25, -0.25, 0.0), (0.0, 1.0)),
    ];

    let i_vertices = gx.buffer_from_data(BuffUse::VERTEX, &i_data[..]);


    // points pipeline
    let p_pipeline = gx.render_pipeline(
        TexOpt::Output, DEPTH_TESTING, ALPHA_BLENDING, MSAA, &vs, &fs, vertex_desc![0 => Float3, 1 => Float2],
        Primitive::PointList, &layout
    );

    let p_data = [
        (( 0.25f32,  0.25f32, 0.0f32), (1.0f32, 0.0f32)),
        ((-0.25,  0.25, 0.0), (0.0, 0.0)),
        (( 0.25, -0.25, 0.0), (1.0, 1.0)),
        ((-0.25, -0.25, 0.0), (0.0, 1.0)),
    ];

    let p_vertices = gx.buffer_from_data(BuffUse::VERTEX, &p_data[..]);


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

                gx.pass_frame_render(
                    Some(Color::GREEN),
                    &[
                        (&t_pipeline, &binding, t_vertices.slice(..), 0..t_data.len() as u32),
                        (&l_pipeline, &binding, l_vertices.slice(..), 0..l_data.len() as u32),
                        (&i_pipeline, &img_binding, i_vertices.slice(..), 0..i_data.len() as u32),
                        (&p_pipeline, &binding, p_vertices.slice(..), 0..p_data.len() as u32),
                    ],
                );


                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}
