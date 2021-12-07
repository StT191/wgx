

use crate::error::*;


use std::str::{FromStr};


fn parse_vec<'a>(line:&mut impl Iterator<Item=&'a str>) -> Res<[f32;3]> {

    let vec:Vec<f32> = line.map(|v| f32::from_str(v)).collect::<Result<_, _>>().map_err(error)?;

    if vec.len() < 2 {
        Err(format!("bad vec length {}", vec.len()))
    }
    else {
        Ok([vec[0], vec[1], if let Some(v) = vec.get(2) { *v } else { 0.0 }])
    }
}


fn parse_face<'a>(line:&mut impl Iterator<Item=&'a str>) -> Res<Vec<(usize, Option<usize>, Option<usize>)>> {

    let face:Vec<(usize, Option<usize>, Option<usize>)> = line.map(|part| {

        let part:Vec<usize> = part.split("/").map(|v| usize::from_str(v)).collect::<Result<_, _>>().map_err(error)?;

        if part.len() == 0 {
            Err("bad face".to_string())
        }
        else {
            Ok((
                part[0]-1,
                if let Some(v) = part.get(1) { Some(v-1) } else { None },
                if let Some(v) = part.get(2) { Some(v-1) } else { None },
            ))
        }
    }).collect::<Result<_, _>>().map_err(error)?;


    let len = face.len();

    if len < 3 || len > 4 {
        Err(format!("bad face length: {}", len))
    }
    else {
        Ok(face)
    }
}



use cgmath::*;


pub fn parse(raw:&str) -> Res<Vec<[[[f32;3];3];3]>> {

    let mut vertices:Vec<[f32;3]> = Vec::new();
    let mut vertex_tex_coords:Vec<[f32;3]> = Vec::new();
    let mut normals:Vec<[f32;3]> = Vec::new();

    let mut faces:Vec<Vec<(usize, Option<usize>, Option<usize>)>> = Vec::new();

    for line in raw.split("\n") {

        let mut line = line.trim().split(" ").filter(|v| v.trim() != "");

        match line.next() {
            Some("v") => { vertices.push(parse_vec(&mut line)?); }
            Some("vt") => { vertex_tex_coords.push(parse_vec(&mut line)?); }
            Some("vn") => { normals.push(parse_vec(&mut line)?); }
            Some("f") => { faces.push(parse_face(&mut line)?); }
            _ => {}
        }
    }

    let mut triangles = Vec::new();

    // let mut once = false;

    for face in faces {

        let mut calc_normals = false;

        let mut trgs = Vec::with_capacity(4);

        for (v, t, n) in face {

            if n.is_none() {
                calc_normals = true;
            }

            trgs.push([
                vertices[v],
                if let Some(i) = t { vertex_tex_coords[i] } else { [0.0, 0.0, 0.0] },
                if let Some(i) = n { normals[i] } else { [0.0, 0.0, 0.0] },
            ]);
        }

        if calc_normals {
            let o = Vector3::from(trgs[0][0]);
            let a = Vector3::from(trgs[1][0]);
            let b = Vector3::from(trgs[2][0]);

            let normal = (a - o).cross(b - o).normalize().into();

            for trg in trgs.iter_mut() { trg[2] = normal; }
        }

        triangles.push([trgs[0], trgs[1], trgs[2]]);

        if trgs.len() == 4 {
            triangles.push([trgs[0], trgs[2], trgs[3]]);
        }
    }

    Ok(triangles)
}
