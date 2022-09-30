
mod arc;
mod arc_cp;
mod arc_ellipse;
mod arc_in;
mod arc_vin;

fn main() {

    let program = std::env::args().nth(1);

    match program.as_deref() {
        None => arc::main(),
        Some("cp") => arc_cp::main(),
        Some("ellipse") => arc_ellipse::main(),
        Some("in") => arc_in::main(),
        Some("vin") => arc_vin::main(),
        Some(unkown) => panic!("program '{unkown}' doesn't exist"),
    }
}