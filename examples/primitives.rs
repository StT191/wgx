#![allow(unused)]

use std::{time::{Instant}};
use futures::executor::block_on;
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};
use image;
use wgx::*;


fn main() {

    const DEPTH_TESTING:bool = false;
    const ALPHA_BLENDING:bool = true;
    const MSAA:u32 = 4;


    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();

    // size
    let sf = window.scale_factor() as f32;

    let width = (sf * 800.0) as u32;
    let heigh = (sf * 600.0) as u32;

    window.set_inner_size(PhysicalSize::<u32>::from((width, heigh)));
    window.set_title("WgFx");


    let mut gx = block_on(Wgx::new(Some(&window), Features::empty(), limits!{})).unwrap();
    let mut target = gx.surface_target((width, heigh), DEPTH_TESTING, MSAA).unwrap();


    // global pipeline
    let shader = gx.load_wgsl(include_str!("../shaders/flat_texture.wgsl"));

    // layout
    let layout = gx.layout(&[
        binding!(0, Shader::FRAGMENT, SampledTexture),
        binding!(1, Shader::FRAGMENT, Sampler)
    ]);


    // colors
    let color_texture = gx.texture((3, 1), 1, TexUse::TEXTURE_BINDING | TexUse::COPY_DST, TEXTURE);
    gx.write_texture(&color_texture, (0, 0, 3, 1), &[
        [255u8, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255],
    ]);
    let color_texture_view = color_texture.create_default_view();

    let sampler = gx.sampler();

    // binding
    let binding = gx.bind(&layout, &[
        bind!(0, TextureView, &color_texture_view),
        bind!(1, Sampler, &sampler),
    ]);


    // triangle pipeline
    let t_pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, (&shader, "vs_main"), (&shader, "fs_main"),
        &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        Primitive::TriangleStrip, Some((&[], &[&layout]))
    );

    let t_data = [
        ([ 0.5,  0.5, 0.0f32], [0.0, 0.0f32]),
        ([-0.5,  0.5, 0.0], [0.0, 0.0]),
        ([ 0.5, -0.5, 0.0], [0.0, 0.0]),
        ([-0.5, -0.5, 0.0], [0.0, 0.0]),
    ];

    let t_vertices = gx.buffer_from_data(BuffUse::VERTEX, &t_data[..]);


    // lines pipeline
    let l_pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, (&shader, "vs_main"), (&shader, "fs_main"),
        &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        Primitive::LineStrip, Some((&[], &[&layout]))
    );

    let l_data = [
        ([ 0.5,  0.5, 0.0f32], [1.0, 0.0f32]),
        ([-0.5,  0.5, 0.0], [1.0, 0.0]),
        ([-0.5, -0.5, 0.0], [1.0, 0.0]),
        ([ 0.5, -0.5, 0.0], [1.0, 0.0]),
        ([ 0.5,  0.5, 0.0], [1.0, 0.0]),
        ([ -1.0, -1.0, 0.0], [1.0, 0.0]),
    ];

    let l_vertices = gx.buffer_from_data(BuffUse::VERTEX, &l_data[..]);


    // points pipeline
    let p_pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, (&shader, "vs_main"), (&shader, "fs_main"),
        &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        Primitive::PointList, Some((&[], &[&layout]))
    );

    let p_data = [
        ([ 0.25,  0.25, 0.0f32], [1.0, 0.0f32]),
        ([-0.25,  0.25, 0.0], [0.5, 0.0]),
        ([ 0.25, -0.25, 0.0], [1.0, 0.0]),
        ([-0.25, -0.25, 0.0], [0.5, 0.0]),
    ];

    let p_vertices = gx.buffer_from_data(BuffUse::VERTEX, &p_data[..]);


    // picture pipeline
    let img = image::open("img/logo_red.png")
        .expect("failed loading image")
        .into_rgba8();

    let (w, h) = (img.width(), img.height());

    let image_texture = gx.texture((w, h), 1, TexUse::TEXTURE_BINDING | TexUse::COPY_DST, TEXTURE);

    gx.write_texture(&image_texture, (0, 0, w, h), &img.as_raw().as_slice());

    let image_texture_view = image_texture.create_default_view();


    // binding
    let img_binding = gx.bind(&layout, &[
        bind!(0, TextureView, &image_texture_view),
        bind!(1, Sampler, &sampler),
    ]);


    let i_pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, (&shader, "vs_main"), (&shader, "fs_main"),
        &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        Primitive::TriangleStrip, Some((&[], &[&layout]))
    );

    let i_data = [
        ([ 0.25,  0.25, 0.0f32], [1.0, 0.0f32]),
        ([-0.25,  0.25, 0.0], [0.0, 0.0]),
        ([ 0.25, -0.25, 0.0], [1.0, 1.0]),
        ([-0.25, -0.25, 0.0], [0.0, 1.0]),
    ];

    let i_vertices = gx.buffer_from_data(BuffUse::VERTEX, &i_data[..]);


    // render bundles
    let bundles = [target.render_bundle(&gx, |rpass| {

        rpass.set_bind_group(0, &binding, &[]);

        rpass.set_pipeline(&t_pipeline);
        rpass.set_vertex_buffer(0, t_vertices.slice(..));
        rpass.draw(0..t_data.len() as u32, 0..1);

        rpass.set_pipeline(&l_pipeline);
        rpass.set_vertex_buffer(0, l_vertices.slice(..));
        rpass.draw(0..l_data.len() as u32, 0..1);


        rpass.set_bind_group(0, &img_binding, &[]);

        rpass.set_pipeline(&i_pipeline);
        rpass.set_vertex_buffer(0, i_vertices.slice(..));
        rpass.draw(0..i_data.len() as u32, 0..1);


        rpass.set_bind_group(0, &binding, &[]);

        rpass.set_pipeline(&p_pipeline);
        rpass.set_vertex_buffer(0, p_vertices.slice(..));
        rpass.draw(0..p_data.len() as u32, 0..1);
    })];


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
                    encoder.render_bundles(attachment, Some(Color::GREEN), &bundles);
                }).expect("frame error");

                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}