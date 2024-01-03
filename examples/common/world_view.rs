
use winit::keyboard::KeyCode;
use wgx::{*, cgmath::*};

#[derive(Debug, Clone, PartialEq)]
pub enum InputKey {
    Up(Option<f32>), Down(Option<f32>),
    Left(Option<f32>), Right(Option<f32>),
    Backwards(Option<f32>), Forwards(Option<f32>),
    TiltUp(Option<f32>), TiltDown(Option<f32>),
    PanLeft(Option<f32>), PanRight(Option<f32>),
    RollLeft(Option<f32>), RollRight(Option<f32>),
    ZoomIn(Option<f32>), ZoomOut(Option<f32>),
    Reset,
}

impl InputKey {
    pub fn match_keycode(keycode: KeyCode) -> Option<Self> {
        match keycode {
            KeyCode::KeyW => Some(InputKey::Up(None)), KeyCode::KeyS => Some(InputKey::Down(None)),
            KeyCode::KeyA => Some(InputKey::Left(None)), KeyCode::KeyD => Some(InputKey::Right(None)),
            KeyCode::KeyQ => Some(InputKey::Backwards(None)), KeyCode::KeyE => Some(InputKey::Forwards(None)),

            KeyCode::KeyI => Some(InputKey::TiltDown(None)), KeyCode::KeyK => Some(InputKey::TiltUp(None)),
            KeyCode::KeyJ => Some(InputKey::PanLeft(None)), KeyCode::KeyL => Some(InputKey::PanRight(None)),
            KeyCode::KeyU => Some(InputKey::RollLeft(None)), KeyCode::KeyO => Some(InputKey::RollRight(None)),

            // US layout!
            KeyCode::KeyZ => Some(InputKey::ZoomIn(None)), KeyCode::KeyX => Some(InputKey::ZoomOut(None)),

            KeyCode::KeyR => Some(InputKey::Reset),

            _ => None,
        }
    }
}


#[derive(Debug)]
pub struct WorldView {

    pub ds: f32, // delta translation
    pub da: f32, // delta angle
    pub df: f32, // delta zoom

    pub fov: FovProjection<f32>,

    pub rotation: Matrix4::<f32>,
    pub scene: Matrix4::<f32>,
    pub objects: Matrix4::<f32>,

    pub clip_matrix: Matrix4::<f32>,
    pub clip_buffer: wgpu::Buffer,

    pub light_matrix: Matrix4::<f32>,
    pub light_buffer: wgpu::Buffer,
}

impl WorldView {

    pub fn new(gx: &impl WgxDevice, ds: f32, da: f32, df: f32, fov: FovProjection<f32>) -> Self {
        let clip_matrix = fov.projection * fov.translation;
        Self {
            ds, da, df, fov,
            rotation: Matrix4::<f32>::identity(),
            scene: Matrix4::<f32>::identity(),
            objects: Matrix4::<f32>::identity(),
            clip_buffer: gx.buffer(BufUse::UNIFORM | BufUse::COPY_DST, 64, false),
            clip_matrix,
            light_matrix: Matrix4::<f32>::identity(),
            light_buffer: gx.buffer(BufUse::UNIFORM | BufUse::COPY_DST, 64, false),
        }
    }

    pub fn write_clip_buffer(&self, gx: &impl WgxDeviceQueue) {
        gx.write_buffer(&self.clip_buffer, 0, AsRef::<[f32; 16]>::as_ref(&self.clip_matrix));
    }

    pub fn write_light_buffer(&self, gx: &impl WgxDeviceQueue) {
        gx.write_buffer(&self.light_buffer, 0, AsRef::<[f32; 16]>::as_ref(&self.light_matrix));
    }

    pub fn calc_clip_matrix(&mut self) {
        self.clip_matrix = self.fov.projection * self.fov.translation * self.rotation * self.scene * self.objects;
    }

    pub fn translate(&mut self, translation: (f32, f32, f32)) {
        apply!(self.scene, Matrix4::from_translation(translation.into()));
    }

    pub fn rotate_x(&mut self, angle_deg: f32) {
        apply!(self.rotation, Matrix4::from_angle_x(Deg(angle_deg)));
    }

    pub fn rotate_y(&mut self, angle_deg: f32) {
        apply!(self.rotation, Matrix4::from_angle_y(Deg(angle_deg)));
    }

    pub fn rotate_z(&mut self, angle_deg: f32) {
        apply!(self.rotation, Matrix4::from_angle_z(Deg(angle_deg)));
    }

    pub fn scale(&mut self, factor: f32) {
        apply!(self.rotation, Matrix4::from_scale(factor));
    }

    pub fn reset_scene_rotation(&mut self) {
        self.scene = Matrix4::identity();
        self.rotation = Matrix4::identity();
    }

    pub fn input(&mut self, key: InputKey) {
        match key {
            InputKey::Up(s) => self.translate((0.0, 0.0, s.unwrap_or(self.ds))),
            InputKey::Down(s) => self.translate((0.0, 0.0, -s.unwrap_or(self.ds))),
            InputKey::Left(s) => self.translate((-s.unwrap_or(self.ds), 0.0, 0.0)),
            InputKey::Right(s) => self.translate((s.unwrap_or(self.ds), 0.0, 0.0)),
            InputKey::Backwards(s) => self.translate((0.0, -s.unwrap_or(self.ds), 0.0)),
            InputKey::Forwards(s) => self.translate((0.0, s.unwrap_or(self.ds), 0.0)),

            InputKey::TiltUp(a) => self.rotate_x(-a.unwrap_or(self.da)),
            InputKey::TiltDown(a) => self.rotate_x(a.unwrap_or(self.da)),
            InputKey::PanLeft(a) => self.rotate_y(a.unwrap_or(self.da)),
            InputKey::PanRight(a) => self.rotate_y(-a.unwrap_or(self.da)),
            InputKey::RollLeft(a) => self.rotate_z(a.unwrap_or(self.da)),
            InputKey::RollRight(a) => self.rotate_z(-a.unwrap_or(self.da)),

            InputKey::ZoomIn(f) => self.scale(1.0 + f.unwrap_or(self.df)),
            InputKey::ZoomOut(f) => self.scale(1.0 - f.unwrap_or(self.df)),

            InputKey::Reset => self.reset_scene_rotation(),
        }
    }

}