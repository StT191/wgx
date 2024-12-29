
use platform::winit::{window::WindowAttributes, event::WindowEvent, dpi::*};
use platform::{*};
use wgx::*;

main_app_closure! {
  LogLevel::Warn,
  WindowAttributes::default()
    .with_inner_size(PhysicalSize::new(1000, 1000))
    // .with_position(PhysicalPosition::new(2000, 200))
    .with_transparent(true)
  ,
  init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, Event) + use<> {

  let window = ctx.window_clone();
  let (gx, mut target) = Wgx::new_with_target(window.clone(), features!(), limits!(), window.inner_size(), false, 1, None).await.unwrap();

  let shader = gx.load_wgsl(wgsl_modules::inline!("$shader" <= {
    struct VertexData {
      @builtin(position) position: vec4f,
      @location(0) world_position: vec2f,
    }
    @vertex
    fn vs_main(@builtin(vertex_index) i: u32) -> VertexData {
      let x = mix(-1.0, 1.0, f32(i % 2));
      let y = mix(-1.0, 1.0, f32(i / 2 % 2));
      return VertexData(vec4f(x, y, 0.0, 1.0), vec2f(x, -y));
    }

    fn nsin(val: f32) -> f32 {
      return fma(sin(val), 0.5, 0.5);
    }

    const pi = 3.141592653589793;

    @fragment
    fn fs_main(@location(0) pos: vec2f) -> @location(0) vec4f {

      let d = length(pos);
      let p = 6.0;

      let r = nsin(d * p*pi + 0.0*pi);
      let g = nsin(d * p*pi + d*1.0*pi);
      let b = nsin(d * p*pi + 1.0*pi);

      return vec4f(r, g, b, fma(r, 0.2, 0.8));
    }
  }));

  let pipeline = target.render_pipeline(&gx,
    None, &[],
    (&shader, "vs_main", None, Primitive {topology: Topology::TriangleStrip, ..Primitive::default()}),
    (&shader, "fs_main", None, Some(Blend::ALPHA_BLENDING)),
  );

  move |_ctx, event| match event {

    Event::WindowEvent(WindowEvent::Resized(size)) => {
      target.update(&gx, size);
    },

    Event::WindowEvent(WindowEvent::RedrawRequested) => {
      target.with_frame(None, |frame| gx.with_encoder(|encoder| {
        encoder.with_render_pass(frame.attachments(None, None, None), |rpass| {
          rpass.set_pipeline(&pipeline);
          rpass.draw(0..4, 0..1);
        });
      })).unwrap_or_else(|m| log::error!("{m:?}"));
    }

    _ => {},
  }
}