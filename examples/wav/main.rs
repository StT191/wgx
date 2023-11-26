
// use std::{time::{Instant}};
use pollster::FutureExt;
use winit::{
  dpi::PhysicalSize,
  event_loop::{ControlFlow, EventLoop},
  window::Window, event::{Event, WindowEvent, KeyboardInput, ElementState},
};
use wgx::{*, cgmath::*};

// common
#[path="../common/world_view.rs"] #[allow(dead_code)]
mod world_view;
use world_view::{WorldView, InputKey};

mod wav_obj;


fn main() {

  const DEPTH_TESTING:bool = true;
  const MSAA:u32 = 4;
  const BLENDING:Option<Blend> = None;


  let (width, height) = (1000, 1000);

  let event_loop = EventLoop::new();
  let window = Window::new(&event_loop).unwrap();
  window.set_inner_size(PhysicalSize::<u32>::from((width, height)));
  window.set_title("WgFx");

  let (gx, surface) = unsafe {Wgx::new(Some(&window), Features::empty(), limits!{})}.block_on().unwrap();
  let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (width, height), MSAA, DEPTH_TESTING).unwrap();


  // pipeline
  let shader = gx.load_wgsl(include_wgsl_module!("../common/shaders/shader_3d_text_diff.wgsl"));


  // triangle pipeline
  let pipeline = target.render_pipeline(&gx,
    None, &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x3, 2 => Float32x3)],
    (&shader, "vs_main", Primitive::default()),
    (&shader, "fs_main", BLENDING),
  );


  // colors
  let color_texture = TextureLot::new_2d_with_data(&gx, (1, 1), 1, DEFAULT_SRGB, None, TexUse::TEXTURE_BINDING, [255u8, 0, 0, 255]);

  let sampler = gx.default_sampler();


  // buffers and binding
  let (width, height) = (width as f32, height as f32);
  let mut world = WorldView::new(&gx, 10.0, 5.0, 0.1, FovProjection::window(45.0, width, height));

  let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
    bind!(0, Buffer, world.clip_buffer),
    bind!(1, Buffer, world.light_buffer),
    bind!(2, TextureView, &color_texture.view),
    bind!(3, Sampler, &sampler),
  ]);


  let triangles = wav_obj::parse(include_str!("./obj/deer.obj")).expect("couldn't parse wav obj");

  let vertex_buffer = gx.buffer_from_data(BufUse::VERTEX, &triangles);


  // render bundles
  let bundles = [target.render_bundle(&gx, |rpass| {
    rpass.set_pipeline(&pipeline);
    rpass.set_bind_group(0, &binding, &[]);
    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
    rpass.draw(0..triangles.len() as u32 * 3, 0..1);
  })];


  // world
  world.objects = // Matrix4::identity()
    Matrix4::from_angle_y(Deg(100.0)) *
    Matrix4::from_scale(0.55) *
    Matrix4::from_translation((0.0, -0.7 * height, 0.0).into())
  ;

  world.calc_clip_matrix();

  world.write_light_buffer(&gx);
  world.write_clip_buffer(&gx);


  // event loop
  event_loop.run(move |event, _, control_flow| {

    *control_flow = ControlFlow::Wait;

    match event {
      Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
        *control_flow = ControlFlow::Exit;
      },

      Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
        target.update(&gx, (size.width, size.height));
        world.fov.resize_window(size.width as f32, size.height as f32, true);
        world.calc_clip_matrix();
        world.write_clip_buffer(&gx);
      },

      Event::WindowEvent { event:WindowEvent::KeyboardInput { input: KeyboardInput {
        virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
      }, ..}, ..} => {
        if let Some(key) = InputKey::match_keycode(keycode) {
          world.input(key);

          world.calc_clip_matrix();
          // world.light_matrix = world.rotation;

          world.write_light_buffer(&gx);
          world.write_clip_buffer(&gx);

          window.request_redraw();
        }
      },

      Event::RedrawRequested(_) => {

        // let then = Instant::now();

        target.with_encoder_frame(&gx, |encoder, frame| {
          encoder.render_bundles(frame.attachments(Some(Color::BLACK), Some(1.0)), &bundles);
        }).expect("frame error");

        // println!("{:?}", then.elapsed());
      },

      _ => {}
    }
  });
}
