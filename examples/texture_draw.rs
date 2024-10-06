
use platform::winit::{
  window::WindowAttributes, event::{WindowEvent, KeyEvent, ElementState}, keyboard::{PhysicalKey, KeyCode},
  dpi::PhysicalSize,
};
use platform::{*, time::*};
use wgx::{*};


main_app_closure! {
    LogLevel::Warn,
    WindowAttributes::default().with_inner_size(PhysicalSize::new(1000, 1000)),
    init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, &AppEvent) {

    let window = ctx.window_clone();

    let srgb = false;
    let msaa = 1;
    let depth_testing = None;
    let blending = None;

    let (gx, mut target) = Wgx::new_with_target(window.clone(), features!(), limits!{}, window.inner_size(), srgb, msaa, depth_testing).await.unwrap();

    // common/shaders
    let shader = gx.load_wgsl(wgsl_modules::include!("common/shaders/shader_flat_text.wgsl"));

    // pipeline
    let pipeline = target.render_pipeline(&gx,
        None, &[vertex_dsc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", Primitive { topology: Topology::TriangleStrip, ..Primitive::default() }),
        (&shader, "fs_main", blending),
    );

    // sampler
    let sampler = gx.default_sampler();

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Vtx([f32;3], [f32;2]);
    unsafe impl wgx::ReadBytes for Vtx {}

    // vertices
    let vertex_data = [
        Vtx([ 0.5,  0.5, 0.0f32], [1.0, 0.0f32]),
        Vtx([-0.5,  0.5, 0.0], [0.0, 0.0]),
        Vtx([ 0.5, -0.5, 0.0], [1.0, 1.0]),
        Vtx([-0.5, -0.5, 0.0], [0.0, 1.0]),
    ];
    let vertices = gx.buffer_from_data(BufUse::VERTEX, &vertex_data[..]);


    // colors
    let color_texture = TextureLot::new_2d_with_data(&gx,
        [1, 1, 1], 1, DEFAULT_SRGB, None, TexUse::TEXTURE_BINDING,
        Color::ORANGE.srgb().u8(),
    );

    let bg_color_draw_target = Color::ORANGE;
    let bg_color_target = Color::ORANGE;


    // draw target
    const DRAW_MSAA:u32 = 1;

    let draw_target = TextureTarget::new(&gx, window.inner_size(), DRAW_MSAA, None, DEFAULT_SRGB, None, TexUse::TEXTURE_BINDING);
    // let draw_target2 = TextureTarget::new(&gx, window.inner_size(), DRAW_MSAA, None, DEFAULT_SRGB, None, TexUse::TEXTURE_BINDING);

    let draw_pipeline = gx.render_pipeline(
        DRAW_MSAA, None, None,
        &[vertex_dsc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", Primitive { topology: Topology::TriangleStrip, ..Primitive::default() }),
        Some((&shader, "fs_main", &[
            (draw_target.view_format(), blending),
            // (draw_target2.view_format(), BLENDING),
        ])),
    );

    let draw_binding = gx.bind(&draw_pipeline.get_bind_group_layout(0), &[
        bind!(0, TextureView, &color_texture.view),
        bind!(1, Sampler, &sampler),
    ]);

    target.with_frame(None, |frame| gx.with_encoder(|encoder| {

        encoder.with_render_pass(
            (
                [
                    Some(draw_target.color_attachment(Some(bg_color_draw_target))),
                    // Some(draw_target2.color_attachment(Some(bg_color_draw_target))),
                ],
                None
            ),
            |rpass| {
                rpass.set_pipeline(&draw_pipeline);
                rpass.set_bind_group(0, &draw_binding, &[]);
                rpass.set_vertex_buffer(0, vertices.slice(..));
                rpass.draw(0..vertex_data.len() as u32, 0..1);
            }
        );

        // !! ecoder witout draw to attachment produces hang!
        encoder.render_pass(frame.attachments(Some(bg_color_target), None, None));

    })).expect("frame error");


    // binding
    let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
        bind!(0, TextureView, &draw_target.view),
        bind!(1, Sampler, &sampler),
    ]);

    // event loop

    move |_ctx: &mut AppCtx, event: &AppEvent| match event {

        AppEvent::WindowEvent(WindowEvent::Resized(size)) => {
            target.update(&gx, *size);
        },

        AppEvent::WindowEvent(WindowEvent::KeyboardInput { event: KeyEvent {
            state: ElementState::Pressed, physical_key: PhysicalKey::Code(KeyCode::KeyR), ..
        }, ..}) => {
            window.request_redraw();
        },

        AppEvent::WindowEvent(WindowEvent::RedrawRequested) => {

            let then = Instant::now();

            target.with_frame(None, |frame| gx.with_encoder(|encoder| {
                encoder.with_render_pass(
                    frame.attachments(Some(bg_color_target), None, None),
                    |rpass| {
                        rpass.set_pipeline(&pipeline);
                        rpass.set_bind_group(0, &binding, &[]);
                        rpass.set_vertex_buffer(0, vertices.slice(..));
                        rpass.draw(0..vertex_data.len() as u32, 0..1);
                    }
                );
            })).expect("frame error");

            println!("{:?}", then.elapsed());
        },

        _ => {}
    }
}