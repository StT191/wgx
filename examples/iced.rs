#![feature(duration_consts_float)]

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
    widget::{Column, Row, Text, TextInput, Slider}
};


#[derive(Debug, Clone)]
pub enum Message {
    Color(Color),
    Text(String),
}

pub struct Controls {
    pub color: Color,
    text: String,
}

impl Controls {
    pub fn new() -> Controls {
        Controls { color: Color::from([0.46, 0.60, 0.46]), text: "".to_string() }
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

    fn view(&self) -> Element<Message, Renderer> {
        let color = self.color;

        Column::new().width(Length::Fill).height(Length::Fill).align_items(Alignment::Center)
        .padding(15).spacing(15)
        .push(Text::new(&self.text).size(22).style(Color::WHITE.iced()).width(Length::Fill).height(Length::Fill))
        .push(TextInput::new(/*&mut self.text_input, */"input text", &self.text, Message::Text).size(22))
        .push(Text::new("Background color").style(Color::WHITE.iced()))
        .push(
            Row::new().width(Length::Units(500)).spacing(20)
            .push(Slider::new(0.0..=1.0, color.r, move |v| Message::Color(Color {r: v, ..color})).step(0.01))
            .push(Slider::new(0.0..=1.0, color.g, move |v| Message::Color(Color {g: v, ..color})).step(0.01))
            .push(Slider::new(0.0..=1.0, color.b, move |v| Message::Color(Color {b: v, ..color})).step(0.01))
        )
        .push(Text::new(format!("{:?}", color)).size(18).style(Color::WHITE.iced()))
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
    let (gx, surface) = block_on(Wgx::new(Some(&window), Features::empty(), limits!{})).unwrap();
    let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (width, height), MSAA, DEPTH_TESTING).unwrap();


    // iced setup
    let renderer = gx.iced_renderer(Settings::default(), target.format());

    let mut gui = Iced::new(renderer, Controls::new(), (width, height), &window);


    let mut frame_timer = timer::StepInterval::from_secs(1.0 / 60.0);
    // let mut frame_counter = timer::IntervalCounter::from_secs(5.0);

    event_loop.run(move |event, _, control_flow| {

        match event {

            Event::NewEvents(StartCause::ResumeTimeReached {..}) => {
                window.request_redraw(); // request frame
                control_flow.set_wait();
            },

            Event::WindowEvent { event, .. } => {
                match event {

                    WindowEvent::CloseRequested => {
                        control_flow.set_exit();
                    }

                    WindowEvent::Resized(size) => {
                        target.update(&gx, (size.width, size.height));
                        window.request_redraw();
                    }

                    /*WindowEvent::KeyboardInput { input: KeyboardInput {
                        virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
                    }, ..} => {
                        if keycode == VirtualKeyCode::R {
                            window.request_redraw();
                        }
                    }*/
                    _ => (),
                }

                gui.event(&event, &window);
            }

            Event::MainEventsCleared => {

                let (need_redraw, _cmd) = gui.update();

                gui.update_cursor(&window);

                let advanced = frame_timer.advance_if_elapsed();

                if need_redraw && *control_flow != ControlFlow::WaitUntil(frame_timer.next) {
                    * control_flow = if advanced {
                        window.request_redraw();
                        ControlFlow::Wait
                    }
                    else { ControlFlow::WaitUntil(frame_timer.next) }
                }
            }

            Event::RedrawRequested(_) => {

                target.with_encoder_frame(&gx, |mut encoder, frame| {

                    encoder.render_pass(frame.attachments(Some(gui.program().color), None));

                    gui.draw(&gx, &mut encoder, frame);

                }).expect("frame error");

                gui.recall_staging_belt();

                // frame_counter.add();
                // if let Some(counted) = frame_counter.count() { println!("{:?}", counted) }
            },

            _ => {}
        }
    });
}