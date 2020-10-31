#![allow(unused)]

// imports
use std::{time::{Instant}, fs::File, io::Read};

use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::*,
};

use wgx::*;
use cgmath::*;


// main
fn main() {

    const DEPTH_TESTING:bool = false;
    const ALPHA_BLENDING:bool = false;
    const MSAA:u32 = 1;


    let event_loop = EventLoop::new();

    let window = Window::new(&event_loop).unwrap();
    window.set_title("WgFx");


    let mut gx = Gx::new(&window, DEPTH_TESTING, MSAA);


    // text_render
    let mut font_data = Vec::new();
    File::open("fonts/Destain-Xgma.ttf").expect("failed loading font").read_to_end(&mut font_data);

    let mut glyphs = gx.glyph_brush(TexOpt::Output, font_data).expect("invalid font");



    let mut text_input = SimpleTextInput::new("Hey Ho!\nWhat is going on? Anyway?\n");
    text_input.set_curser_end();


    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, ..} => {
                *control_flow = ControlFlow::Exit;
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                gx.update(size.width, size.height, DEPTH_TESTING, MSAA);
            },

            Event::WindowEvent { event: WindowEvent::ReceivedCharacter(letter), .. } => {
                if text_input.insert(letter) {
                    window.request_redraw();
                }
                else {
                    println!("{:?}", letter);
                }
            },

            Event::WindowEvent { event:WindowEvent::KeyboardInput { input: KeyboardInput {
                virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
            }, ..}, ..} => {
                if match keycode {
                    VirtualKeyCode::Left => text_input.recede(),
                    VirtualKeyCode::Right => text_input.advance(),
                    VirtualKeyCode::Back => text_input.remove(),
                    VirtualKeyCode::Delete => text_input.delete(),
                    VirtualKeyCode::Home => text_input.set_curser(0),
                    VirtualKeyCode::End => text_input.set_curser_end(),
                    _ => false
                } {
                    window.request_redraw();
                }
            },

            Event::RedrawRequested(_) => {

                let then = Instant::now();


                gx.with_encoder_frame(|encoder, gx| {
                    gx.draw(encoder, Some(Color::GREEN), &[]);

                    glyphs.add_text(
                        vec![
                            Text::new(&text_input.text_before_curser()).with_scale(50.0)
                            .with_color(Color::from([0x2,0x2,0x12])),
                            Text::new("|").with_scale(50.0)
                            .with_color(Color::WHITE),
                            Text::new(text_input.text_after_curser()).with_scale(50.0)
                            .with_color(Color::from([0x2,0x2,0x12])),
                        ],
                        None, Some((800.0, f32::INFINITY)),
                        Some(layout!(Wrap, Left, Top))
                    );

                    let trf =
                        // Matrix4::from_translation((0.0, 0.0, 0.0).into()) *
                        // Matrix4::from_angle_z(Deg(45.0)) *
                        // Matrix4::from_angle_y(Deg(88.0)) *
                        // Matrix4::from_translation((-1200.0, 900.0, 0.0).into()) *
                        Matrix4::from_translation((-1200.0, 900.0, 0.0).into()) *
                        // Matrix4::from_angle_x(Deg(45.0)) *
                        Matrix4::from_scale(3.0);

                    gx.draw_glyphs(encoder, &mut glyphs, Some(trf), None);
                });


                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}
