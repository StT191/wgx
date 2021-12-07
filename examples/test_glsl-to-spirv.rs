#[allow(unused)]

use glsl_to_spirv::{compile, ShaderType, ShaderType::*};
use std::{fs, io::{stdout, copy}, env};

pub fn from_file(path:&str, ty:ShaderType) -> fs::File {
    let code = fs::read_to_string(path).unwrap();
    glsl_to_spirv::compile(&code, ty).unwrap()
}

fn main() {

    let mut args = env::args();

    let ty = match args.nth(1).unwrap().as_ref() {
        "-v" => Vertex,
        "-f" => Fragment,
        "-c" => Compute,
        "-g" => Geometry,
        "-tc" => TessellationControl,
        "-te" => TessellationEvaluation,
        _ => panic!("bad type argument")
    };

    let filename = args.next().unwrap();

    let mut shader = from_file(&filename, ty);

    copy(&mut shader, &mut stdout()).unwrap();
}