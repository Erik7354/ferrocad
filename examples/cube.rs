use rs_cad::{Cuboid, Solid, ToLength, write_stl};

fn main() -> std::io::Result<()> {
    let cube = Cuboid::cube(20.mm());
    write_stl(&cube.mesh(1.mm()), "cube.stl")?;
    println!("wrote cube.stl");
    Ok(())
}
