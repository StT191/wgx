
use platform::winit::{
  window::WindowAttributes, event::{WindowEvent, KeyEvent, ElementState}, keyboard::{PhysicalKey, KeyCode},
  dpi::PhysicalSize,
};
use platform::{*, time::*};
use wgx::{*, math::*};


main_app_closure! {
    LogLevel::Warn,
    WindowAttributes::default().with_inner_size(PhysicalSize::new(1000, 1000)),
    init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, Event) + use<> {

    let window = ctx.window_clone();

    let srgb = false;
    let msaa = 1;
    let depth_testing = None;
    let blending = None;

    let (gx, mut target) = Wgx::new_with_target(window.clone(), features!(ADDRESS_MODE_CLAMP_TO_BORDER), limits!{}, window.inner_size(), srgb, msaa, depth_testing).await.unwrap();

    // common/shaders
    let shader = gx.load_wgsl(wgsl_modules::include!("common/shaders/shader_flat_text.wgsl"));

    // pipeline
    let pipeline = RenderPipelineConfig::new(
            &[vertex_dsc!(Vertex, 0 => Float32x3), vertex_dsc!(Vertex, 1 => Float32x2)],
            &shader, "vs_main", Primitive { topology: Topology::TriangleStrip, ..Primitive::default() },
        )
        .fragment(&shader, "fs_main")
        .render_target::<1>(&target, blending, Default::default())
        .pipeline(&gx)
    ;

    // sampler
    let sampler = gx.sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        ..std_sampler_descriptor()
    });

    // colors
    let color_texture = TextureLot::new_2d_with_data(&gx,
        [2, 2, 1], 1, TexFmt::Rgba8UnormSrgb, None, TexUse::TEXTURE_BINDING,
        &[
            Color::GREEN.srgb().u8(), Color::ORANGE.srgb().u8(),
            Color::RED.srgb().u8(), Color::BLUE.srgb().u8(),
        ],
    );

    // pixel-position in normalized u/v-space ...
    let p_dim = Vec2::from(color_texture.size().map(|v| 1.0 / v as f32));
    let pp = |[x, y]:[u32;2]| p_dim * (vec2(x as f32, y as f32) + 0.5);

    // vertices
    let vertex_data = [
        vec3(-0.5, -0.5, 0.0),
        vec3( 0.5, -0.5, 0.0),
        vec3(-0.5,  0.5, 0.0),
        vec3( 0.5,  0.5, 0.0),
    ];
    let vertices = gx.buffer_from_data(BufUse::VERTEX, &vertex_data[..]);

    let tex_coords = [[0, 1], [1, 1], [0, 0], [1, 0]];
    let draw_tex_coords = gx.buffer_from_data(BufUse::VERTEX, &tex_coords.map(pp));

    let bg_color_draw_target = Color::ORANGE;
    let bg_color_target = Color::ORANGE;

    // draw target
    const DRAW_MSAA:u32 = 1;

    let draw_target = TextureTarget::new(&gx, window.inner_size(), DRAW_MSAA, None, TexFmt::Rgba8UnormSrgb, None, TexUse::TEXTURE_BINDING);
    // let draw_target2 = TextureTarget::new(&gx, window.inner_size(), DRAW_MSAA, None, TexFmt::Rgba8UnormSrgb, None, TexUse::TEXTURE_BINDING);

    let draw_pipeline = RenderPipelineConfig::new(
            &[vertex_dsc!(Vertex, 0 => Float32x3), vertex_dsc!(Vertex, 1 => Float32x2)],
            &shader, "vs_main", Primitive { topology: Topology::TriangleStrip, ..Primitive::default() },
        )
        .fragment(&shader, "fs_main")
        .target::<1>((draw_target.format(), blending).target())
        // .target::<2>((draw_target2.format(), blending).target())
        .msaa(DRAW_MSAA)
        .pipeline(&gx)
    ;

    let draw_binding = gx.bind(&draw_pipeline.get_bind_group_layout(0), &[
        bind!(0, TextureView, &color_texture.view),
        bind!(1, Sampler, &sampler),
    ]);

    gx.with_encoder(|encoder| {
        encoder.with_render_pass(
            ([
                Some(draw_target.color_attachment(Some(bg_color_draw_target)).into()),
                // Some(draw_target2.color_attachment(Some(bg_color_draw_target)).into()),
            ], None),
            |rpass| {
                rpass.set_pipeline(&draw_pipeline);
                rpass.set_bind_group(0, &draw_binding, &[]);
                rpass.set_vertex_buffer(0, vertices.slice(..));
                rpass.set_vertex_buffer(1, draw_tex_coords.slice(..));
                rpass.draw(0..vertex_data.len() as u32, 0..1);
            }
        );
    });

    let display_tex_coords = gx.buffer_from_data(
        BufUse::VERTEX,
        &tex_coords.map(|[x, y]| [x as f32, y as f32]),
    );

    // binding
    let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
        bind!(0, TextureView, &draw_target.view),
        bind!(1, Sampler, &sampler),
    ]);

    // event loop

    move |_ctx, event| match event {

        Event::WindowEvent(WindowEvent::Resized(size)) => {
            target.update(&gx, size);
        },

        Event::WindowEvent(WindowEvent::KeyboardInput { event: KeyEvent {
            state: ElementState::Pressed, physical_key: PhysicalKey::Code(KeyCode::KeyR), ..
        }, ..}) => {
            window.request_redraw();
        },

        Event::WindowEvent(WindowEvent::RedrawRequested) => {

            let then = Instant::now();

            target.with_frame(None, |frame| gx.with_encoder(|encoder| {
                encoder.with_render_pass(
                    frame.attachments(Some(bg_color_target), None, None),
                    |rpass| {
                        rpass.set_pipeline(&pipeline);
                        rpass.set_bind_group(0, &binding, &[]);
                        rpass.set_vertex_buffer(0, vertices.slice(..));
                        rpass.set_vertex_buffer(1, display_tex_coords.slice(..));
                        rpass.draw(0..vertex_data.len() as u32, 0..1);
                    }
                );
            })).expect("frame error");

            println!("{:?}", then.elapsed());
        },

        _ => {}
    }
}