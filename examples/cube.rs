use rs_cad::{write_stl, Cuboid, Solid, ToLength};

fn main() -> std::io::Result<()> {
    let cube = Cuboid::cube(20.mm());
    write_stl(&cube.mesh(1.mm()), "cube.stl")?;
    println!("wrote cube.stl");
    Ok(())
}
