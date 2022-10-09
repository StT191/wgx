
// use std::{time::{Instant}};
use futures::executor::block_on;
use iced_wgpu::Settings;
use iced_winit::winit;
use self::winit::{
    dpi::{PhysicalSize},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, Icon}, event::*,
};
use wgx::{*, /*cgmath::**/};


// gui

use iced_wgpu::Renderer;
use iced_winit::{
    Alignment, Command, Element, Length, Program,
    widget::{Column, Row, Text, text_input, TextInput, slider, Slider}
};


#[derive(Debug, Clone)]
pub enum Message {
    Color(Color),
    Text(String),
}

pub struct Controls {
    pub color: Color,
    text: String,
    text_input: text_input::State,
    sliders: [slider::State; 3],
}

// Qt.rgba(0.46, 0.6, 0.46, 1)

impl Controls {
    pub fn new() -> Controls {
        Controls {
            color: Color::from([0.46, 0.60, 0.46]),
            text: "".to_string(),
            text_input: Default::default(),
            sliders: Default::default(),
        }
    }
}


impl Program for Controls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Color(color) => { self.color = color; }
            Message::Text(text) => { self.text = text; }
            // _ => {}
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let [sl_r, sl_g, sl_b] = &mut self.sliders;
        let color = self.color;

        Column::new().width(Length::Fill).height(Length::Fill).align_items(Alignment::Center)
        .padding(15).spacing(15)
        .push(Text::new(&self.text).size(22).color(Color::WHITE).width(Length::Fill).height(Length::Fill))
        .push(TextInput::new(&mut self.text_input, "input text", &self.text, Message::Text).size(22))
        .push(Text::new("Background color").color(Color::WHITE))
        .push(
            Row::new().width(Length::Units(500)).spacing(20)
            .push(Slider::new(sl_r, 0.0..=1.0, color.r, move |v| Message::Color(Color {r: v, ..color})).step(0.01))
            .push(Slider::new(sl_g, 0.0..=1.0, color.g, move |v| Message::Color(Color {g: v, ..color})).step(0.01))
            .push(Slider::new(sl_b, 0.0..=1.0, color.b, move |v| Message::Color(Color {b: v, ..color})).step(0.01))
        )
        .push(Text::new(format!("{:?}", color)).size(18).color(Color::WHITE))
        .into()
    }
}



fn main() {

    const DEPTH_TESTING:bool = false;
    // const ALPHA_BLENDING:bool = false;
    const MSAA:u32 = 1;

    // load icon
    let img = image::load_from_memory(include_bytes!("./img/logo_red_96.png")).expect("failed loading image").into_rgba8();

    // window setup
    let event_loop = EventLoop::new();


    let (width, height) = (1200, 800);

    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((width, height)));
    window.set_title("WgFx");

    // window icon
    let (w, h) = (img.width(), img.height());
    window.set_window_icon(Some(Icon::from_rgba(img.into_raw(), w, h).expect("failed converting image to icon")));


    // wgx setup
    let mut gx = block_on(Wgx::new(Some(&window), Features::empty(), limits!{})).unwrap();
    let mut target = gx.surface_target((width, height), DEPTH_TESTING, MSAA).unwrap();


    // iced setup
    let renderer = gx.iced_renderer(Settings::default(), target.format());

    let mut gui = Iced::new(renderer, Controls::new(), (width, height), &window, false);


    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Wait;

        match event {

            Event::WindowEvent { event, .. } => {
                match event {

                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }

                    WindowEvent::Resized(size) => {
                        target.update(&gx, (size.width, size.height));
                        window.request_redraw();
                    }

                    WindowEvent::KeyboardInput { input: KeyboardInput {
                        virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
                    }, ..} => {
                        if keycode == VirtualKeyCode::R {
                            window.request_redraw();
                        }
                    }
                    _ => (),
                }

                gui.event(&event, &window);
            }

            Event::MainEventsCleared => {
                gui.update(&window);

                /*if let Some(command) = res {
                    for action in command.actions() {

                        if let Action::Future(future) = action {
                            futures::executor::block_on(async {
                                println!("{:?}", future.await)
                            });
                        }
                        else {
                            println!("{:?}", action)
                        }
                    }
                }*/
            }

            Event::RedrawRequested(_) => {

                // let then = Instant::now();
                target.with_encoder_frame(&gx, |mut encoder, attachment| {

                    encoder.render_pass(attachment, Some(gui.program().color));

                    gui.draw(&gx, &mut encoder, attachment, &window);

                }).expect("frame error");

                gui.recall_staging_belt();

                // println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}