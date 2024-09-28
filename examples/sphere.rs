

use platform::winit::{
  window::WindowAttributes, event::{WindowEvent, KeyEvent, ElementState}, keyboard::PhysicalKey,
  dpi::PhysicalSize,
};
use platform::{*, time::*};
use wgx::{*, math::*};
use std::ops::Neg;

// common
#[path="common/world_view.rs"] #[allow(dead_code)]
mod world_view;
use world_view::*;


main_app_closure! {
  LogLevel::Warn,
  WindowAttributes::default().with_inner_size(PhysicalSize::new(1000, 1000)),
  init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, &AppEvent) {

  let window = ctx.window_clone();

  let msaa = 4;
  let depth_testing = Some(DEFAULT_DEPTH);
  let blending = None;
  let features = features!(POLYGON_MODE_LINE, MULTI_DRAW_INDIRECT);

  let (gx, mut target) = Wgx::new_with_target(window.clone(), features, limits!{}, window.inner_size(), msaa, depth_testing).await.unwrap();

  // pipeline
  let shader = gx.load_wgsl(wgsl_modules::include!("common/shaders/shader_3d_inst_text_diff.wgsl"));

  let pipeline = target.render_pipeline(&gx,
    None, &[
      vertex_dsc!(Vertex, 0 => Float32x3, 1 => Float32x3, 2 => Float32x3),
      vertex_dsc!(Instance, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4)
    ],
    (&shader, "vs_main", Primitive {
      cull_mode: None, // Some(Face::Back),
      polygon_mode: Polygon::Fill,
      ..Primitive::default()
    }),
    (&shader, "fs_main", blending),
  );

  // colors
  let color_texture = TextureLot::new_2d_with_data(&gx, [1, 1, 1], 1, DEFAULT_SRGB, None, TexUse::TEXTURE_BINDING, [255u8, 0, 0, 255]);
  let sampler = gx.default_sampler();


  // vertices

  use std::f32::consts::FRAC_PI_2;

  let steps = 12usize;
  let smooth = false;

  let mut mesh:Vec<[[f32;3];3]> = Vec::with_capacity(3 * steps * steps);

  let t_c = [1.0, 1.0, 0.0]; // texture coordinates

  let a_s = FRAC_PI_2 / steps as f32;

  // directions: v from +y down, h from -z to +x
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

      let a = [sin_v1*sin_s1h0, cos_v1, -sin_v1*cos_s1h0];
      let b = [sin_v1*sin_s1h1, cos_v1, -sin_v1*cos_s1h1];

      // v0 x s0
      let fi_s0h0 = h0 * a_s0;
      let fi_s0h1 = h1 * a_s0;

      let cos_s0h0 = f32::cos(fi_s0h0);
      let cos_s0h1 = f32::cos(fi_s0h1);

      let sin_s0h0 = f32::sin(fi_s0h0);
      let sin_s0h1 = f32::sin(fi_s0h1);

      let c = [sin_v0*sin_s0h0, cos_v0, -sin_v0*cos_s0h0];
      let d = [sin_v0*sin_s0h1, cos_v0, -sin_v0*cos_s0h1];

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
        let n = Vec3::normal_from_triangle(a.into(), b.into(), c.into()).neg().into();
        mesh.push([a, t_c, n]);
        mesh.push([b, t_c, n]);
        mesh.push([c, t_c, n]);

        if j < k {
          let n = Vec3::normal_from_triangle(b.into(), d.into(), c.into()).neg().into();
          mesh.push([b, t_c, n]);
          mesh.push([d, t_c, n]);
          mesh.push([c, t_c, n]);
        }
      }
    }
  }

  // println!("{:#?}", mesh);

  let instance_data = [
    Mat4::from_rotation_y(f32::to_radians(000.0)),
    Mat4::from_rotation_y(f32::to_radians(090.0)),
    Mat4::from_rotation_y(f32::to_radians(180.0)),
    Mat4::from_rotation_y(f32::to_radians(270.0)),
    Mat4::from_rotation_y(f32::to_radians(000.0))*Mat4::from_rotation_z(f32::to_radians(180.0)),
    Mat4::from_rotation_y(f32::to_radians(090.0))*Mat4::from_rotation_z(f32::to_radians(180.0)),
    Mat4::from_rotation_y(f32::to_radians(180.0))*Mat4::from_rotation_z(f32::to_radians(180.0)),
    Mat4::from_rotation_y(f32::to_radians(270.0))*Mat4::from_rotation_z(f32::to_radians(180.0)),
  ];


  // buffers
  let indirect_buffer = gx.buffer_from_data(BufUse::INDIRECT, [
    DrawIndirectArgs::try_from_ranges(0..mesh.len() as usize, 0..instance_data.len() as usize).unwrap(),
  ]);

  let vertex_buffer = gx.buffer_from_data(BufUse::VERTEX, &*mesh);
  let instance_buffer = gx.buffer_from_data(BufUse::VERTEX, instance_data);


  // world
  let [width, height] = window.inner_size().into();
  let mut world = WorldView::new(&gx, 10.0, 5.0, 0.1, FovProjection::window(45.0, width, height));

  world.objects = Mat4::from_uniform_scale(0.25 * height);
  world.calc_clip_matrix();

  let light_matrix = Mat4::from_rotation_x(f32::to_radians(-30.0));

  world.light_matrix = light_matrix * world.rotation; // keep light

  world.write_clip_buffer(&gx);
  world.write_light_buffer(&gx);


  // bind
  let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
    bind!(0, Buffer, world.clip_buffer),
    bind!(1, Buffer, world.light_buffer),
    bind!(2, TextureView, &color_texture.view),
    bind!(3, Sampler, &sampler),
  ]);

  // render bundles
  let bundles = [target.render_bundle(&gx, |rpass| {
    rpass.set_pipeline(&pipeline);
    rpass.set_bind_group(0, &binding, &[]);
    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
    rpass.set_vertex_buffer(1, instance_buffer.slice(..));
    rpass.draw_indirect(&indirect_buffer, 0);
  })];

  // event loop

  move |_ctx: &mut AppCtx, event: &AppEvent| match event {

    AppEvent::WindowEvent(WindowEvent::Resized(size)) => {
      target.update(&gx, *size);
      world.fov.resize_window(size.width as f32, size.height as f32, true);
      world.calc_clip_matrix();
      // world.light_matrix = light_matrix * world.rotation; // keep light
      world.write_clip_buffer(&gx);
      world.write_light_buffer(&gx);
    },

    AppEvent::WindowEvent(WindowEvent::KeyboardInput { event: KeyEvent {
      physical_key: PhysicalKey::Code(keycode), state: ElementState::Pressed, ..
    }, ..}) => {
      if let Some(key) = InputKey::match_keycode(*keycode) {
        world.input(key);
        world.calc_clip_matrix();
        // world.light_matrix = light_matrix * world.rotation; // keep light
        world.write_clip_buffer(&gx);
        world.write_light_buffer(&gx);
        window.request_redraw();
      }
    },

    AppEvent::WindowEvent(WindowEvent::RedrawRequested) => {

      let then = Instant::now();

      target.with_frame(None, |frame| gx.with_encoder(|encoder| {
        encoder.pass_bundles(frame.attachments(Some(Color::BLACK), Some(1.0), None), &bundles);
      })).expect("frame error");

      println!("{:?}", then.elapsed());
    },

    _ => {}
  }
}
