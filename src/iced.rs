
use iced_wgpu::{Viewport, Renderer};
use iced_winit::{
    winit::{
        dpi::PhysicalPosition, window::Window,
        event::{WindowEvent, ModifiersState as Modifiers},
    },
    renderer::{Style}, program::{Program, State}, mouse::Interaction,
    Clipboard as NativeClipboard,
    *,
};
#[cfg(target_family = "wasm")]
use iced_winit::winit::event::{KeyboardInput};

use iced_native::clipboard::Clipboard as ClipboardTrait;

use wgpu::{CommandEncoder, util::StagingBelt};
use crate::{Wgx, RenderAttachable};


pub struct Iced<P:'static + Program<Renderer=Renderer>, C: ClipboardTrait> {
    renderer: Renderer,
    program_state: State<P>,
    viewport: Viewport,
    cursor: PhysicalPosition<f64>,
    interaction: Interaction,
    pub modifiers: Modifiers,
    clipboard: C,
    staging_belt: StagingBelt,
    debug: Debug,
}


impl<P:'static + iced_winit::Program<Renderer=Renderer>> Iced<P, NativeClipboard> {
    pub fn new_native(renderer:Renderer, program:P, size:(u32, u32), window:&Window) -> Self {
        let clipboard = NativeClipboard::connect(window);
        Self::new_with_clipboad(renderer, program, size, window, clipboard)
    }
}


impl<P:'static + iced_winit::Program<Renderer=Renderer>, C: ClipboardTrait> Iced<P, C> {

    pub fn new_with_clipboad(mut renderer:Renderer, program:P, (width, height):(u32, u32), window:&Window, clipboard: C) -> Self {

        let mut debug = Debug::new();

        let viewport = Viewport::with_physical_size(Size::new(width, height), window.scale_factor());

        let cursor = PhysicalPosition::new(-1.0, -1.0);

        let program_state = State::new(
            program, viewport.logical_size(),
            &mut renderer, &mut debug,
        );

        let interaction = program_state.mouse_interaction();

        Self {
            renderer, program_state, viewport, cursor, interaction,
            modifiers: Modifiers::default(),
            clipboard,
            staging_belt: StagingBelt::new(10240),
            debug,
        }
    }

    pub fn program(&mut self) -> &P {
        self.program_state.program()
    }

    pub fn clipboard(&mut self) -> &mut C {
        &mut self.clipboard
    }

    pub fn event(&mut self, event:&WindowEvent) {

        // on wasm we need to track if modifiers changed manually and fire the modifiers changed event manually
        #[cfg(target_family = "wasm")]
        let mut modifiers_changed = false;

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = *position;
            }

            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = *modifiers;
            }

            // collect modifiers manually on web platform
            #[cfg(target_family = "wasm")]
            #[allow(deprecated)]
            WindowEvent::KeyboardInput { input: KeyboardInput { modifiers, .. }, ..} => {
                if &self.modifiers != modifiers {
                    self.modifiers = *modifiers;
                    modifiers_changed = true;
                }
            }

            WindowEvent::Resized(size) => {
                self.viewport = Viewport::with_physical_size(
                    Size::new(size.width, size.height),
                    self.viewport.scale_factor(),
                );
            }

            WindowEvent::ScaleFactorChanged { scale_factor, ref new_inner_size } => {
                self.viewport = Viewport::with_physical_size(
                    Size::new(new_inner_size.width, new_inner_size.height),
                    *scale_factor,
                );
            }

            _ => (),
        }

        #[cfg(target_family = "wasm")]
        if modifiers_changed {
            if let Some(event) = iced_winit::conversion::window_event(
                &WindowEvent::ModifiersChanged(self.modifiers), self.viewport.scale_factor(), self.modifiers,
            ) {
                self.program_state.queue_event(event);
            }
        }

        if let Some(event) = iced_winit::conversion::window_event(
            event, self.viewport.scale_factor(), self.modifiers,
        ) {
            self.program_state.queue_event(event);
        }
    }


    pub fn message(&mut self, message:P::Message) {
        self.program_state.queue_message(message)
    }


    pub fn update_cursor(&mut self, window: &Window) {
        let interaction = self.program_state.mouse_interaction();
        if self.interaction != interaction {
            window.set_cursor_icon(conversion::mouse_interaction(interaction));
            self.interaction = interaction;
        }
    }


    pub fn update(&mut self) -> (bool, Option<Command<P::Message>>) {
        if !self.program_state.is_queue_empty() {

            let (_events, command) = self.program_state.update(
                self.viewport.logical_size(),
                conversion::cursor_position(
                    self.cursor,
                    self.viewport.scale_factor(),
                ),
                &mut self.renderer,
                &Theme::Light,
                &Style { text_color: Color::BLACK },
                &mut self.clipboard,
                &mut self.debug,
            );

            (true, command)
        }
        else { (false, None) }
    }


    pub fn draw(&mut self, gx:&Wgx, encoder:&mut CommandEncoder, target: &impl RenderAttachable) {

        // borrow before the closure
        let (staging_belt, viewport, debug) = (&mut self.staging_belt, &self.viewport, &self.debug);

        self.renderer.with_primitives(|backend, primitive| {
            backend.present(
                &gx.device,
                staging_belt,
                encoder,
                target.color_views().0,
                primitive,
                viewport,
                &debug.overlay(),
            );
        });

        self.staging_belt.finish();
    }


    pub fn recall_staging_belt(&mut self) {
        self.staging_belt.recall();
    }
}