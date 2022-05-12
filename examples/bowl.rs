#![allow(unused)]

use std::{time::{Instant}};
use futures::executor::block_on;
use winit::{
  dpi::PhysicalSize,
  event_loop::{ControlFlow, EventLoop},
  window::Window, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
};
use wgx::{*, cgmath::*};


fn main() {

  const DEPTH_TESTING:bool = true;
  const MSAA:u32 = 4;
  const ALPHA_BLENDING:bool = false;


  let (width, height) = (1000, 1000);

  let event_loop = EventLoop::new();
  let window = Window::new(&event_loop).unwrap();
  window.set_inner_size(PhysicalSize::<u32>::from((width, height)));
  window.set_title("WgFx");

  let mut gx = block_on(Wgx::new(Some(&window), Features::empty(), limits!{})).unwrap();
  let mut target = gx.surface_target((width, height), DEPTH_TESTING, MSAA).unwrap();


  // pipeline
  let shader = gx.load_wgsl(include_str!("../shaders/standard_instance_texture.wgsl"));


  // triangle pipeline
  let pipeline = target.render_pipeline(
    &gx, ALPHA_BLENDING, (&shader, "vs_main"), (&shader, "fs_main"),
    &[
      vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x3, 2 => Float32x3),
      vertex_desc!(Instance, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4)
    ],
    Primitive::TriangleList, None
  );


  // colors
  let color_texture = gx.texture((1, 1), 1, TexUse::TEXTURE_BINDING | TexUse::COPY_DST, TEXTURE);
  gx.write_texture(&color_texture, (0, 0, 1, 1), &[
    [255u8, 0, 0, 255], //[0, 255, 0, 255], [0, 0, 255, 255],
  ]);
  let color_texture_view = color_texture.create_default_view();

  let sampler = gx.sampler();


  // image
  // let img = image::open("img/logo_red.png")
  //     .expect("failed loading image")
  //     .into_rgba8();

  // let (w, h) = (img.width(), img.height());

  // let image_texture = gx.texture((w, h), 1, TexUse::TEXTURE_BINDING | TexUse::COPY_DST, TEXTURE);

  // gx.write_texture(&image_texture, (0, 0, w, h), &img.as_raw().as_slice());


  let mut clip_buffer = gx.buffer(BuffUse::UNIFORM | BuffUse::COPY_DST, 64, false);
  let mut light_buffer = gx.buffer(BuffUse::UNIFORM | BuffUse::COPY_DST, 64, false);

  let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
    bind!(0, Buffer, &clip_buffer),
    bind!(1, Buffer, &light_buffer),
    bind!(2, TextureView, &color_texture_view),
    bind!(3, Sampler, &sampler),
  ]);


  // vertexes

  let steps = 64usize;
  let smooth = true;

  let step_a = steps as f32 / std::f32::consts::FRAC_PI_2; // step angle

  let mut vertex_data:Vec<[[f32;3];3]> = Vec::with_capacity(2 * 2 * 3 * steps * steps);

  let t_c = [1.0, 1.0, 0.0];


  for k in 0..steps {

    let fi_a0 = k as f32 / step_a;
    let fi_a1 = (k as f32 + 1.0) / step_a;

    let cos_a0 = f32::cos(fi_a0);
    let cos_a1 = f32::cos(fi_a1);

    let sin_a0 = f32::sin(fi_a0);
    let sin_a1 = f32::sin(fi_a1);

    for j in 0..steps {

      let fi_b0 = j as f32 / step_a;
      let fi_b1 = (j as f32 + 1.0) / step_a;

      let cos_b0 = f32::cos(fi_b0);
      let cos_b1 = f32::cos(fi_b1);

      let sin_b0 = f32::sin(fi_b0);
      let sin_b1 = f32::sin(fi_b1);

      let a = [cos_a0*sin_b0, sin_a0, cos_a0*cos_b0];
      let b = [cos_a1*sin_b0, sin_a1, cos_a1*cos_b0];

      let c = [cos_a1*sin_b1, sin_a1, cos_a1*cos_b1];
      let d = [cos_a0*sin_b1, sin_a0, cos_a0*cos_b1];

      if (smooth) {
        vertex_data.push([a, t_c, a]);
        vertex_data.push([d, t_c, d]);
        vertex_data.push([c, t_c, c]);

        vertex_data.push([a, t_c, a]);
        vertex_data.push([c, t_c, c]);
        vertex_data.push([b, t_c, b]);
      }
      else {
        let n = normal_from_triangle(a, d, c).into();

        vertex_data.push([a, t_c, n]);
        vertex_data.push([d, t_c, n]);
        vertex_data.push([c, t_c, n]);

        vertex_data.push([a, t_c, n]);
        vertex_data.push([c, t_c, n]);
        vertex_data.push([b, t_c, n]);
      }
    }
  }


  let vertex_buffer = gx.buffer_from_data(BuffUse::VERTEX, &vertex_data);

  // let triangles = wav_obj::parse(include_str!("../obj/deer.obj")).expect("couldn't parse wav obj");
  // let vertex_buffer = gx.buffer_from_data(BuffUse::VERTEX, &triangles);


  // let instance_data = [
  //   Matrix4::<f32>::from_angle_x(Deg(0.0)),
  //   Matrix4::<f32>::from_angle_x(Deg(90.0)),
  //   Matrix4::<f32>::from_angle_x(Deg(180.0)),
  //   Matrix4::<f32>::from_angle_x(Deg(270.0)),
  //   Matrix4::<f32>::from_angle_x(Deg(0.0))   * Matrix4::from_angle_y(Deg(180.0)),
  //   Matrix4::<f32>::from_angle_x(Deg(90.0))  * Matrix4::from_angle_y(Deg(180.0)),
  //   Matrix4::<f32>::from_angle_x(Deg(180.0)) * Matrix4::from_angle_y(Deg(180.0)),
  //   Matrix4::<f32>::from_angle_x(Deg(270.0)) * Matrix4::from_angle_y(Deg(180.0)),
  // ];

  let instance_data = [
    Matrix4::<f32>::from_nonuniform_scale( 1.0, 1.0, 1.0),
    Matrix4::<f32>::from_nonuniform_scale(-1.0, 1.0, 1.0),
    Matrix4::<f32>::from_nonuniform_scale( 1.0,-1.0, 1.0),
    Matrix4::<f32>::from_nonuniform_scale(-1.0,-1.0, 1.0),
    Matrix4::<f32>::from_nonuniform_scale( 1.0, 1.0,-1.0),
    Matrix4::<f32>::from_nonuniform_scale(-1.0, 1.0,-1.0),
    Matrix4::<f32>::from_nonuniform_scale( 1.0,-1.0,-1.0),
    Matrix4::<f32>::from_nonuniform_scale(-1.0,-1.0,-1.0),
  ];

  let instance_buffer = gx.buffer_from_data(BuffUse::VERTEX, &instance_data);


  // render bundles
  let bundles = [target.render_bundle(&gx, |rpass| {
    rpass.set_pipeline(&pipeline);
    rpass.set_bind_group(0, &binding, &[]);
    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
    rpass.set_vertex_buffer(1, instance_buffer.slice(..));
    rpass.draw(0..vertex_data.len() as u32, 0..instance_data.len() as u32);
    // rpass.draw(0..triangles.len() as u32 * 3, 0..1);
  })];


  // matrix
  const DA:f32 = 5.0;
  const DS:f32 = 50.0;

  let fov_deg = 45.0;

  let (mut width, mut height) = (width as f32, height as f32);

  // let mut scale = 1.0;
  // let (mut w, mut h) = (0.4, 0.4);

  let fov = FovProjection::window(fov_deg, width, height);
  let mut projection = fov.projection * fov.translation;

  let camera_correction = fov.translation;

  let obj_mat =
    // Matrix4::identity()
    Matrix4::from_scale(0.25 * height)
    // Matrix4::from_translation((0.0, -0.7 * height, 0.0).into())
  ;

  // let light_matrix = Matrix4::<f32>::from_angle_x(Deg(-45.0)) * Matrix4::from_angle_y(Deg(45.0));
  let light_matrix = Matrix4::<f32>::identity();

  // let clip_matrix = projection * rot_matrix * Matrix4::from_nonuniform_scale(w*width, h*height, 1.0);

  let mut rot_matrix = Matrix4::identity();
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

        width = size.width as f32;
        height = size.height as f32;

        let fov = FovProjection::window(fov_deg, width, height);
        projection = fov.projection * fov.translation;

        // projection
        let clip_matrix = projection * rot_matrix * world_matrix * obj_mat;

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

          VirtualKeyCode::I => { apply!(rot_matrix, Matrix4::from_angle_x(Deg( DA))); },
          VirtualKeyCode::K => { apply!(rot_matrix, Matrix4::from_angle_x(Deg(-DA))); },
          VirtualKeyCode::J => { apply!(rot_matrix, Matrix4::from_angle_y(Deg( DA))); },
          VirtualKeyCode::L => { apply!(rot_matrix, Matrix4::from_angle_y(Deg(-DA))); },
          VirtualKeyCode::U => { apply!(rot_matrix, Matrix4::from_angle_z(Deg( DA))); },
          VirtualKeyCode::O => { apply!(rot_matrix, Matrix4::from_angle_z(Deg(-DA))); },

          VirtualKeyCode::A => { apply!(world_matrix, Matrix4::from_translation((-DS, 0.0, 0.0).into())); },
          VirtualKeyCode::D => { apply!(world_matrix, Matrix4::from_translation(( DS, 0.0, 0.0).into())); },
          VirtualKeyCode::W => { apply!(world_matrix, Matrix4::from_translation((0.0, 0.0,  DS).into())); },
          VirtualKeyCode::S => { apply!(world_matrix, Matrix4::from_translation((0.0, 0.0, -DS).into())); },
          VirtualKeyCode::Q => { apply!(world_matrix, Matrix4::from_translation((0.0, -DS, 0.0).into())); },
          VirtualKeyCode::E => { apply!(world_matrix, Matrix4::from_translation((0.0,  DS, 0.0).into())); },

          VirtualKeyCode::Y => { apply!(world_matrix, Matrix4::from_scale(0.9)); },
          VirtualKeyCode::X => { apply!(world_matrix, Matrix4::from_scale(1.1)); },

          VirtualKeyCode::R => {
            rot_matrix = Matrix4::identity();
            world_matrix = Matrix4::identity();
            // scale = 1.0;
            // w = 0.4;
            // h = 0.4;
          },

          _ => { redraw = false; }
        } {
          if redraw {

            let clip_matrix = projection * rot_matrix * world_matrix * obj_mat;
            // let light_matrix = rot_matrix * light_matrix;
            let light_matrix = light_matrix;

            gx.write_buffer(&mut clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
            gx.write_buffer(&mut light_buffer, 0, AsRef::<[f32; 16]>::as_ref(&light_matrix));

            window.request_redraw();
          }
        }
      },

      Event::RedrawRequested(_) => {

        let then = Instant::now();

        target.with_encoder_frame(&gx, |encoder, attachment| {
          encoder.render_bundles(attachment, Some(Color::BLACK), &bundles);
        }).expect("frame error");

        println!("{:?}", then.elapsed());
      },

      _ => {}
    }
  });
}
