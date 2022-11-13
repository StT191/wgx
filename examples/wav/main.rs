
// use std::{time::{Instant}};
use pollster::FutureExt;
use winit::{
  dpi::PhysicalSize,
  event_loop::{ControlFlow, EventLoop},
  window::Window, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
};
use wgx::{*, cgmath::*};


fn main() {

  const DEPTH_TESTING:bool = true;
  const MSAA:u32 = 4;
  const ALPHA_BLENDING:Option<BlendState> = None;


  let (width, height) = (1000, 1000);

  let event_loop = EventLoop::new();
  let window = Window::new(&event_loop).unwrap();
  window.set_inner_size(PhysicalSize::<u32>::from((width, height)));
  window.set_title("WgFx");

  let (gx, surface) = Wgx::new(Some(&window), Features::empty(), limits!{}).block_on().unwrap();
  let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (width, height), MSAA, DEPTH_TESTING).unwrap();


  // pipeline
  let shader = gx.load_wgsl(include_wgsl_module!("../shaders/v3d_text_diff.wgsl"));


  // triangle pipeline
  let pipeline = target.render_pipeline(&gx,
    None, &[vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x3, 2 => Float32x3)],
    (&shader, "vs_main", Primitive::TriangleList),
    (&shader, "fs_main", ALPHA_BLENDING),
  );


  // colors
  let color_texture = TextureLot::new_2d_with_data(&gx, (1, 1), 1, TEXTURE, TexUse::TEXTURE_BINDING, [255u8, 0, 0, 255]);

  let sampler = gx.default_sampler();


  // buffers and binding
  let mut clip_buffer = gx.buffer(BufUse::UNIFORM | BufUse::COPY_DST, 64, false);
  let mut light_buffer = gx.buffer(BufUse::UNIFORM | BufUse::COPY_DST, 64, false);

  let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
    bind!(0, Buffer, &clip_buffer),
    bind!(1, Buffer, &light_buffer),
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



  // matrix
  const DA:f32 = 5.0;
  const DS:f32 = 50.0;

  let fov_deg = 45.0;

  let (width, height) = (width as f32, height as f32);

  // let mut scale = 1.0;
  // let (mut w, mut h) = (0.4, 0.4);

  let fov = FovProjection::window(fov_deg, width, height);
  let mut projection = fov.projection * fov.translation;

  // let camera_correction = fov.translation;

  let obj_mat =
    // Matrix4::identity()
    Matrix4::from_scale(0.55) *
    Matrix4::from_translation((0.0, -0.7 * height, 0.0).into())
  ;

  let light_matrix = Matrix4::<f32>::from_angle_x(Deg(-45.0)) * Matrix4::from_angle_y(Deg(45.0));

  // let clip_matrix = projection * rot_matrix * Matrix4::from_nonuniform_scale(w*width, h*height, 1.0);

  let mut rot_matrix_x = Matrix4::identity();
  let mut rot_matrix_y = Matrix4::identity();
  let mut rot_matrix_z = Matrix4::identity();
  let mut world_matrix = Matrix4::identity();

  let clip_matrix = projection * obj_mat;

  // gx.write_buffer(&mut world_buffer, 0, AsRef::<[f32; 16]>::as_ref(&world_matrix));
  gx.write_buffer(&mut clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
  gx.write_buffer(&mut light_buffer, 0, AsRef::<[f32; 16]>::as_ref(&(light_matrix)));
  // gx.write_buffer(&mut viewport_buffer, 0, &[width, height]);


  // event loop
  event_loop.run(move |event, _, control_flow| {

    *control_flow = ControlFlow::Wait;

    match event {
      Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
        *control_flow = ControlFlow::Exit;
      },

      Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
        target.update(&gx, (size.width, size.height));

        let width = size.width as f32;
        let height = size.height as f32;

        let fov = FovProjection::window(fov_deg, width, height);
        projection = fov.projection * fov.translation;

        // projection
        let clip_matrix = projection * rot_matrix_x * rot_matrix_y * rot_matrix_z * world_matrix * obj_mat;

        gx.write_buffer(&mut clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
        // gx.write_buffer(&mut viewport_buffer, 0, &[width, height]);
      },

      Event::WindowEvent { event:WindowEvent::KeyboardInput { input: KeyboardInput {
        virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
      }, ..}, ..} => {
        let mut redraw = true;
        match keycode {

          // VirtualKeyCode::I => { apply!(world_matrix, within(&camera_correction, &Matrix4::from_angle_x(Deg(-DA))).expect("no inversion")); },
          // VirtualKeyCode::K => { apply!(world_matrix, within(&camera_correction, &Matrix4::from_angle_x(Deg( DA))).expect("no inversion")); },
          // VirtualKeyCode::J => { apply!(world_matrix, within(&camera_correction, &Matrix4::from_angle_y(Deg( DA))).expect("no inversion")); },
          // VirtualKeyCode::L => { apply!(world_matrix, within(&camera_correction, &Matrix4::from_angle_y(Deg(-DA))).expect("no inversion")); },
          // VirtualKeyCode::U => { apply!(world_matrix, within(&camera_correction, &Matrix4::from_angle_z(Deg( DA))).expect("no inversion")); },
          // VirtualKeyCode::O => { apply!(world_matrix, within(&camera_correction, &Matrix4::from_angle_z(Deg(-DA))).expect("no inversion")); },

          VirtualKeyCode::I => { apply!(rot_matrix_x, Matrix4::from_angle_x(Deg( DA))); },
          VirtualKeyCode::K => { apply!(rot_matrix_x, Matrix4::from_angle_x(Deg(-DA))); },
          VirtualKeyCode::J => { apply!(rot_matrix_y, Matrix4::from_angle_y(Deg( DA))); },
          VirtualKeyCode::L => { apply!(rot_matrix_y, Matrix4::from_angle_y(Deg(-DA))); },
          VirtualKeyCode::U => { apply!(rot_matrix_z, Matrix4::from_angle_z(Deg( DA))); },
          VirtualKeyCode::O => { apply!(rot_matrix_z, Matrix4::from_angle_z(Deg(-DA))); },

          VirtualKeyCode::A => { apply!(world_matrix, Matrix4::from_translation((-DS, 0.0, 0.0).into())); },
          VirtualKeyCode::D => { apply!(world_matrix, Matrix4::from_translation(( DS, 0.0, 0.0).into())); },
          VirtualKeyCode::W => { apply!(world_matrix, Matrix4::from_translation((0.0, 0.0,  DS).into())); },
          VirtualKeyCode::S => { apply!(world_matrix, Matrix4::from_translation((0.0, 0.0, -DS).into())); },
          VirtualKeyCode::Q => { apply!(world_matrix, Matrix4::from_translation((0.0, -DS, 0.0).into())); },
          VirtualKeyCode::E => { apply!(world_matrix, Matrix4::from_translation((0.0,  DS, 0.0).into())); },

          VirtualKeyCode::Y => { apply!(world_matrix, Matrix4::from_scale(0.9)); },
          VirtualKeyCode::X => { apply!(world_matrix, Matrix4::from_scale(1.1)); },

          VirtualKeyCode::R => {
            rot_matrix_x = Matrix4::identity();
            rot_matrix_y = Matrix4::identity();
            rot_matrix_z = Matrix4::identity();
            world_matrix = Matrix4::identity();
            // scale = 1.0;
            // w = 0.4;
            // h = 0.4;
          },

          _ => { redraw = false; }
        } {
          if redraw {

            let clip_matrix = projection * rot_matrix_x * rot_matrix_y * rot_matrix_z * world_matrix * obj_mat;
            let light_matrix = rot_matrix_x * rot_matrix_y * rot_matrix_z * light_matrix;

            gx.write_buffer(&mut clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
            gx.write_buffer(&mut light_buffer, 0, AsRef::<[f32; 16]>::as_ref(&light_matrix));

            window.request_redraw();
          }
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
