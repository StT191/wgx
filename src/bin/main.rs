#![allow(unused)]

// imports
use std::{time::{Duration, Instant}, fs::read_to_string};

use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};

use wgfx::*;


// main
fn main() {

    const DEPTH_TESTING:bool = true;
    const ALPHA_BLENDING:bool = true;
    const MSAA:u32 = 8;

    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();
    window.set_title("WgFx");

    let mut gx = Gx::new(&window, DEPTH_TESTING, MSAA);


    // global params
    let layout = gx.binding(&[
        binding!(0, FRAGMENT, SampledTexture),
        binding!(1, FRAGMENT, Sampler)
    ]);

    let vs = gx.load_glsl(&read_to_string("shaders/main.vert").unwrap(), ShaderType::Vertex);
    let fs = gx.load_glsl(&read_to_string("shaders/main.frag").unwrap(), ShaderType::Fragment);

    let vertex_desc = vertex_desc![0 => Float3, 1 => Float2];


    // first render

    let pipeline = gx.render_pipeline(
        false, DEPTH_TESTING, ALPHA_BLENDING, MSAA, &vs, &fs,
        vertex_desc, PrimitiveTopology::TriangleList,
        &layout
    );


    let texture = gx.texture(2, 1, 1, 1, TextureUsage::COPY_DST | TextureUsage::SAMPLED, TexOpt::Texture);

    gx.with_encoder(|mut encoder, gx| {
        let buff = gx.buffer_from_data::<(u8, u8, u8, u8)>(BufferUsage::COPY_SRC, &[
            (255, 0, 0, 230), (0, 0, 255, 230)
        ]);
        buffer_to_texture(encoder, &buff, (2, 1, 0), &texture, (0.0, 0.0, 0, 2, 1));
    });

    const N:usize = 9;

    // dings
    let data:[((f32, f32, f32), (f32, f32)); N] = [
        ((-0.25, -0.5, 0.2), (0.0, 1.0)),
        ((-0.5, -0.5, 0.2), (0.0, 1.0)),
        ((-0.5, 0.5, 0.2), (0.0, 1.0)),

        ((0.25, -0.5, 0.35), (0.0, 1.0)),
        ((0.5, -0.5, 0.35), (1.0, 1.0)),
        ((-1.0, 0.5, 0.1), (1.0, 1.0)),

        ((-0.75, -0.5, 0.1), (0.0, 1.0)),
        ((-1.0, -0.5, 0.1), (1.0, 1.0)),
        ((-0.3, 0.5, 0.1), (1.0, 1.0)),
    ];

    let vertices = gx.buffer_from_data(BufferUsage::VERTEX, &data[0..N]);

    let texture_view = texture.create_default_view();
    let sampler = gx.sampler();

    let bound = gx.bind(&layout, &[
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
                gx.update(size.width, size.height);
            },

            Event::RedrawRequested(_) => {
                gx.with_encoder_frame(|mut encoder, frame, deph_view, msaa| {
                    pass_render(encoder, &frame.view, deph_view, msaa,
                        wgpu::Color::GREEN,
                        &[(&pipeline, &vertices, 0..N as u32, &bound)],
                    );
                });
            },

            _ => {}
        }
    });
}
