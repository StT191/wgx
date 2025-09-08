
use platform::winit::{
  window::WindowAttributes, event::{WindowEvent, KeyEvent, ElementState}, keyboard::PhysicalKey,
  dpi::PhysicalSize,
};
use platform::*;
use wgx::{*, math::*};

// common
#[path="common/world_view.rs"] #[allow(dead_code)]
mod world_view;
use world_view::*;


main_app_closure! {
  LogLevel::Warn,
  WindowAttributes::default()
    .with_inner_size(PhysicalSize::new(1000, 1000))
    .with_transparent(true)
  ,
  init_app,
}

async fn init_app(ctx: &mut AppCtx) -> impl FnMut(&mut AppCtx, Event) + use<> {

  let window = ctx.window_clone();

  let srgb = true;
  let msaa = 4;
  let depth_testing = Some(TexFmt::Depth32Float);
  let blending = None;
  let features = features!(/*MAPPABLE_PRIMARY_BUFFERS*/);

  let (gx, mut target) = Wgx::new_with_target(window.clone(), features, limits!{}, window.inner_size(), srgb, msaa, depth_testing).await.unwrap();

  // pipeline
  let shader = gx.load_wgsl(wgsl_modules::include!("common/shaders/shader_3d_inst_text_diff.wgsl"));

  let constants = shader_constants!{LL_m: 0.01, LL_ml: 0.06};

  let pipeline = RenderPipelineConfig::new(
      &[
        vertex_dsc!(Vertex, 0 => Float32x4), vertex_dsc!(Vertex, 1 => Float32x2), vertex_dsc!(Vertex, 2 => Float32x4),
        vertex_dsc!(Instance, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4),
      ],
      &shader, "vs_main", Primitive {
        cull_mode: None, // Some(Face::Back),
        polygon_mode: Polygon::Fill,
        ..Primitive::default()
      },
    )
    .fragment(&shader, "fs_main").fragment_shader_constants(&constants)
    .render_target::<1>(&target, blending, Default::default())
    .pipeline(&gx)
  ;

  // colors
  let bg_color = Color::from([0x00, 0x00, 0x00, 0xCC]);

  let color_texture = TextureLot::new_2d_with_data(&gx, [1, 1, 1], 1, TexFmt::Rgba8UnormSrgb, None, TexUse::TEXTURE_BINDING, [255u8, 0, 0, 255]);
  let sampler = gx.sampler(&std_sampler_descriptor());

  // compute vertices

  let steps = 8u32;

  let row_len = 2 * 3 * steps; // faces * verts * steps
  let row_num = 3 * steps; // dirs * steps

  let mesh_len = row_len * row_num;

  let vertex_size = TexFmt::Rgba32Float.block_copy_size(None).unwrap();


  let mesh_size = vertex_size as u64 * mesh_len as u64;

  // check limits
  let max_texture_side = gx.device().limits().max_texture_dimension_2d;

  if row_len > max_texture_side { panic!("row-length ({row_len}) is greater then max_texture_side ({max_texture_side})") }
  if row_num > max_texture_side { panic!("row-number ({row_num}) is greater then max_texture_side ({max_texture_side})") }


  log::warn!("mesh_len: {mesh_len:#}");

  let vertex_buffer = gx.buffer(BufUse::VERTEX | BufUse::COPY_DST | BufUse::COPY_SRC, mesh_size, false);
  let normal_buffer = gx.buffer(BufUse::VERTEX | BufUse::COPY_DST, mesh_size, false);

  let texcoord_buffer = gx.buffer(BufUse::VERTEX, mesh_size/2, false); // empty buffer to satisfy wgpu quirk

  // cp pipeline
  let cp_shader = gx.load_wgsl(wgsl_modules::include!("common/shaders/frag_compute_sphere_square.wgsl"));

  let consts = shader_constants!{
    steps: steps,
    step_da: std::f32::consts::FRAC_PI_2 / 2.0 / steps as f32 // delta angle per step
  };

  /*let cp_pipeline = gx.render_pipeline(
    1, None, None, &[],
    (&cp_shader, "vs_main", Some(&consts), Primitive { topology: Topology::TriangleStrip, ..Primitive::default() }),
    Some((&cp_shader, "fs_main", Some(&consts), &[(TexFmt::Rgba32Float, None), (TexFmt::Rgba32Float, None)])),
  );*/

  let cp_pipeline = RenderPipelineConfig::new(
      &[], &cp_shader, "vs_main",  Primitive { topology: Topology::TriangleStrip, ..Primitive::default() },
    ).vertex_shader_constants(&consts)
    .fragment(&cp_shader, "fs_main").fragment_shader_constants(&consts)
    .target::<1>(TexFmt::Rgba32Float.target())
    .target::<2>(TexFmt::Rgba32Float.target())
    .pipeline(&gx)
  ;

  let compute_tex = TextureLot::new_2d(&gx,
    [row_len, row_num, 1], 1, TexFmt::Rgba32Float, None, TexUse::RENDER_ATTACHMENT | TexUse::COPY_SRC,
  );
  let compute_tex_normal = TextureLot::new_2d(&gx,
    [row_len, row_num, 1], 1, TexFmt::Rgba32Float, None, TexUse::RENDER_ATTACHMENT | TexUse::COPY_SRC,
  );

  assert_eq!(compute_tex.bytes_per_row().unwrap() * row_num, mesh_size as u32);


  // let read_buffer = gx.buffer(BufUse::COPY_DST | BufUse::MAP_READ, mesh_size, false);

  gx.with_encoder(|encoder| {

    encoder.with_render_pass(
      ([
        Some(compute_tex.color_attachment(Some(Color::TRANSPARENT)).into()),
        Some(compute_tex_normal.color_attachment(Some(Color::TRANSPARENT)).into()),
      ], None),
      |rpass| {
        rpass.set_pipeline(&cp_pipeline);
        rpass.draw(0..4, 0..1);
      }
    );

    encoder.copy_texture_to_buffer(
      compute_tex.texture.as_image_copy(),
      (&vertex_buffer, 0, compute_tex.bytes_per_row(), None).to(),
      compute_tex.texture.size(),
    );

    encoder.copy_texture_to_buffer(
      compute_tex_normal.texture.as_image_copy(),
      (&normal_buffer, 0, compute_tex_normal.bytes_per_row(), None).to(),
      compute_tex_normal.texture.size(),
    );

    // encoder.copy_buffer_to_buffer(&vertex_buffer, 0, &read_buffer, 0, mesh_size);
  });


  // read out the first triangles
  /*read_buffer.with_map_sync(&gx, 0..(6*vertex_size as u64), MapMode::Read, |buffer_slice| {

    let mapped = buffer_slice.get_mapped_range();
    let vertices: &[[[f32;3];3]] = unsafe { mapped.align_to().1 };
    // let vertices: Vec<_> = vertices.iter().map(|v| v[0]).collect();
    // let vertices: &[Vertex] = unsafe { vertices.align_to().1 };
    let vertices: Vec<_> = vertices.iter().map(|v| format!("{:?}", v)).collect();

    log::warn!("{:#?}", vertices);

  }).unwrap();*/


  // instance data

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
  let instance_buffer = gx.buffer_from_data(BufUse::VERTEX, instance_data);


  // world
  let PhysicalSize { width, height } = window.inner_size();
  let (width, height) = (width as f32, height as f32);

  let mut world = WorldView::new(&gx, 10.0, 5.0, 0.1, FovProjection::window(45.0, width, height));

  world.objects = Mat4::from_uniform_scale(0.25 * height);
  world.calc_clip_matrix();

  let light_matrix = Mat4::from_rotation_x(f32::to_radians(-30.0));

  world.light_matrix = light_matrix * world.rotation; // keep light


  // staging belt
  let mut staging_belt = StagingBelt::new(4 * world.clip_buffer.size());

  gx.with_encoder(|mut encoder| {
    staging_belt.write_data(&gx, &mut encoder, &world.clip_buffer, 0, world.clip_matrix);
    staging_belt.write_data(&gx, &mut encoder, &world.light_buffer, 0, world.light_matrix);
    staging_belt.finish();
  });
  staging_belt.recall();


  // bind
  let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
    bind!(0, Buffer, world.clip_buffer),
    bind!(1, Buffer, world.light_buffer),
    bind!(2, TextureView, &color_texture.view),
    bind!(3, Sampler, &sampler),
  ]);

  // render bundles
  let bundles = [target.render_bundle(&gx, |_| {}, |rpass| {
    rpass.set_pipeline(&pipeline);
    rpass.set_bind_group(0, &binding, &[]);
    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
    rpass.set_vertex_buffer(1, texcoord_buffer.slice(..)); // bind empty buffer to satisfy wgpu quirk
    rpass.set_vertex_buffer(2, normal_buffer.slice(..));
    rpass.set_vertex_buffer(3, instance_buffer.slice(..));
    rpass.draw(0..mesh_len as u32, 0..instance_data.len() as u32);
  })];


  // event loop

  move |_ctx, event| match event {

    Event::WindowEvent(WindowEvent::Resized(size)) => {
      target.update(&gx, size);
      world.fov.resize_window(size.width as f32, size.height as f32, true);
      world.calc_clip_matrix();
      world.light_matrix = light_matrix * world.rotation; // keep light

      gx.with_encoder(|mut encoder| {
        staging_belt.write_data(&gx, &mut encoder, &world.clip_buffer, 0, world.clip_matrix);
        staging_belt.write_data(&gx, &mut encoder, &world.light_buffer, 0, world.light_matrix);
        staging_belt.finish();
      });
      staging_belt.recall();
    },

    Event::WindowEvent(WindowEvent::KeyboardInput { event: KeyEvent {
      physical_key: PhysicalKey::Code(keycode), state: ElementState::Pressed, ..
    }, ..}) => {
      if let Some(key) = InputKey::match_keycode(keycode) {
        world.input(key);
        world.calc_clip_matrix();
        world.light_matrix = light_matrix * world.rotation; // keep light

        gx.with_encoder(|mut encoder| {
          staging_belt.write_data(&gx, &mut encoder, &world.clip_buffer, 0, world.clip_matrix);
          staging_belt.write_data(&gx, &mut encoder, &world.light_buffer, 0, world.light_matrix);
          staging_belt.finish();
        });
        staging_belt.recall();

        window.request_redraw();
      }
    },

    Event::WindowEvent(WindowEvent::RedrawRequested) => {

      // let then = time::Instant::now();

      target.with_frame(None, |frame| gx.with_encoder(|encoder| {
        encoder.pass_bundles(frame.attachments(Some(bg_color), Some(1.0), None), &bundles);
      })).expect("frame error");

      // log::warn!("{:?}", then.elapsed());
    },

    _ => {}
  }
}