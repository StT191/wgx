
use platform::winit::{window::WindowAttributes, event::WindowEvent, dpi::*};
use platform::{*};
use wgx::{*, math::*};

main_app_closure! {
  LogLevel::Warn,
  WindowAttributes::default()
    .with_inner_size(PhysicalSize::new(1000, 1000))
    // .with_position(PhysicalPosition::new(1000, 200))
  ,
  init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, Event) + use<> {

  let window = ctx.window_clone();
  let (gx, mut target) = Wgx::new_with_target(window.clone(), features!(), limits!(), window.inner_size(), false, 1, None).await.unwrap();

  let shader = gx.load_wgsl(wgsl_modules::inline!("$shader" <= {
    struct VertexData {
      @builtin(position) position: vec4f,
      @location(0) pos: vec2f,
      @location(1) col: u32,
    }
    @vertex
    fn vs_main(@location(0) p: vec2f, @location(1) col: u32) -> VertexData {
      let pos = p / 1000.0 * 2.0 - vec2f(1.0, 1.0);
      return VertexData(vec4f(pos, 0.0, 1.0), p, col);
    }
    @fragment
    fn fs_main(@location(0) _pos: vec2f, @location(1) col: u32) -> @location(0) vec4f {
      return unpack4x8unorm(col);
    }
  }));

  let pipeline = RenderPipelineConfig::new(
      &[vertex_dsc!(Vertex, 0 => Float32x2, 1 => Uint32)],
      &shader, "vs_main", Primitive {
        topology: Topology::PointList,
        ..Primitive::default()
      },
    )
    .fragment(&shader, "fs_main")
    .render_target::<1>(&target, Some(Blend::ALPHA_BLENDING), Default::default())
    .pipeline(&gx)
  ;


  #[repr(C)]
  #[derive(Default, Clone, Copy)]
  struct Vtx(Vec2, u32);
  unsafe impl ReadBytes for Vtx {}

  const LINE_LEN: usize = 2048;
  const VTX_LEN: usize = LINE_LEN + 10;

  let mut vertices: [Vtx; VTX_LEN] = [Default::default(); VTX_LEN];

  // bezier
  let l0 = vec2(0.0, 0.0);
  let l1 = vec2(0.0, 1000.0);
  let l2 = vec2(1000.0, 1000.0);
  let d0 = l1 - l0;
  let d1 = l2 - l1;

  let bezier = move |t| {
    let h0 = l0 + t*d0;
    let h1 = l1 + t*d1;
    let dh = h1 - h0;
    h0 + t*dh
  };

  let closest = move |pos| {

    let mut min = 0.0;
    let mut max = 1.0;
    let mut avg = (max + min) / 2.0;

    while max - min > 0.001 {

      let d_max = bezier(max).distance(pos);
      let d_min = bezier(min).distance(pos);

      if d_min < d_max {
        max = avg;
      }
      else {
        min = avg;
      }

      avg = (max + min) / 2.0;
    }

    avg
  };


  for i in 0..LINE_LEN {
    let t = i as f32 / (LINE_LEN-1) as f32;
    vertices[i] = Vtx(bezier(t), Color::from_value_u8(0x33).u32());
  }

  let vtx_buff = gx.buffer_from_data(BufUse::VERTEX | BufUse::COPY_DST, vertices);
  let vtx_size = std::mem::size_of::<Vtx>();

  window.set_cursor_visible(false);

  move |_ctx, event| match event {

    Event::WindowEvent(WindowEvent::Resized(size)) => {
      target.update(&gx, size);
    },

    Event::WindowEvent(WindowEvent::CursorMoved {position, ..}) => {

      let pos = vec2(position.x as f32, 1000.0 - position.y as f32);

      gx.write_buffer(&vtx_buff, (LINE_LEN * vtx_size) as u64, [
        Vtx(pos, Color::RED.u32()),
        Vtx(bezier(closest(pos)), Color::GREEN.u32()),
      ]);

      window.request_redraw();
    },

    Event::WindowEvent(WindowEvent::RedrawRequested) => {
      target.with_frame(None, |frame| gx.with_encoder(|encoder| {
        encoder.with_render_pass(frame.attachments(Some(Color::BLACK), None, None), |rpass| {
          rpass.set_pipeline(&pipeline);
          rpass.set_vertex_buffer(0, vtx_buff.slice(..));
          rpass.draw(0..vertices.len() as u32, 0..1);
        });
      })).unwrap_or_else(|m| log::error!("{m:?}"));
    }

    _ => {},
  }
}