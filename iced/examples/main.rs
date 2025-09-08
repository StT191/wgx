
use platform::winit::{window::{WindowAttributes}, event::{WindowEvent}};
use platform::*;
use wgx_iced::*;
use wgx::*;

mod ui;

main_app_closure! {
  LogLevel::Warn,
  WindowAttributes::default().with_title("WgFx"),
  init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, Event) + use<> {

  let window = ctx.window_clone();

  // let features = features!(TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES);
  // let limits = limits!(max_inter_stage_shader_components: 60);

  let (gx, mut target) = Wgx::new_with_target(window.clone(), features!(), limits!(), window.inner_size(), true, 1, None).await.unwrap();

  log::warn!("{:?}", gx.adapter.get_info());


  let renderer = Engine::new_wgx(&gx, target.format(), 4).renderer();

  let mut gui = Gui::new(ctx, renderer, ui::Ui::default(), ui::theme());


  let mut frame_counter = timer::IntervalCounter::from_secs(5.0);

  move |ctx, event| {

    let event_was_queued = gui.event(ctx, &event);

    // redraw handling
    if event_was_queued {
      ctx.request_frame();
    }

    match event {

      Event::Timeout {id: 0, ..} => {
        ctx.request_frame()
      },

      Event::WindowEvent(WindowEvent::Resized(size)) => {
        target.update(&gx, size);
      },

      Event::WindowEvent(WindowEvent::RedrawRequested) => {

        // gui handling
        let repaint_delay = gui.update(ctx);

        // draw
        target.with_frame(None, |frame| {

          let bg_color = gui.program.bg_color;
          gui.draw(frame, Some(bg_color));

        }).expect("frame error");

        if repaint_delay <= ctx.frame_duration() {
          ctx.request_frame();
        }
        else if let Some(next_frame) = ctx.frame_time().checked_add(repaint_delay) {
          ctx.set_timeout(0, next_frame);
        }

        frame_counter.add();
        if let Some(counted) = frame_counter.count() { println!("{:?}", counted) }

      },

      _ => (),
    }
  }
}