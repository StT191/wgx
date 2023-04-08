
// imports
use std::{time::{Instant}};
use pollster::FutureExt;
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::Window, event::{Event, WindowEvent},
};
use wgx::{*, cgmath::*};

// main
pub fn main() {

    const DEPTH_TESTING:bool = false;
    const MSAA:u32 = 1;
    const ALPHA_BLENDING:Option<BlendState> = None;


    let (width, height) = (1000, 1000);

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::<u32>::from((width, height)));
    window.set_title("WgFx");

    let (gx, surface) = unsafe {Wgx::new(Some(&window), Features::empty(), limits!{})}.block_on().unwrap();
    let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (width, height), MSAA, DEPTH_TESTING).unwrap();


    // pipeline
    let shader = gx.load_wgsl(include_wgsl_module!("./shaders/ellipse.wgsl"));

    let pipeline = target.render_pipeline(&gx,
        None, &[vertex_desc!(Vertex, 0 => Float32x2, 1 => Float32x4)],
        (&shader, "vs_main", Primitive::TriangleList),
        (&shader, "fs_main", ALPHA_BLENDING),
    );


    // path

    let color = Color::RED.f32();

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Vtx([f32;2], [f32; 4]);
    unsafe impl wgx::ReadBytes for Vtx {}

    // corners
    let c = [
        Vtx([-1.0, -1.0f32], color),
        Vtx([ 1.0, -1.0],    color),
        Vtx([ 1.0,  1.0],    color),
        Vtx([-1.0,  1.0],    color),
    ];

    // vertices
    let data = [
        c[0], c[1], c[2],
        c[0], c[2], c[3],
    ];
    let vertices = gx.buffer_from_data(BufUse::VERTEX, &data[..]);


    // projection
    let projection = Matrix4::<f32>::identity();

    let pj_buffer = gx.buffer_from_data(BufUse::UNIFORM | BufUse::COPY_DST, AsRef::<[f32; 16]>::as_ref(&projection));
    // let tf_buffer = gx.buffer_from_data(BufUse::UNIFORM | BufUse::COPY_DST, AsRef::<[f32; 16]>::as_ref(&projection));

    let dim_buffer = gx.buffer_from_data(BufUse::UNIFORM | BufUse::COPY_DST, [1.0_f32; 4]);

    // binding
    let binding = gx.bind(&pipeline.get_bind_group_layout(0), &[
        bind!(0, Buffer, &pj_buffer, 0, None),
        // bind!(1, Buffer, tf_buffer.slice(..)),
        bind!(1, Buffer, &dim_buffer, 0, None),
    ]);


    // event loop
    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {
                *control_flow = ControlFlow::Exit;
            },

            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {

                target.update(&gx, (size.width, size.height));

                let (width, height) = (size.width as f32, size.height as f32);

                let (w, h) = (width/3.0, height/3.0);

                let obj_mat:Matrix4::<f32> =
                    // Matrix4::from_angle_x(Deg(75.0)) *
                    // Matrix4::from_angle_y(Deg(75.0)) *
                    // Matrix4::from_angle_z(Deg(45.0)) *
                    // Matrix4::from_nonuniform_scale(1.0, 1.0, 1.0) *
                    Matrix4::from_nonuniform_scale(w, h, 1.0)
                ;

                let p_p = Matrix4::from_nonuniform_scale(1.0, 1.0, 0.0);

                let dim_x = (obj_mat * Vector4::<f32>::new(1.0, 0.0, 0.0, 0.0)).magnitude();
                let dim_y = (obj_mat * Vector4::<f32>::new(0.0, 1.0, 0.0, 0.0)).magnitude();

                let dim_x_p = (p_p * obj_mat * Vector4::<f32>::new(1.0, 0.0, 0.0, 0.0)).magnitude();
                let dim_y_p = (p_p * obj_mat * Vector4::<f32>::new(0.0, 1.0, 0.0, 0.0)).magnitude();

                // projection
                let projection =
                    // window_fov_projection(70.0, width, height) *
                    flat_window_projection(width, height, 0.0) *
                    Matrix4::from_translation(Vector3::<f32>::new(width/2.0, height/2.0, 0.0)) *
                    obj_mat
                ;

                gx.write_buffer(&pj_buffer, 0, AsRef::<[f32; 16]>::as_ref(&projection));
                gx.write_buffer(&dim_buffer, 0, [dim_x, dim_y, dim_x_p/dim_x, dim_y_p/dim_y]);
                // gx.write_buffer(&dim_buffer, 0, &[dim_x_p, dim_y_p, 1.0, 1.0]);
                // gx.write_buffer(&dim_buffer, 0, &[dim_x, dim_y, 1.0, 1.0]);
            },

            Event::WindowEvent {
                event:WindowEvent::KeyboardInput{
                    input: winit::event::KeyboardInput {
                        virtual_keycode:Some(winit::event::VirtualKeyCode::R), ..
                    }, ..
                }, ..
            } => {
                window.request_redraw();
            },

            Event::RedrawRequested(_) => {

                let then = Instant::now();


                target.with_encoder_frame(&gx, |encoder, frame| {

                    encoder.with_render_pass(frame.attachments(Some(Color::GREEN), None), |mut rpass| {
                        rpass.set_pipeline(&pipeline);
                        rpass.set_bind_group(0, &binding, &[]);
                        rpass.set_vertex_buffer(0, vertices.slice(..));
                        rpass.draw(0..data.len() as u32, 0..1);
                    });

                }).expect("frame error");


                println!("{:?}", then.elapsed());
            },

            _ => {}
        }
    });
}