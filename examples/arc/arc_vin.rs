
use std::{time::{Instant}};
use pollster::FutureExt;
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
};
use wgx::{*, cgmath::*};


pub fn main() {

    const DEPTH_TESTING:bool = false;
    const MSAA:u32 = 4;
    const ALPHA_BLENDING:Option<BlendState> = None;


    let (width, height) = (1000, 1000);

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((width, height)));
    window.set_title("WgFx");

    let (gx, surface) = unsafe {Wgx::new(Some(&window), Features::empty(), limits!{}).block_on()}.unwrap();
    let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (width, height), MSAA, DEPTH_TESTING).unwrap();


    // pipeline
    let shader = gx.load_wgsl(include_wgsl_module!("./shaders/arc_vin.wgsl"));

    let pipeline = target.render_pipeline(&gx,
        None, &[
            vertex_desc!(Instance, 0 => Float32x3, 1 => Float32x3, 2 => Float32x3, 3 => Float32x4),
            vertex_desc!(Vertex, 4 => Float32x4),
        ],
        (&shader, "vs_main", Primitive::TriangleList),
        (&shader, "fs_main", ALPHA_BLENDING),
    );


    let red = Color::RED.f32();
    let blue = Color::BLUE.f32();

    // corners
    let c = [
        [ 0.0,  0.0, 0.0_f32],
        [-1.0,  0.0, 0.0],
        [ 0.0, -1.0, 0.0],
        [ 1.0,  0.0, 0.0],
        [ 0.0,  1.0, 0.0],
    ];

    let mut instance_data = vec![
        (c[1], c[0], c[2], blue),
        (c[2], c[0], c[3], red),
        (c[3], c[0], c[4], blue),
        (c[4], c[0], c[1], red),
    ];

    for _ in 0..100 {
        instance_data.push(instance_data[0]);
        instance_data.push(instance_data[1]);
        instance_data.push(instance_data[2]);
        instance_data.push(instance_data[3]);
    }


    let steps = 64;
    let steps_f = steps as f32;

    let mut vertex_data:Vec<[f32;4]> = Vec::with_capacity(3 * steps as usize);

    let pi0 = std::f32::consts::FRAC_PI_2;

    for j in 0..steps {

        let fi0 = j as f32 / steps_f * pi0;
        let fi1 = (j as f32 + 1.0) / steps_f * pi0;

        vertex_data.push([f32::cos(fi0), f32::sin(fi0), 0.0, 1.0]);
        vertex_data.push([          0.0,           0.0, 0.0, 1.0]);
        vertex_data.push([f32::cos(fi1), f32::sin(fi1), 0.0, 1.0]);
    }



    let instance_buffer = gx.buffer_from_data(BufUse::VERTEX, &instance_data);
    let vertex_buffer = gx.buffer_from_data(BufUse::VERTEX, &vertex_data);
    // let step_buffer = gx.buffer_from_data(BufUse::UNIFORM, &[steps as u32]);


    // projection
    // let world_buffer = gx.buffer(BufUse::UNIFORM | BufUse::COPY_DST, 64, false);
    let clip_buffer = gx.buffer(BufUse::UNIFORM | BufUse::COPY_DST, 64, false);
    // let viewport_buffer = gx.buffer(BufUse::UNIFORM | BufUse::COPY_DST, 8, false);


    // binding
    let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
        // bind!(0, Buffer, &world_buffer),
        bind!(1, Buffer, &clip_buffer),
        // bind!(2, Buffer, &step_buffer),
        // bind!(2, Buffer, &viewport_buffer),
    ]);


    // render bundles
    let bundles = [target.render_bundle(&gx, |rpass| {
        rpass.set_pipeline(&pipeline);
        rpass.set_bind_group(0, &binding, &[]);
        rpass.set_vertex_buffer(0, instance_buffer.slice(..));
        rpass.set_vertex_buffer(1, vertex_buffer.slice(..));
        rpass.draw(0..vertex_data.len() as u32, 0..instance_data.len() as u32);
    })];


    // matrix
    const DA:f32 = 3.0;
    const DT:f32 = 0.01;

    let (mut width, mut height) = (width as f32, height as f32);
    let (mut w, mut h) = (0.4, 0.4);

    let mut rot_matrix = Matrix4::<f32>::identity();

    let fov = FovProjection::window(30.0, width, height);
    let projection =
        fov.projection * fov.translation
        // flat_window_projection(width, height, 0.0) *
        // Matrix4::from_translation((width/2.0, height/2.0, 0.0).into())
    ;

    let clip_matrix = projection * rot_matrix * Matrix4::from_nonuniform_scale(w*width, h*height, 1.0);

    // gx.write_buffer(&world_buffer, 0, AsRef::<[f32; 16]>::as_ref(&world_matrix));
    gx.write_buffer(&clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
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

                width = size.width as f32;
                height = size.height as f32;

                // projection
                let clip_matrix = projection * rot_matrix * Matrix4::from_nonuniform_scale(w*width, h*height, 1.0);

                gx.write_buffer(&clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));
                // gx.write_buffer(&viewport_buffer, 0, &[width, height]);
            },

            Event::WindowEvent { event:WindowEvent::KeyboardInput { input: KeyboardInput {
                virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
            }, ..}, ..} => {
                let mut redraw = true;
                match keycode {
                    VirtualKeyCode::I => { rot_matrix = Matrix4::from_angle_x(Deg( DA)) * rot_matrix; },
                    VirtualKeyCode::K => { rot_matrix = Matrix4::from_angle_x(Deg(-DA)) * rot_matrix; },
                    VirtualKeyCode::J => { rot_matrix = Matrix4::from_angle_y(Deg( DA)) * rot_matrix; },
                    VirtualKeyCode::L => { rot_matrix = Matrix4::from_angle_y(Deg(-DA)) * rot_matrix; },
                    VirtualKeyCode::U => { rot_matrix = Matrix4::from_angle_z(Deg( DA)) * rot_matrix; },
                    VirtualKeyCode::O => { rot_matrix = Matrix4::from_angle_z(Deg(-DA)) * rot_matrix; },

                    VirtualKeyCode::W => { h += DT; },
                    VirtualKeyCode::S => { h -= DT; },
                    VirtualKeyCode::A => { w -= DT; },
                    VirtualKeyCode::D => { w += DT; },

                    VirtualKeyCode::R => {
                        rot_matrix = Matrix4::identity();
                        w = 0.4;
                        h = 0.4;
                    },

                    _ => { redraw = false; }
                } {
                    if redraw {

                        let clip_matrix = projection * rot_matrix * Matrix4::from_nonuniform_scale(w*width, h*height, 1.0);

                        gx.write_buffer(&clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&clip_matrix));

                        window.request_redraw();
                    }
                }
            },

            Event::RedrawRequested(_) => {

                let then = Instant::now();

                target.with_encoder_frame(&gx, |encoder, frame| {
                    encoder.render_bundles(frame.attachments(Some(Color::GREEN), None), &bundles);
                }).expect("frame error");

                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}