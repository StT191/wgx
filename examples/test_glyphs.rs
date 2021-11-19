#![allow(unused)]

// imports
use std::{time::{Instant}, fs::read};

use winit::{
    dpi::PhysicalSize,
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


    use futures::task::SpawnExt;
    let mut local_pool = futures::executor::LocalPool::new();


    // window setup
    let event_loop = EventLoop::new();


    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((1200, 1000)));
    window.set_title("WgFx");


    // wgx setup
    let mut gx = Wgx::new(Some(&window), 0, None);
    let mut target = gx.surface_target((1200, 1000), DEPTH_TESTING, MSAA).expect("render target failed");


    // text_render
    // let font_data = include_bytes!("../fonts/font_active.ttf");
    let font_data = read("fonts/font_active.ttf").expect("failed loading font");

    let mut glyphs = gx.glyph_brush(OUTPUT, font_data).expect("invalid font");

    let mut text_input = SimpleTextInput::new("Hey Ho!\nWhat is going on? Anyway?\n");
    text_input.set_curser_end();


    let mut staging_belt = wgpu::util::StagingBelt::new(10240);


    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, ..} => {
                *control_flow = ControlFlow::Exit;
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                target.update(&gx, (size.width, size.height));
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
                    VirtualKeyCode::Up => text_input.recede_line(),
                    VirtualKeyCode::Down => text_input.advance_line(),
                    VirtualKeyCode::Back => text_input.remove(),
                    VirtualKeyCode::Delete => text_input.delete(),
                    VirtualKeyCode::Home => text_input.set_curser(0),
                    VirtualKeyCode::End => text_input.set_curser_end(),
                    VirtualKeyCode::Return => text_input.insert('\n'),
                    _ => false
                } {
                    window.request_redraw();
                }
            },

            Event::RedrawRequested(_) => {

                let then = Instant::now();

                let (width, height) = target.size();
                let (width, height) = (width as f32, height as f32);

                glyphs.add_text(
                    vec![
                        Text::new(&text_input.text_before_curser()).with_scale(100.0)
                        .with_color(Color::from([0x2,0x2,0x12])),
                        Text::new("|").with_scale(100.0)
                        .with_color(Color::WHITE),
                        Text::new(text_input.text_after_curser()).with_scale(100.0)
                        .with_color(Color::from([0x2,0x2,0x12])),
                    ],
                    None, Some((width - 40.0, f32::INFINITY)),
                    Some(layout!(Wrap, Left, Top))
                );



                let trf =
                    flat_window_projection(width, height) *
                    // window_fov_projection(30.0, width, height) *
                    // Matrix4::from_translation((0.0, 0.0, 0.0).into()) *
                    // Matrix4::from_angle_z(Deg(45.0)) *
                    // Matrix4::from_angle_y(Deg(88.0)) *
                    // Matrix4::from_translation((-1200.0, 900.0, 0.0).into()) *
                    Matrix4::from_translation((20.0, 20.0, 0.0).into()) *
                    // Matrix4::from_angle_x(Deg(45.0)) *
                    Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0)
                ;


                target.with_encoder_frame(&gx, |encoder, attachment| {

                    encoder.render_pass(attachment, Some(Color::GREEN));

                    encoder.draw_glyphs(&gx, attachment, &mut glyphs, trf, None, Some(&mut staging_belt));

                    staging_belt.finish();

                }).expect("frame error");


                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}
