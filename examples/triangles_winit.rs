
use std::sync::Arc;
use platform::winit::{
    event_loop::{EventLoop, ControlFlow}, dpi::PhysicalSize,
    window::WindowAttributes, event::{Event, WindowEvent, KeyEvent, ElementState, StartCause},
    keyboard::{PhysicalKey, KeyCode},
};
use platform::{timer::*, pollster::FutureExt};
use wgx::{*};


fn main() {

    let event_loop = EventLoop::new().unwrap();
    let attrs = WindowAttributes::default().with_inner_size(PhysicalSize::new(1000, 1000));
    #[allow(deprecated)]
    let window = Arc::new(event_loop.create_window(attrs).unwrap());


    let srgb = true;
    let msaa = 4;
    let depth_testing = Some(TexFmt::Depth32Float);
    let blending = Some(Blend::ALPHA_BLENDING);

    let (gx, mut target) = Wgx::new_with_target(window.clone(), features!(), limits!{}, window.inner_size(), srgb, msaa, depth_testing).block_on().unwrap();

    // global pipeline
    let shader = gx.load_wgsl(wgsl_modules::include!("common/shaders/shader_flat_text.wgsl"));

    let pipeline = target.render_pipeline(&gx,
        None, &[vertex_dsc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", None, Primitive::default()),
        (&shader, "fs_main", None, blending),
    );

    // colors
    let texture = TextureLot::new_2d_with_data(&gx,
        [2, 1, 1], 1, TexFmt::Rgba8UnormSrgb, None, /*TexUse::COPY_SRC |*/ TexUse::TEXTURE_BINDING,
        [[255u8, 0, 0, 255], [0, 0, 255, 50]]
    );

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Vtx([f32;3], [f32;2]);
    unsafe impl wgx::ReadBytes for Vtx {}

    // vertices
    let data = [
        Vtx([-0.25, -0.5, 0.35], [0.0, 0.0]),
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
    let sampler = gx.std_sampler();

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
    let mut frame_timer = StepInterval::from_secs(1.0 / 60.0);

    #[allow(deprecated)]
    event_loop.run(move |event, event_target| {

        event_target.set_control_flow(ControlFlow::WaitUntil(frame_timer.next));

        match event {
            Event::NewEvents(StartCause::ResumeTimeReached {..}) => {
                window.request_redraw(); // request frame
                frame_timer.step_by(1);
                event_target.set_control_flow(ControlFlow::Wait);
            },

            Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                event_target.exit();
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                target.update(&gx, [size.width, size.height]);
            },

            Event::WindowEvent { event: WindowEvent::KeyboardInput { event: KeyEvent {
                state: ElementState::Pressed, physical_key: PhysicalKey::Code(KeyCode::KeyR), ..
            }, ..}, ..} => {
                window.request_redraw();
            },

            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {

                // let then = Instant::now();

                target.with_frame(None, |frame| gx.with_encoder(|encoder| {
                    encoder.pass_bundles(frame.attachments(Some(Color::GREEN), Some(1.0), None), &bundles);
                })).expect("frame error");

                // println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    }).unwrap();
}
