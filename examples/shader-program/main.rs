
use platform::winit::{
  window::WindowAttributes, event::{WindowEvent, KeyEvent, ElementState}, keyboard::{PhysicalKey, KeyCode},
  dpi::PhysicalSize,
};
use platform::{*, time::*};
use wgx::{*};


main_app_closure! {
  LogLevel::Warn,
  WindowAttributes::default().with_inner_size(PhysicalSize {width: 1280, height: 900}),
  init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, &AppEvent) {

  let window = ctx.window_clone();

  let msaa = 1;
  let depth_testing = None;
  let blending = None;

  let (gx, mut target) = Wgx::new_with_target(
    window.clone(), features!(PUSH_CONSTANTS), limits!{max_push_constant_size: 4},
    window.inner_size(), msaa, depth_testing,
  ).await.unwrap();


  let shader_src = match &*std::env::args().nth(1).expect("Specify a program!") {
    "balls" => wgsl_modules::include!("programs/balls.wgsl"),
    "opt" => wgsl_modules::include!("programs/opt.wgsl"),
    "wavy" => wgsl_modules::include!("programs/wavy.wgsl"),
    unkown => panic!("program '{unkown}' doesn't exist"),
  };

  // pipeline
  let shader = gx.load_wgsl(shader_src);

  let layout = gx.layout(&[
    binding!(0, Stage::VERTEX_FRAGMENT, UniformBuffer, 16),
  ]);

  let pipeline = target.render_pipeline(&gx,
    Some((push_constants![0..4 => Stage::FRAGMENT], &[&layout])),
    &[vertex_dsc!(Vertex, 0 => Float32x2)],
    (&shader, "vs_main", Primitive::default()),
    (&shader, "fs_main", blending),
  );

  // vertices
  let vertex_data = [
    [-1.0, -1.0f32], [ 1.0, -1.0f32], [ 1.0,  1.0f32],
    [-1.0, -1.0f32], [ 1.0,  1.0f32], [-1.0,  1.0f32],
  ];
  let vertices = gx.buffer_from_data(BufUse::VERTEX, &vertex_data[..]);


  // data
  const DF:f32 = 0.01;

  let PhysicalSize { width, height } = window.inner_size();
  let (mut width, mut height) = (width as f32, height as f32);
  let mut scale = 1.0 as f32;

  let time = Instant::now();


  // buffer
  let view_buffer = gx.buffer_from_data(BufUse::UNIFORM | BufUse::COPY_DST, [width, height, width/height, scale]);

  // binding
  let binding = gx.bind(&layout, &[
    bind!(0, Buffer, &view_buffer),
  ]);


  // event loop
  ctx.animate = true;

  // let mut frame_counter = timer::IntervalCounter::from_secs(5.0);

  move |_ctx: &mut AppCtx, event: &AppEvent| match event {

    AppEvent::WindowEvent(WindowEvent::Resized(size)) => {
      target.update(&gx, *size);

      width = size.width as f32;
      height = size.height as f32;

      // write buffer
      gx.write_buffer(&view_buffer, 0, [width, height, width/height, scale]);
    },

    AppEvent::WindowEvent(WindowEvent::KeyboardInput { event: KeyEvent {
      physical_key: PhysicalKey::Code(keycode), state: ElementState::Pressed, ..
    }, ..}) => {
      let mut update = true;

      match keycode {
        KeyCode::KeyY => { scale += DF; },
        KeyCode::KeyX => { scale -= DF; },

        KeyCode::KeyR => { scale = 1.0; },

        _ => { update = false; }
      } {
        if update { gx.write_buffer(&view_buffer, 0, [width, height, width/height, scale]); }
      }
    },

    AppEvent::WindowEvent(WindowEvent::RedrawRequested) => {

      // draw
      target.with_frame(None, |frame| gx.with_encoder(|encoder| {
        encoder.with_render_pass(frame.attachments(Some(Color::BLACK), None, None), |rpass| {
          rpass.set_pipeline(&pipeline);
          rpass.set_bind_group(0, &binding, &[]);
          rpass.set_vertex_buffer(0, vertices.slice(..));
          rpass.set_push_constants(Stage::FRAGMENT, 0, &time.elapsed().as_secs_f32().to_ne_bytes());
          rpass.draw(0..vertex_data.len() as u32, 0..1);
        });
      })).expect("frame error");

      // statistics
      /*frame_counter.add();
      if let Some(counted) = frame_counter.count() { println!("{:?}", counted) }*/
    },

    _ => {}
  }
}