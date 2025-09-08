
use platform::winit::event::WindowEvent;
use platform::{time::Duration, AppCtx, Event};

#[cfg(target_family="wasm")]
use platform::{WebClipboard, log};

use egui::{Context, ClippedPrimitive, TexturesDelta, ViewportCommand, ViewportInfo, ViewportId};
use egui_winit::{State, update_viewport_info, process_viewport_commands, ActionRequested};

use wgx::{WgxDeviceQueue, wgpu::{CommandEncoder, RenderPass}};
use crate::*;


pub struct EguiCtx {
  pub context: Context,
  pub state: State,
  pub screen_dsc: ScreenDescriptor,

  #[cfg(target_family="wasm")]
  pub web_clipboard: WebClipboard,
}

impl EguiCtx {

  pub fn new(app_ctx: &AppCtx) -> Self {
    let context = Context::default();
    // install image loaders, need to be added via features in egui_extras
    egui_extras::install_image_loaders(&context);

    let screen_dsc = ScreenDescriptor::from_window(app_ctx.window());

    #[allow(unused_mut)]
    let mut state = State::new(
      context.clone(), ViewportId::ROOT, app_ctx.window(),
      Some(screen_dsc.pixels_per_point), None /* theme */, None /* max_texture_side */,
    );

    #[cfg(not(target_family="wasm"))] {
      Self { state, screen_dsc, context }
    }

    #[cfg(target_family="wasm")] {

      state.set_clipboard_text("DUMMY_CONTENT".to_string());

      let web_clipboard = WebClipboard::connect(app_ctx, true);
      log::warn!("{:?}", &web_clipboard);

      Self { state, screen_dsc, context, web_clipboard }
    }
  }

  pub fn event(&mut self, app_ctx: &AppCtx, app_event: &Event) -> (bool, bool) {
    match app_event {

      Event::WindowEvent(window_event) => {

        if matches!(window_event, WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged {..}) {
          self.screen_dsc = ScreenDescriptor::from_window(app_ctx.window());
        }

        if *window_event != WindowEvent::RedrawRequested {
          let res = self.state.on_window_event(app_ctx.window(), window_event);

          #[cfg(target_family="wasm")] {

            let events = &mut self.state.egui_input_mut().events;

            if let Some(egui::Event::Paste(_)) = events.last() {

              if !self.web_clipboard.is_listening() {
                self.web_clipboard.fetch();
              }

              // unqueue event, wait for ClipboardPaste/Fetch
              events.pop();

              return (false, false)
            }
          }

          return (res.repaint, res.consumed)
        }

        (false, false)
      },

      #[cfg(target_family="wasm")]
      Event::ClipboardPaste | Event::ClipboardFetch => {
        if let Some(text) = self.web_clipboard.read() {
          self.state.egui_input_mut().events.push(egui::Event::Paste(text));
          (true, true)
        }
        else {(false, false)}
      },

      _ => (false, false),
    }
  }

  pub fn run(&mut self, app_ctx: &AppCtx, ui_fn: impl FnMut(&Context)) -> FrameOutput {

    let mut input = self.state.take_egui_input(app_ctx.window());

    let viewport_id = input.viewport_id;

    let viewport_info = input.viewports.get_mut(&viewport_id).unwrap();
    update_viewport_info(viewport_info, &self.context, app_ctx.window(), false);

    let mut output = self.context.run(input, ui_fn);

    #[cfg(target_family="wasm")]
    output.platform_output.commands.retain_mut(|command| {
      match command {
        OutputCommand::CopyText(text) => {
          self.web_clipboard.write(std::mem::take(text));
          false
        },
        _ => true,
      }
    });

    self.state.handle_platform_output(app_ctx.window(), output.platform_output);

    let viewport_output = output.viewport_output.remove(&viewport_id).unwrap();

    let mut viewport_info = ViewportInfo::default();
    let mut actions_requested = Vec::new();

    if !viewport_output.commands.is_empty() {
      process_viewport_commands(
        &self.context,
        &mut viewport_info,
        viewport_output.commands.iter().cloned(),
        app_ctx.window(),
        &mut actions_requested,
      );
    }

    FrameOutput {
      clipped_primitives: self.context.tessellate(output.shapes, output.pixels_per_point),
      textures_delta: output.textures_delta,
      screen_dsc: self.screen_dsc.clone(),
      viewport_events: viewport_info.events,
      commands: viewport_output.commands,
      actions_requested,
      repaint_delay: viewport_output.repaint_delay,
    }
  }
}


pub struct FrameOutput {
  pub clipped_primitives: Vec<ClippedPrimitive>,
  pub textures_delta: TexturesDelta,
  pub screen_dsc: ScreenDescriptor,
  pub viewport_events: Vec<ViewportEvent>,
  pub commands: Vec<ViewportCommand>,
  pub actions_requested: Vec<ActionRequested>,
  pub repaint_delay: Duration,
}

impl FrameOutput {

  pub fn prepare(&self, renderer: &mut Renderer, gx: &impl WgxDeviceQueue, encoder: &mut CommandEncoder) {
    prepare_renderer(renderer, gx, encoder, &self.textures_delta, &self.clipped_primitives, &self.screen_dsc);
  }

  pub fn render<'a>(&'a self, renderer: &'a Renderer, rpass: &mut RenderPass<'static>) {
    renderer.render(rpass, &self.clipped_primitives, &self.screen_dsc);
  }
}