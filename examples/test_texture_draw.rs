
use std::{time::{Instant}};

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};

use wgx::*;


fn main() {

    const DEPTH_TESTING:bool = false;
    const ALPHA_BLENDING:bool = false;
    const MSAA:u32 = 1;


    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();

    // size
    let sf = window.scale_factor() as f32;

    let width = (sf * 800.0) as u32;
    let height = (sf * 600.0) as u32;

    window.set_inner_size(PhysicalSize::<u32>::from((width, height)));
    window.set_title("WgFx");


    let mut gx = Wgx::new(Some(&window));
    let mut target = gx.surface_target((width, height), DEPTH_TESTING, MSAA).expect("render target failed");


    // shaders
    let shader = gx.load_wgsl(include_str!("../shaders/flat_texture.wgsl"));


    // layout
    let layout = gx.binding(&[
        binding!(0, FRAGMENT, SampledTexture),
        binding!(1, FRAGMENT, Sampler)
    ]);


    // pipeline
    let pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, (&shader, "vs_main"), (&shader, "fs_main"),
        vertex_desc![0 => Float32x3, 1 => Float32x2],
        Primitive::TriangleStrip, &layout
    );


    // sampler
    let sampler = gx.sampler();


    // vertices
    let vertex_data = [
        ([ 0.5,  0.5, 0.0f32], [1.0, 0.0f32]),
        ([-0.5,  0.5, 0.0], [0.0, 0.0]),
        ([ 0.5, -0.5, 0.0], [1.0, 1.0]),
        ([-0.5, -0.5, 0.0], [0.0, 1.0]),
    ];
    let vertices = gx.buffer_from_data(BuffUse::VERTEX, &vertex_data[..]);


    // colors
    let color_texture = gx.texture((1, 1), 1, TexUse::TEXTURE_BINDING | TexUse::COPY_DST, TEXTURE);
    gx.write_texture(&color_texture, (0, 0, 1, 1), &[Color::RED.u8()]);
    let color_texture_view = color_texture.create_default_view();



    // draw target
    let draw_target = TextureTarget::new(&gx, (width, height), false, 1, TexUse::TEXTURE_BINDING, TEXTURE);

    let draw_pipeline = draw_target.render_pipeline(
        &gx, false, (&shader, "vs_main"), (&shader, "fs_main"),
        vertex_desc![0 => Float32x3, 1 => Float32x2],
        Primitive::TriangleStrip, &layout
    );

    let draw_binding = gx.bind(&layout, &[
        bind!(0, TextureView, &color_texture_view),
        bind!(1, Sampler, &sampler),
    ]);

    target.with_encoder_frame(&gx, |encoder, attachment| { // !! ecoder witout draw to attachment produces hang!
        encoder.draw(&draw_target.attachment(), Some(Color::YELLOW),
            &[(&draw_pipeline, &draw_binding, vertices.slice(..), 0..vertex_data.len() as u32)]
        );
        encoder.draw(attachment, None, &[]);
    }).expect("frame error");



    // binding
    let binding = gx.bind(&layout, &[
        bind!(0, TextureView, &draw_target.attachment().view),
        bind!(1, Sampler, &sampler),
    ]);


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

                    encoder.draw(attachment, Some(Color::GREEN), &[
                        (&pipeline, &binding, vertices.slice(..), 0..vertex_data.len() as u32),
                    ]);

                }).expect("frame error");

                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}
