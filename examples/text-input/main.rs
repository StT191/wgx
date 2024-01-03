
// imports
use std::{time::{Instant}};
use pollster::FutureExt;
use winit::{
    event_loop::{ControlFlow, EventLoop}, dpi::PhysicalSize,
    window::Window, event::{Event, WindowEvent, KeyEvent, ElementState},
    keyboard::{PhysicalKey, KeyCode},
};
use wgx::{*, cgmath::*};


#[allow(dead_code)]
mod text_input;
use text_input::*;


// main
fn main() {

    const DEPTH_TESTING:bool = false;
    // const BLENDING:Option<Blend> = None;
    const MSAA:u32 = 1;


    // use futures::task::SpawnExt;
    // let mut local_pool = futures::executor::LocalPool::new();


    // window setup
    let event_loop = EventLoop::new().unwrap();

    let window = Window::new(&event_loop).unwrap();
    let _ = window.request_inner_size(PhysicalSize::<u32>::from((1200u32, 1000u32)));
    window.set_title("WgFx");


    // wgx setup
    let (gx, surface) = unsafe {Wgx::new(Some(&window), features!(), limits!{})}.block_on().unwrap();
    let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (1200, 1000), MSAA, DEPTH_TESTING).unwrap();


    // text_render
    let font_data = include_bytes!("./fonts/caladea.ttf").to_vec();
    // let font_data = include_bytes!("./fonts/font_active.ttf").to_vec();

    // let mut glyphs = gx.glyph_brush_with_depth(target.format(), font_data).expect("invalid font");
    let mut glyphs = gx.glyph_brush(target.format(), font_data).expect("invalid font");

    let mut text_input = SimpleTextInput::new("Hey Ho!\nWhat is going on? Anyway?\n");
    text_input.set_curser_end();


    let mut staging_belt = wgpu::util::StagingBelt::new(10240);


    event_loop.run(move |event, event_target| {

        event_target.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                event_target.exit();
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                target.update(&gx, (size.width, size.height));
            },

            Event::WindowEvent { event: WindowEvent::KeyboardInput { event: KeyEvent {
                physical_key, text, state: ElementState::Pressed, ..
            }, ..}, ..} => {

                let mut redraw = false;

                if let Some(letters) = text {
                    for character in letters.chars() {
                        if text_input.insert(character) {
                            redraw = true;
                        }
                    }
                }

                if let PhysicalKey::Code(keycode) = physical_key {
                    redraw |= match keycode {
                        KeyCode::ArrowLeft => text_input.recede(),
                        KeyCode::ArrowRight => text_input.advance(),
                        KeyCode::ArrowUp => text_input.recede_line(),
                        KeyCode::ArrowDown => text_input.advance_line(),
                        KeyCode::Backspace => text_input.remove(),
                        KeyCode::Delete => text_input.delete(),
                        KeyCode::Home => text_input.set_curser(0),
                        KeyCode::End => text_input.set_curser_end(),
                        KeyCode::Enter => text_input.insert('\n'),
                        _ => false
                    }
                }

                if redraw { window.request_redraw() }
            },

            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {

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
                        Text::new(text_input.text_before_curser()).with_scale(font_size)
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

                target.with_frame(None, |frame| gx.with_encoder(|encoder| {

                    encoder.render_pass(frame.attachments(Some(Color::WHITE), Some(1.0)));

                    encoder.draw_glyphs( /*_with_depth(*/
                        &gx, frame, /*frame.depth_attachment(None).ok_or("depth attachment missing")?,*/
                        &mut glyphs, trf, None, &mut staging_belt
                    ).expect("glyphs error");

                    staging_belt.finish();

                })).expect("frame error");

                staging_belt.recall();


                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    }).unwrap();
}
