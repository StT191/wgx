
use std::{time::{Instant}};
use futures::executor::block_on;
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


    let mut gx = block_on(Wgx::new(Some(&window), Features::empty(), limits!{})).unwrap();
    let mut target = gx.surface_target((width, height), DEPTH_TESTING, MSAA).unwrap();


    // shaders
    let shader = gx.load_wgsl(include_str!("../shaders/flat_texture.wgsl"));


    // pipeline
    let pipeline = target.render_pipeline(
        &gx, ALPHA_BLENDING, (&shader, "vs_main"), (&shader, "fs_main"),
        &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        Primitive::TriangleStrip, None,
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
    let vertices = gx.buffer_from_data(BufUse::VERTEX, &vertex_data[..]);


    // colors
    let color_texture = gx.texture((1, 1), 1, TexUse::TEXTURE_BINDING | TexUse::COPY_DST, TEXTURE);
    gx.write_texture(&color_texture, (0, 0, 1, 1), Color::from([0.5, 0.0, 0.0]).u8());
    let color_texture_view = color_texture.create_default_view();



    // draw target
    let draw_target = TextureTarget::new(&gx, (width, height), false, 1, TexUse::TEXTURE_BINDING, TEXTURE);

    let draw_pipeline = draw_target.render_pipeline(
        &gx, false, (&shader, "vs_main"), (&shader, "fs_main"),
        &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        Primitive::TriangleStrip, None,
    );

    let draw_binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
        bind!(0, TextureView, &color_texture_view),
        bind!(1, Sampler, &sampler),
    ]);

    target.with_encoder_frame(&gx, |encoder, attachment| { // !! ecoder witout draw to attachment produces hang!

        encoder.with_render_pass(&draw_target.attachment().unwrap(), Some(Color::ORANGE), |mut rpass| {
            rpass.set_pipeline(&draw_pipeline);
            rpass.set_bind_group(0, &draw_binding, &[]);
            rpass.set_vertex_buffer(0, vertices.slice(..));
            rpass.draw(0..vertex_data.len() as u32, 0..1);
        });

        encoder.render_pass(attachment, None);

    }).expect("frame error");



    // binding
    let binding = gx.bind(&draw_pipeline.get_bind_group_layout(0), &[
        bind!(0, TextureView, &draw_target.attachment().unwrap().view),
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

                    encoder.with_render_pass(attachment, Some(Color::GREEN), |mut rpass| {
                        rpass.set_pipeline(&pipeline);
                        rpass.set_bind_group(0, &binding, &[]);
                        rpass.set_vertex_buffer(0, vertices.slice(..));
                        rpass.draw(0..vertex_data.len() as u32, 0..1);
                    });

                }).expect("frame error");

                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}