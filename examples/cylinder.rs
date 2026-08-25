use ferrocad::{Cylinder, Solid, ToLength, write_stl};

fn main() -> std::io::Result<()> {
    let cylinder = Cylinder::new(20.mm(), 10.mm());
    write_stl(&cylinder.mesh(1.mm()), "cylinder.stl")?;
    println!("wrote cylinder.stl");
    Ok(())
}
