
use std::{time::{Instant}};
use pollster::FutureExt;
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};
use wgx::{*};


fn main() {

    const DEPTH_TESTING:bool = true;
    const MSAA:u32 = 4;
    const BLENDING:Option<Blend> = Some(Blend::ALPHA_BLENDING);


    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((600, 600)));
    window.set_title("WgFx");


    let (gx, surface) = unsafe {Wgx::new(Some(&window), Features::empty(), limits!{})}.block_on().unwrap();
    let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (600, 600), MSAA, DEPTH_TESTING).unwrap();


    // global pipeline
    let shader = gx.load_wgsl(include_wgsl_module!("common/shaders/shader_flat_text.wgsl"));

    let pipeline = target.render_pipeline(&gx,
        None, &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", Primitive::default()),
        (&shader, "fs_main", BLENDING),
    );

    // colors
    let texture = TextureLot::new_2d_with_data(&gx,
        (2, 1), 1, TEXTURE, /*TexUse::COPY_SRC |*/ TexUse::TEXTURE_BINDING,
        [[255u8, 0, 0, 255], [0, 0, 255, 50]]
    );

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Vtx([f32;3], [f32;2]);
    unsafe impl wgx::ReadBytes for Vtx {}

    // vertices
    let data = [
        Vtx([-0.25, -0.5, 0.35f32], [0.0, 0.0f32]),
        Vtx([0.0, -0.5, 0.35], [1.0, 0.0]),
        Vtx([-1.0, 0.5, 0.1], [0.0, 0.0]),

        Vtx([0.25, -0.5, 0.1], [0.0, 0.0]),
        Vtx([0.5, -0.5, 0.1], [1.0, 0.0]),
        Vtx([-1.0, 0.5, 0.6], [0.0, 0.0]),

        Vtx([-0.75, -0.5, 0.1], [0.0, 0.0]),
        Vtx([-1.0, -0.5, 0.1], [1.0, 0.0]),
        Vtx([-0.3, 0.5, 0.312], [1.0, 0.0]),
    ];
    let vertices = gx.buffer_from_data(BufUse::VERTEX, &data[..]);


    // texture + sampler
    let sampler = gx.default_sampler();

    let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
        bind!(0, TextureView, &texture.view),
        bind!(1, Sampler, &sampler),
    ]);


    // render bundles
    let bundles = [target.render_bundle(&gx, |rpass| {
        rpass.set_pipeline(&pipeline);
        rpass.set_bind_group(0, &binding, &[]);
        rpass.set_vertex_buffer(0, vertices.slice(..));
        rpass.draw(0..data.len() as u32, 0..1);
    })];

    // event loop

    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                *control_flow = ControlFlow::Exit;
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                let size = (size.width, size.height);
                target.update(&gx, size);
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
                    encoder.with_render_pass(
                        frame.attachments(Some(Color::GREEN), Some(1.0)),
                        |mut rpass| {
                            rpass.execute_bundles(&bundles);
                        }
                    );
                }).expect("frame error");

                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}