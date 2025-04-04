
use platform::winit::{window::{WindowAttributes}, event::{WindowEvent}};
use platform::{*, Event};
use wgx_egui::*;
use wgx::*;

mod ui;

main_app_closure! {
  LogLevel::Warn,
  WindowAttributes::default().with_title("WgFx"),
  init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, Event) + use<> {

  let window = ctx.window_clone();

  let (gx, mut target) = Wgx::new_with_target(window.clone(), features!(), limits!(), window.inner_size(), false, 1, None).await.unwrap();

  // egui setup

  let mut ui = ui::new();
  let mut egs_renderer = renderer(&gx, &target);
  let mut egs = EguiCtx::new(ctx);

  // run once to initialize fonts
  gx.with_encoder(|enc| egs.run(ctx, |_ctx| {}).prepare(&mut egs_renderer, &gx, enc));

  let add_primitives = {

    let circle = Shape::circle_filled([100.0, 100.0].into(), 80.0, Color32::from_rgb(0x00, 0xF0, 0xF0));

    let text = egs.context.fonts(|fonts| {
      Shape::text(
        fonts, [200.0, 200.0].into(), Align2::LEFT_CENTER,
        "HALLO TEST Hallo Test!",
        FontId { size: 14.0, family: FontFamily::default() },
        Color32::WHITE,
      )
    });

    egs.context.tessellate(
      clip_shapes([circle, text], egs.context.screen_rect()).collect(),
      egs.screen_dsc.pixels_per_point,
    )
  };


  // epainting ...

  let mut ept_renderer = renderer(&gx, &target);
  let mut ept = EpaintCtx::new(ScreenDescriptor::from_window(&window), 2048, FontDefinitions::default());

  let mut primitives = Vec::new();

  let shapes = [

    Shape::circle_filled([100.0, 100.0].into(), 40.0, Color32::from_rgb(0xF0, 0xA0, 0x00)),

    Shape::text(
      &ept.fonts, [200.0, 220.0].into(), Align2::LEFT_CENTER,
      "EPAINT: HALLO TEST Hallo Test!",
      FontId { size: 14.0, family: FontFamily::default() },
      Color32::from_rgb(0xF0, 0x00, 0x00),
    ),
  ];

  let mut frame_counter = timer::IntervalCounter::from_secs(3.0);

  move |ctx, event| {

    let (repaint, _) = egs.event(ctx, &event);

    if repaint { ctx.request_frame(); }

    match event {

      Event::Timeout {id: 0, ..} => ctx.request_frame(),

      Event::WindowEvent(WindowEvent::Resized(size)) => {

        target.update(&gx, size);

        // redraw epait ...
        ept.screen_dsc = ScreenDescriptor::from_window(&window);

        primitives.clear();

        ept.tessellate(
          Default::default(),
          ept.clip_shapes(shapes.iter().cloned(), None),
          &mut primitives
        );

        gx.with_encoder(|encoder| {
          ept.prepare(&mut ept_renderer, &gx, encoder, &primitives);
        });
      },

      Event::WindowEvent(WindowEvent::RedrawRequested) => {

        // gui handling
        let mut output = egs.run(ctx, &mut ui);

        output.clipped_primitives.extend_from_slice(&add_primitives);

        // draw
        target.with_frame(None, |frame| gx.with_encoder(|encoder| {

          output.prepare(&mut egs_renderer, &gx, encoder);

          encoder.with_render_pass(frame.attachments(Some(Color::WHITE.into()), None, None), |rpass| {

            output.render(&egs_renderer, rpass);

            ept.render(&ept_renderer, rpass, &primitives);

          });

        })).expect("frame error");

        if output.repaint_delay <= ctx.frame_duration() {
          ctx.request_frame();
        }
        else if let Some(next_frame) = ctx.frame_time().checked_add(output.repaint_delay) {
          ctx.set_timeout(0, next_frame);
        }

        // handle other commands
        for command in output.commands {
          log::warn!("Cmd: {:#?}", command);
          if command == ViewportCommand::Close {
            ctx.exit = true;
          }
        }

        frame_counter.add();
        if let Some(counted) = frame_counter.count() { log::warn!("{:?}", counted) }
        // window.request_redraw(); // draw as many as possible

      },

      _ => (),
    }
  }
}