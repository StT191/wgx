
use platform::winit::{dpi::PhysicalPosition, event::WindowEvent, keyboard::ModifiersState};

use platform::{AppCtx, Event, time::*};
use wgx::wgpu::TextureFormat;
use wgx::{Wgx, RenderAttachable, Color};

use iced_wgpu::{Renderer};
pub use iced_wgpu::{Engine};

use iced_winit::{
  graphics::{Viewport, Antialiasing},
  runtime::{user_interface::{UserInterface, Cache, State}},
  core::{Event as IcedEvent, Element, mouse::{Interaction, Cursor}, Pixels, Size, Font, renderer::Style, window::RedrawRequest},
  conversion,
};

#[cfg(target_family = "wasm")]
use iced_winit::{
  winit::{event::{KeyEvent, ElementState}, keyboard::{PhysicalKey, KeyCode}},
  runtime::keyboard::{self, key},
};

use super::{Clipboard, IntoIcedCoreColor};


// render engine constructor trait
pub trait RenderEngine {
  fn new_wgx(gx: &Wgx, format: TextureFormat, msaa: u32) -> Self;
  fn renderer(self) -> Renderer;
}

impl RenderEngine for Engine {

  fn new_wgx(gx: &Wgx, format: TextureFormat, msaa: u32) -> Self {

    let antialiasing = match msaa {
      1 => None,
      2 => Some(Antialiasing::MSAAx2),
      4 => Some(Antialiasing::MSAAx4),
      8 => Some(Antialiasing::MSAAx8),
      16 => Some(Antialiasing::MSAAx16),
      _ => panic!("RenderEngine: unsupported msaa value of {:?}", msaa),
    };

    Engine::new(&gx.adapter, gx.device.clone(), gx.queue.clone(), format, antialiasing)
  }

  fn renderer(self) -> Renderer {
    Renderer::new(self, Font::DEFAULT, Pixels(12.0))
  }
}


// gui program trait
pub trait Program {
  type Theme;
  type Message;

  fn update(&mut self, message: Self::Message);

  fn view(&self) -> Element<'_, Self::Message, Self::Theme, Renderer>;
}


// Gui
pub struct Gui<P: Program> {
  renderer: Renderer,
  pub program: P,
  cache: Cache,
  pub event_queue: Vec<IcedEvent>,
  pub message_queue: Vec<P::Message>,
  pub viewport: Viewport,
  cursor_position: PhysicalPosition<f64>,
  interaction: Interaction,
  modifiers: ModifiersState,
  pub theme: P::Theme,
  pub style: Style,
  pub clipboard: Clipboard,
}

impl<P: Program> Gui<P> {

  pub fn new(app_ctx: &AppCtx, renderer: Renderer, program: P, theme: P::Theme) -> Self {

    let size = app_ctx.window().inner_size();
    let scale_factor = app_ctx.window().scale_factor() as f32;

    let viewport = Viewport::with_physical_size(Size::new(size.width, size.height), scale_factor);

    #[cfg(not(target_family = "wasm"))] let clipboard = Clipboard::connect(app_ctx.window_clone());
    #[cfg(target_family = "wasm")] let clipboard = Clipboard::connect(app_ctx);

    Self {
      renderer, program,
      cache: Cache::new(),
      event_queue: Vec::new(),
      message_queue: Vec::new(),
      viewport,
      cursor_position: PhysicalPosition::new(-1.0, -1.0),
      interaction: Interaction::default(),
      modifiers: ModifiersState::default(), theme,
      style: Style::default(),
      clipboard,
    }
  }

