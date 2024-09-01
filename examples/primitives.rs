
use platform::winit::{
  window::WindowBuilder, event::{WindowEvent, KeyEvent, ElementState}, keyboard::{PhysicalKey, KeyCode},
  dpi::PhysicalSize,
};
use platform::{*, time::*};
use wgx::{*};


main_app_closure! {
    LogLevel::Warn,
    WindowBuilder::new().with_inner_size(PhysicalSize {width: 1000, height: 1000}),
    init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, &AppEvent) {

    let window = ctx.window_clone();

    let msaa = 4;
    let depth_testing = None;
    let blending = Some(Blend::ALPHA_BLENDING);

    let (gx, mut target) = Wgx::new_with_target(window.clone(), features!(), limits!{}, window.inner_size(), msaa, depth_testing).await.unwrap();

    // global pipeline
    let shader = gx.load_wgsl(wgsl_modules::include!("common/shaders/shader_flat_text.wgsl"));

    // layout
    let layout = gx.layout(&[
        binding!(0, Stage::FRAGMENT, Texture, D2),
        binding!(1, Stage::FRAGMENT, Sampler)
    ]);


    // colors
    let color_texture = TextureLot::new_2d_with_data(&gx,
        [3, 1, 1], 1, DEFAULT_SRGB, None, TexUse::TEXTURE_BINDING,
        [[255u8, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]]
    );

    let sampler = gx.default_sampler();

    // binding
    let binding = gx.bind(&layout, &[
        bind!(0, TextureView, &color_texture.view),
        bind!(1, Sampler, &sampler),
    ]);


    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Vtx([f32;3], [f32;2]);
    unsafe impl wgx::ReadBytes for Vtx {}


    // triangle pipeline
    let t_pipeline = target.render_pipeline(&gx,
        Some((&[], &[&layout])), &[vertex_dsc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", Primitive { topology: Topology::TriangleStrip, ..Primitive::default() }),
        (&shader, "fs_main", blending),
    );

    let t_data = [
        Vtx([ 0.5,  0.5, 0.0f32], [0.0, 0.0f32]),
        Vtx([-0.5,  0.5, 0.0], [0.0, 0.0]),
        Vtx([ 0.5, -0.5, 0.0], [0.0, 0.0]),
        Vtx([-0.5, -0.5, 0.0], [0.0, 0.0]),
    ];

    let t_vertices = gx.buffer_from_data(BufUse::VERTEX, &t_data[..]);


    // lines pipeline
    let l_pipeline = target.render_pipeline(&gx,
        Some((&[], &[&layout])), &[vertex_dsc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", Primitive { topology: Topology::LineStrip, ..Primitive::default() }),
        (&shader, "fs_main", blending),
    );

    let l_data = [
        Vtx([ 0.5,  0.5, 0.0f32], [1.0, 0.0f32]),
        Vtx([-0.5,  0.5, 0.0], [1.0, 0.0]),
        Vtx([-0.5, -0.5, 0.0], [1.0, 0.0]),
        Vtx([ 0.5, -0.5, 0.0], [1.0, 0.0]),
        Vtx([ 0.5,  0.5, 0.0], [1.0, 0.0]),
        Vtx([ -1.0, -1.0, 0.0], [1.0, 0.0]),
    ];

    let l_vertices = gx.buffer_from_data(BufUse::VERTEX, &l_data[..]);


    // points pipeline
    let p_pipeline = target.render_pipeline(&gx,
        Some((&[], &[&layout])), &[vertex_dsc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", Primitive { topology: Topology::PointList, ..Primitive::default() }),
        (&shader, "fs_main", blending),
    );

    let p_data = [
        Vtx([ 0.25,  0.25, 0.0f32], [1.0, 0.0f32]),
        Vtx([-0.25,  0.25, 0.0], [0.5, 0.0]),
        Vtx([ 0.25, -0.25, 0.0], [1.0, 0.0]),
        Vtx([-0.25, -0.25, 0.0], [0.5, 0.0]),
    ];

    let p_vertices = gx.buffer_from_data(BufUse::VERTEX, &p_data[..]);


    // picture pipeline
    let img = image::load_from_memory(include_bytes!("common/img/logo_red.png"))
        .expect("failed loading image")
        .into_rgba8()
    ;

    let (w, h) = (img.width(), img.height());
    let image_texture = TextureLot::new_2d_with_data(&gx, [w, h, 1], 1, DEFAULT_SRGB, None, TexUse::TEXTURE_BINDING, &img.as_raw()[..]);

    // binding
    let img_binding = gx.bind(&layout, &[
        bind!(0, TextureView, &image_texture.view),
        bind!(1, Sampler, &sampler),
    ]);


    let i_pipeline = target.render_pipeline(&gx,
        Some((&[], &[&layout])), &[vertex_dsc!(Vertex, 0 => Float32x3, 1 => Float32x2)],
        (&shader, "vs_main", Primitive { topology: Topology::TriangleStrip, ..Primitive::default() }),
        (&shader, "fs_main", blending),
    );

    let i_data = [
        Vtx([ 0.25,  0.25, 0.0f32], [1.0, 0.0f32]),
        Vtx([-0.25,  0.25, 0.0], [0.0, 0.0]),
        Vtx([ 0.25, -0.25, 0.0], [1.0, 1.0]),
        Vtx([-0.25, -0.25, 0.0], [0.0, 1.0]),
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
                encoder.pass_bundles(frame.attachments(Some(Color::GREEN), None, None), &bundles);
            })).expect("frame error");

            println!("{:?}", then.elapsed());
        },

        _ => {}
    }
}