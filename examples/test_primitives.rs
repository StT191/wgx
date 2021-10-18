#![allow(unused)]

// imports
use image;

use std::{time::{Instant}, fs::read};

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};

use wgx::*;
use cgmath::*;


// main
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


    let mut gx = Wgx::new(Some(&window));
    let mut target = gx.surface_target((width, heigh), DEPTH_TESTING, MSAA).expect("render target failed");

    // clear
    // gx.pass_frame_render(Some(Color::GREEN), &[]);


    // global pipeline
    let vs = gx.load_glsl(include_str!("../shaders/pass_texC.vert"), ShaderType::Vertex);
    let fs = gx.load_glsl(include_str!("../shaders/texture_flat.frag"), ShaderType::Fragment);

    // layout
    let layout = gx.binding(&[
        binding!(0, FRAGMENT, SampledTexture),
        binding!(1, FRAGMENT, Sampler)
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
        &gx, ALPHA_BLENDING, &vs, &fs, vertex_desc![0 => Float32x3, 1 => Float32x2],
        Primitive::TriangleStrip, &layout
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
        &gx, ALPHA_BLENDING, &vs, &fs, vertex_desc![0 => Float32x3, 1 => Float32x2],
        Primitive::LineStrip, &layout
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


    // picture pipeline
    let img = image::open("img/logo_red.png")
        .expect("failed loading image")
        .into_rgba8();

    let (w, h) = (img.width(), img.height());

    let image_texture = gx.texture((w, h), 1, TexUse::TEXTURE_BINDING | TexUse::COPY_DST, TEXTURE);

    gx.write_texture(&image_texture, (0, 0, w, h), &img.as_raw().as_slice() );

    let image_texture_view = image_texture.create_default_view();


    // binding
    let img_binding = gx.bind(&layout, &[
        bind!(0, TextureView, &image_texture_view),
        bind!(1, Sampler, &sampler),
    ]);


    let i_pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, &vs, &fs, vertex_desc![0 => Float32x3, 1 => Float32x2],
        Primitive::TriangleStrip, &layout
    );

    let i_data = [
        ([ 0.25,  0.25, 0.0f32], [1.0, 0.0f32]),
        ([-0.25,  0.25, 0.0], [0.0, 0.0]),
        ([ 0.25, -0.25, 0.0], [1.0, 1.0]),
        ([-0.25, -0.25, 0.0], [0.0, 1.0]),
    ];

    let i_vertices = gx.buffer_from_data(BuffUse::VERTEX, &i_data[..]);


    // points pipeline
    let p_pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, &vs, &fs, vertex_desc![0 => Float32x3, 1 => Float32x2],
        Primitive::PointList, &layout
    );

    let p_data = [
        ([ 0.25,  0.25, 0.0f32], [1.0, 0.0f32]),
        ([-0.25,  0.25, 0.0], [0.0, 0.0]),
        ([ 0.25, -0.25, 0.0], [1.0, 1.0]),
        ([-0.25, -0.25, 0.0], [0.0, 1.0]),
    ];

    let p_vertices = gx.buffer_from_data(BuffUse::VERTEX, &p_data[..]);


    // text_render
    let font_data = read("fonts/font_active.ttf").expect("failed loading font");

    let mut glyphs = gx.glyph_brush(OUTPUT, font_data).expect("invalid font");

    let projection = unit_fov_projection(30.0, width as f32 / heigh as f32, sf*1000.0);


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

                glyphs.add_text(
                    vec![Text::new("Hey Ho!\nWhat is going on? Anyway?")
                    .with_scale(sf*50.0).with_color(Color::from([0x2,0x2,0x12]))],
                    None, Some((sf*200.0, f32::INFINITY)), None
                );

                let trf =
                    projection *
                    // Matrix4::from_translation((0.0, 0.0, 0.0).into()) *
                    // Matrix4::from_angle_z(Deg(45.0)) *
                    // Matrix4::from_angle_y(Deg(88.0)) *
                    Matrix4::from_translation((-sf*1200.0, sf*900.0, 0.0).into()) *
                    // Matrix4::from_angle_x(Deg(45.0)) *
                    Matrix4::from_scale(3.0);


                target.with_encoder_frame(&gx, |encoder, attachment| {
                    encoder.draw(attachment,
                        Some(Color::GREEN),
                        &[
                            (&t_pipeline, &binding, t_vertices.slice(..), 0..t_data.len() as u32),
                            (&l_pipeline, &binding, l_vertices.slice(..), 0..l_data.len() as u32),
                            (&i_pipeline, &img_binding, i_vertices.slice(..), 0..i_data.len() as u32),
                            (&p_pipeline, &binding, p_vertices.slice(..), 0..p_data.len() as u32),
                        ]
                    );
                    encoder.draw_glyphs(&gx, attachment, &mut glyphs, trf, None, None);
                });


                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}
