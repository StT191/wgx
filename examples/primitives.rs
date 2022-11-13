
use std::{time::{Instant}};
use pollster::FutureExt;
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};
use wgx::*;


fn main() {

    const DEPTH_TESTING:bool = false;
    const MSAA:u32 = 4;
    const ALPHA_BLENDING:Option<BlendState> = Some(BlendState::ALPHA_BLENDING);


    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();

    // size
    let sf = window.scale_factor() as f32;

    let width = (sf * 800.0) as u32;
    let heigh = (sf * 600.0) as u32;

    window.set_inner_size(PhysicalSize::<u32>::from((width, heigh)));
    window.set_title("WgFx");


    let (gx, surface) = Wgx::new(Some(&window), Features::empty(), limits!{}).block_on().unwrap();
    let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (width, heigh), MSAA, DEPTH_TESTING).unwrap();


    // global pipeline
    let shader = gx.load_wgsl(include_wgsl_module!("./shaders/flat_text.wgsl"));

    // layout
    let layout = gx.layout(&[
        binding!(0, Shader::FRAGMENT, SampledTexture2D),
        binding!(1, Shader::FRAGMENT, Sampler)
    ]);


    // colors
    let color_texture = TextureLot::new_2d_with_data(&gx,
        (3, 1), 1, TEXTURE, TexUse::TEXTURE_BINDING,
        [[255u8, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]]
    );

    let sampler = gx.default_sampler();

    // binding
    let binding = gx.bind(&layout, &[
        bind!(0, TextureView, &color_texture.view),
        bind!(1, Sampler, &sampler),
    ]);


    // triangle pipeline
    let t_pipeline = target.render_pipeline(&gx,
        Some((&[], &[&layout])), &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", Primitive::TriangleStrip),
        (&shader, "fs_main", ALPHA_BLENDING),
    );

    let t_data = [
        ([ 0.5,  0.5, 0.0f32], [0.0, 0.0f32]),
        ([-0.5,  0.5, 0.0], [0.0, 0.0]),
        ([ 0.5, -0.5, 0.0], [0.0, 0.0]),
        ([-0.5, -0.5, 0.0], [0.0, 0.0]),
    ];

    let t_vertices = gx.buffer_from_data(BufUse::VERTEX, &t_data[..]);


    // lines pipeline
    let l_pipeline = target.render_pipeline(&gx,
        Some((&[], &[&layout])), &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", Primitive::LineStrip),
        (&shader, "fs_main", ALPHA_BLENDING),
    );

    let l_data = [
        ([ 0.5,  0.5, 0.0f32], [1.0, 0.0f32]),
        ([-0.5,  0.5, 0.0], [1.0, 0.0]),
        ([-0.5, -0.5, 0.0], [1.0, 0.0]),
        ([ 0.5, -0.5, 0.0], [1.0, 0.0]),
        ([ 0.5,  0.5, 0.0], [1.0, 0.0]),
        ([ -1.0, -1.0, 0.0], [1.0, 0.0]),
    ];

    let l_vertices = gx.buffer_from_data(BufUse::VERTEX, &l_data[..]);


    // points pipeline
    let p_pipeline = target.render_pipeline(&gx,
        Some((&[], &[&layout])), &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", Primitive::PointList),
        (&shader, "fs_main", ALPHA_BLENDING),
    );

    let p_data = [
        ([ 0.25,  0.25, 0.0f32], [1.0, 0.0f32]),
        ([-0.25,  0.25, 0.0], [0.5, 0.0]),
        ([ 0.25, -0.25, 0.0], [1.0, 0.0]),
        ([-0.25, -0.25, 0.0], [0.5, 0.0]),
    ];

    let p_vertices = gx.buffer_from_data(BufUse::VERTEX, &p_data[..]);


    // picture pipeline
    let decoder = png::Decoder::new(&include_bytes!("img/logo_red.png")[..]);
    let mut reader = decoder.read_info().expect("failed decoding image");

    let mut img_data = vec![0; reader.output_buffer_size()];


    let info = reader.next_frame(&mut img_data).expect("failed reading image");

    /*let img = image::load_from_memory(include_bytes!("img/logo_red.png"))
        .expect("failed loading image")
        .into_rgba8();*/

    let image_texture = TextureLot::new_2d_with_data(&gx, (info.width, info.height), 1, TEXTURE, TexUse::TEXTURE_BINDING, img_data);

    // binding
    let img_binding = gx.bind(&layout, &[
        bind!(0, TextureView, &image_texture.view),
        bind!(1, Sampler, &sampler),
    ]);


    let i_pipeline = target.render_pipeline(&gx,
        Some((&[], &[&layout])), &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", Primitive::TriangleStrip),
        (&shader, "fs_main", ALPHA_BLENDING),
    );

    let i_data = [
        ([ 0.25,  0.25, 0.0f32], [1.0, 0.0f32]),
        ([-0.25,  0.25, 0.0], [0.0, 0.0]),
        ([ 0.25, -0.25, 0.0], [1.0, 1.0]),
        ([-0.25, -0.25, 0.0], [0.0, 1.0]),
    ];

    let i_vertices = gx.buffer_from_data(BufUse::VERTEX, &i_data[..]);


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

                target.with_encoder_frame(&gx, |encoder, frame| {
                    encoder.render_bundles(frame.attachments(Some(Color::GREEN), None), &bundles);
                }).expect("frame error");

                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}