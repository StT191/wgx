
use std::{time::{Instant}};
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

  let (gx, surface) = unsafe {Wgx::new(Some(&window), Features::MULTI_DRAW_INDIRECT, limits!{})}.block_on().unwrap();
  let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (width, height), MSAA, DEPTH_TESTING).unwrap();


  // pipeline
  let shader = gx.load_wgsl(include_wgsl_module!("./shaders/v3d_inst_text_diff.wgsl"));


  // triangle pipeline
  let pipeline = target.render_pipeline(&gx,
    None, &[
      vertex_desc!(Vertex, 0 => Float32x3, 1 => Float32x3, 2 => Float32x3),
      vertex_desc!(Instance, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4)
    ],
    (&shader, "vs_main", Primitive::TriangleList),
    (&shader, "fs_main", ALPHA_BLENDING),
  );

  // colors
  let color_texture = TextureLot::new_2d_with_data(&gx, (1, 1), 1, TEXTURE, TexUse::TEXTURE_BINDING, [255u8, 0, 0, 255]);

  let sampler = gx.default_sampler();


  let clip_buffer = gx.buffer(BufUse::UNIFORM | BufUse::COPY_DST, 64, false);
  let light_buffer = gx.buffer(BufUse::UNIFORM | BufUse::COPY_DST, 64, false);

  let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
    bind!(0, Buffer, &clip_buffer),
    bind!(1, Buffer, &light_buffer),
    bind!(2, TextureView, &color_texture.view),
    bind!(3, Sampler, &sampler),
  ]);


  // vertexes
  use std::f32::consts::FRAC_PI_2;
  let steps = 24usize;
  let smooth = false;

  let mut mesh:Vec<[[f32;3];3]> = Vec::with_capacity(3 * steps * steps);

  let t_c = [1.0, 1.0, 0.0]; // texture coordinate

  let f_steps = steps as f32;
  let a_s = FRAC_PI_2 / f_steps;

  for k in 0..steps {

    let v0 = k as f32;
    let v1 = v0 + 1.0;

    let fi_v0 = v0 * a_s;
    let fi_v1 = v1 * a_s;

    let cos_v0 = f32::cos(fi_v0);
    let cos_v1 = f32::cos(fi_v1);

    let sin_v0 = f32::sin(fi_v0);
    let sin_v1 = f32::sin(fi_v1);

    let a_s0 = if v0 == 0.0 { 0.0 } else { FRAC_PI_2 / v0 };
    let a_s1 = FRAC_PI_2 / v1;

    for j in 0..(k + 1) {

      let h0 = j as f32;
      let h1 = h0 + 1.0;

      // v1 x s1
      let fi_s1h0 = h0 * a_s1;
      let fi_s1h1 = h1 * a_s1;

      let cos_s1h0 = f32::cos(fi_s1h0);
      let cos_s1h1 = f32::cos(fi_s1h1);

      let sin_s1h0 = f32::sin(fi_s1h0);
      let sin_s1h1 = f32::sin(fi_s1h1);

      let a = [sin_v1*cos_s1h0, sin_v1*sin_s1h0, cos_v1];
      let b = [sin_v1*cos_s1h1, sin_v1*sin_s1h1, cos_v1];

      // v0 x s0
      let fi_s0h0 = h0 * a_s0;
      let fi_s0h1 = h1 * a_s0;

      let cos_s0h0 = f32::cos(fi_s0h0);
      let cos_s0h1 = f32::cos(fi_s0h1);

      let sin_s0h0 = f32::sin(fi_s0h0);
      let sin_s0h1 = f32::sin(fi_s0h1);

      let c = [sin_v0*cos_s0h0, sin_v0*sin_s0h0, cos_v0];
      let d = [sin_v0*cos_s0h1, sin_v0*sin_s0h1, cos_v0];

      if smooth {
        mesh.push([a, t_c, a]);
        mesh.push([b, t_c, b]);
        mesh.push([c, t_c, c]);

        if j < k {
          mesh.push([b, t_c, b]);
          mesh.push([d, t_c, d]);
          mesh.push([c, t_c, c]);
        }
      }
      else {
        let n = normal_from_triangle(a, b, c).into();
        mesh.push([a, t_c, n]);
        mesh.push([b, t_c, n]);
        mesh.push([c, t_c, n]);

        if j < k {
          let n = normal_from_triangle(b, d, c).into();
          mesh.push([b, t_c, n]);
          mesh.push([d, t_c, n]);
          mesh.push([c, t_c, n]);
        }
      }
    }
  }

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


  let mut group = MultiDrawIndirect::new(&gx, (None, mesh.len()), (None, 2*instance_data.len()), (None, 1));

  let mesh_range = group.vertices.write_multiple(None, &mesh);
  let instance_range = group.instances.write_multiple(None, &instance_data);

  group.indirect.write(None, &DrawIndirect::try_from_ranges(mesh_range, instance_range).unwrap());

  group.write_buffers(&gx, .., .., ..);


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
    Matrix4::from_scale(0.25 * height)
    // Matrix4::from_translation((0.0, -0.7 * height, 0.0).into())
  ;

  // let light_matrix = Matrix4::<f32>::from_angle_x(Deg(-45.0)) * Matrix4::from_angle_y(Deg(45.0));
  let light_matrix = Matrix4::<f32>::identity();

  // let clip_matrix = projection * rot_matrix * Matrix4::from_nonuniform_scale(w*width, h*height, 1.0);

  let mut rot_matrix = Matrix4::identity();
  let mut world_matrix = Matrix4::identity();

  let clip_matrix = projection * obj_mat;

  // gx.write_buffer(&world_buffer, 0, AsRef::<[f32; 16]>::as_ref(&world_matrix));
  gx.write_buffer(&clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
  gx.write_buffer(&light_buffer, 0, AsRef::<[f32; 16]>::as_ref(&(light_matrix)));
  // gx.write_buffer(&viewport_buffer, 0, &[width, height]);


  // event loop
  event_loop.run(move |event, _, control_flow| {

    *control_flow = ControlFlow::Wait;

    match event {
      Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
        *control_flow = ControlFlow::Exit;
      },

      Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
        target.update(&gx, (size.width, size.height));

        // width = size.width as f32;
        // height = size.height as f32;

        let fov = FovProjection::window(fov_deg, width, height);
        projection = fov.projection * fov.translation;

        // projection
        let clip_matrix = projection * rot_matrix * world_matrix * obj_mat;

        gx.write_buffer(&clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
        // gx.write_buffer(&&viewport_buffer, 0, &[width, height]);
      },

      Event::WindowEvent { event:WindowEvent::KeyboardInput { input: KeyboardInput {
        virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
      }, ..}, ..} => {
        let mut redraw = true;
        match keycode {

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
          },

          _ => { redraw = false; }
        } {
          if redraw {

            let clip_matrix = projection * rot_matrix * world_matrix * obj_mat;
            // let light_matrix = rot_matrix * light_matrix;
            let light_matrix = light_matrix;

            gx.write_buffer(&clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
            gx.write_buffer(&light_buffer, 0, AsRef::<[f32; 16]>::as_ref(&light_matrix));

            window.request_redraw();
          }
        }
      },

      Event::RedrawRequested(_) => {

        let then = Instant::now();

        target.with_encoder_frame(&gx, |encoder, frame| {
          encoder.with_render_pass(frame.attachments(Some(Color::BLACK), Some(1.0)), |mut rpass| {
            rpass.set_pipeline(&pipeline);
            rpass.set_bind_group(0, &binding, &[]);
            rpass.set_vertex_buffer(0, group.vertices.buffer.slice(..));
            rpass.set_vertex_buffer(1, group.instances.buffer.slice(..));
            rpass.multi_draw_indirect(&group.indirect.buffer, 0, group.indirect.len() as u32);
          });
        }).expect("frame error");

        println!("{:?}", then.elapsed());
      },

      _ => {}
    }
  });
}