  pub fn event(&mut self, _app_ctx: &AppCtx, app_event: &Event) -> bool {
    match app_event {

      Event::WindowEvent(window_event) => {

        match window_event {
          WindowEvent::CursorMoved { position, .. } => {
            self.cursor_position = *position;
          }

          WindowEvent::ModifiersChanged(modifiers) => {
            self.modifiers = modifiers.state();
          }

          #[cfg(target_family = "wasm")]
          WindowEvent::KeyboardInput { event: KeyEvent {
            physical_key: PhysicalKey::Code(KeyCode::KeyV), state: ElementState::Pressed, ..
          }, ..} => {
            if self.modifiers.control_key() && self.clipboard.web.is_connected() {
              if !self.clipboard.web.is_listening() {
                self.clipboard.web.fetch();
              }
              // return early, don't queue event, wait for ClipboardPaste/Fetch
              return false;
            }
          }

          WindowEvent::Resized(size) => {
            self.viewport = Viewport::with_physical_size(
              Size::new(size.width, size.height),
              self.viewport.scale_factor(),
            );
          }

          WindowEvent::ScaleFactorChanged { scale_factor, ..} => {
            self.viewport = Viewport::with_physical_size(
              Size::new(self.viewport.physical_width(), self.viewport.physical_height()),
              *scale_factor as f32,
            );
          }

          _ => {}
        }

        if let Some(iced_event) = conversion::window_event(
          window_event.clone(), self.viewport.scale_factor(), self.modifiers,
        ) {
          self.event_queue.push(iced_event);
          true
        }
        else { false }
      },

      #[cfg(target_family="wasm")]
      Event::ClipboardPaste | Event::ClipboardFetch => {
        self.event_queue.push(IcedEvent::Keyboard(keyboard::Event::KeyPressed {
          key: key::Key::Character("v".into()),
          modified_key: key::Key::Character("v".into()),
          physical_key: key::Physical::Code(key::Code::KeyV),
          location: keyboard::Location::Standard,
          modifiers: keyboard::Modifiers::CTRL,
          text: None,
        }));
        true
      },

      _ => false,
    }
  }

  pub fn update(&mut self, app_ctx: &AppCtx) -> Duration {

    let mut user_interface = UserInterface::build(
      self.program.view(),
      self.viewport.logical_size(),
      std::mem::take(&mut self.cache),
      &mut self.renderer,
    );

    // Update the user interface
    let (state, _event_statuses) = user_interface.update(
      &self.event_queue,
      Cursor::Available(conversion::cursor_position(
        self.cursor_position,
        self.viewport.scale_factor(),
      )),
      &mut self.renderer,
      &mut self.clipboard,
      &mut self.message_queue,
    );

    self.cache = user_interface.into_cache();

    self.event_queue.clear();

    for message in self.message_queue.drain(..) {
      self.program.update(message);
    }

    // handle redraw state
    match state {
      State::Updated { mouse_interaction, redraw_request, .. } => {

        // handle mouse interaction
        if mouse_interaction != self.interaction {
          app_ctx.window().set_cursor(conversion::mouse_interaction(mouse_interaction));
          self.interaction = mouse_interaction;
        }

        match redraw_request {
          RedrawRequest::NextFrame => Duration::ZERO,
          RedrawRequest::Wait => Duration::MAX,
          RedrawRequest::At(instant) => instant.saturating_duration_since(Instant::now()),
        }
      },

      State::Outdated => Duration::MAX, // should't occur normaly
    }
  }

  pub fn draw(&mut self, target: &impl RenderAttachable, clear_color: Option<Color>) {

    // draw user interface
    let mut user_interface = UserInterface::build(
      self.program.view(),
      self.viewport.logical_size(),
      std::mem::take(&mut self.cache),
      &mut self.renderer,
    );

    user_interface.draw(
      &mut self.renderer,
      &self.theme,
      &self.style,
      Cursor::Available(conversion::cursor_position(
        self.cursor_position,
        self.viewport.scale_factor(),
      )),
    );

    self.cache = user_interface.into_cache();

    // render to target
    let (view, format, _) = target.color_views();

    self.renderer.present(
      clear_color.map(|cl| cl.iced_core()),
      format,
      view,
      &self.viewport,
    );
  }

  pub fn screenshot(&mut self, color: Color) -> Vec<u8> {
    self.renderer.screenshot(&self.viewport, color.iced_core())
  }

}
