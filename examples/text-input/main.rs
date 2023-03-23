#![allow(dead_code)]

// imports
use std::{time::{Instant}};
use pollster::FutureExt;
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::*,
};
use wgx::{*, cgmath::*};

mod text_input;
use text_input::*;


// main
fn main() {

    const DEPTH_TESTING:bool = false;
    const ALPHA_BLENDING:Option<BlendState> = None;
    const MSAA:u32 = 1;


    // use futures::task::SpawnExt;
    // let mut local_pool = futures::executor::LocalPool::new();


    // window setup
    let event_loop = EventLoop::new();


    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((1200u32, 1000u32)));
    window.set_title("WgFx");


    // wgx setup
    let (gx, surface) = unsafe {Wgx::new(Some(&window), Features::empty(), limits!{})}.block_on().unwrap();
    let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (1200, 1000), MSAA, DEPTH_TESTING).unwrap();


    // text_render
    let font_data = include_bytes!("./fonts/caladea.ttf").to_vec();
    // let font_data = include_bytes!("./fonts/font_active.ttf").to_vec();

    // let mut glyphs = gx.glyph_brush_with_depth(target.format(), font_data).expect("invalid font");
    let mut glyphs = gx.glyph_brush(target.format(), font_data).expect("invalid font");

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

                // let scale_factor = window.scale_factor() as f32; // gives better font-clarity
                // let scale_factor = window.scale_factor() as f32 * 4.0/3.0; // gives better font-clarity
                let scale_factor = 1.0; // gives better font-clarity
                // let scale_factor = 1.0; // gives better font-clarity

                let font_size = 20.0 * window.scale_factor() as f32 * scale_factor;
                let color = Color::from([0x02,0x02,0x12]);
                // let cursor_color = Color::LIGHT_GREY;
                let cursor_color = Color::LIGHT_GREY;

                glyphs.add_text(
                    vec![
                        Text::new(&text_input.text_before_curser()).with_scale(font_size)
                        .with_color(color),
                        Text::new("|").with_scale(font_size)
                        .with_color(cursor_color),
                        Text::new(text_input.text_after_curser()).with_scale(font_size)
                        .with_color(color),
                    ],
                    None, Some(((width - 40.0) * scale_factor, f32::INFINITY)),
                    Some(layout!(Wrap, Left, Top))
                );

                let trf =
                    flat_window_projection(width, height, 0.0) *
                    // window_fov_projection(30.0, width, height) *
                    // Matrix4::from_translation((0.0, 0.0, 0.0).into()) *
                    // Matrix4::from_angle_z(Deg(45.0)) *
                    // Matrix4::from_angle_y(Deg(88.0)) *
                    // Matrix4::from_translation((-1200.0, 900.0, 0.0).into()) *
                    Matrix4::from_translation((20.0, 20.0, 0.0).into()) *
                    Matrix4::from_scale(1.0/scale_factor)
                    // Matrix4::from_angle_x(Deg(45.0)) *
                ;

                target.with_encoder_frame(&gx, |encoder, frame| {

                    encoder.render_pass(frame.attachments(Some(Color::WHITE), Some(1.0)));

                    encoder.draw_glyphs( /*_with_depth(*/
                        &gx, frame, /*frame.depth_attachment(None).ok_or("depth attachment missing")?,*/
                        &mut glyphs, trf, None, &mut staging_belt
                    ).expect("glyphs error");

                    staging_belt.finish();

                }).expect("frame error");

                staging_belt.recall();


                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}
