
use platform::winit::{window::{WindowAttributes}, event::{WindowEvent}};
use platform::{*, Event};
use wgx::{*};

use vello::{*, peniko::{*, Color}, kurbo::*};

main_app_closure! {
  LogLevel::Warn,
  WindowAttributes::default().with_title("WgFx"),
  init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, Event) + use<> {

  let window = ctx.window_clone();

  let (gx, mut target) = Wgx::new_with_target(window.clone(), features!(), limits!(), window.inner_size(), false, 1, None).await.unwrap();

  let [w, h] = target.size();
  let mut canvas_tex = TextureLot::new_2d(&gx, [w, h, 1], 1, TexFmt::Rgba8Unorm, None, TexUse::STORAGE_BINDING | TexUse::TEXTURE_BINDING);
  let blitter = wgpu::util::TextureBlitter::new(&gx.device, target.format());

  let mut renderer = Renderer::new(
     &gx.device,
     RendererOptions {
        use_cpu: false,
        antialiasing_support: AaSupport::area_only(),
        num_init_threads: std::num::NonZeroUsize::new(1),
        pipeline_cache: None,
     },
  ).expect("Failed to create renderer");

  let mut scene = vello::Scene::new();
  scene.fill(
    Fill::NonZero,
    Affine::IDENTITY,
    Color::from_rgb8(242, 140, 168),
    None,
    &Circle::new((200.0, 200.0), 120.0),
  );

  let mut frame_counter = timer::IntervalCounter::from_secs(3.0);

  move |_ctx, event| {

    match event {

      Event::WindowEvent(WindowEvent::Resized(size)) => {

        target.update(&gx, size);

        canvas_tex.descriptor.set_size_2d(size.into());
        canvas_tex = TextureLot::new(&gx, canvas_tex.descriptor);
      },

      Event::WindowEvent(WindowEvent::RedrawRequested) => {

        let [width, height] = canvas_tex.size();

        renderer.render_to_texture(
          &gx.device, &gx.queue,
          &scene, &canvas_tex.view,
          &vello::RenderParams {
            base_color: Color::BLACK, // Background color
            width, height,
            antialiasing_method: AaConfig::Area,
          },
        ).expect("Failed to render to a texture");

        // copy to frame
        target.with_frame(None, |frame| gx.with_encoder(|encoder| {
          blitter.copy(&gx.device, encoder, &canvas_tex.view, &frame.view);
        })).expect("frame error");

        frame_counter.add();
        if let Some(counted) = frame_counter.count() { log::warn!("{:?}", counted) }
        // window.request_redraw(); // draw as many as possible

      },

      _ => (),
    }
  }
}