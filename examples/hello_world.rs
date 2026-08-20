use ferrocad::{Cuboid, Cylinder, Solid, ToLength, write_stl};

fn main() -> std::io::Result<()> {
    let plate = Cuboid::new(40.mm(), 30.mm(), 4.mm()).difference(Cylinder::new(5.mm(), 10.mm()));

    write_stl(&plate.mesh(500.um()), "hello_world.stl")?;
    println!("wrote hello_world.stl");
    Ok(())
}
