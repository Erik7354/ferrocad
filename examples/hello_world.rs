use ferrocad::{Cuboid, Cylinder, Solid, ToLength, write_stl};

fn main() -> std::io::Result<()> {
    let plate = Cuboid::new(40.mm(), 30.mm(), 4.mm())
        .bake(500.um())
        .difference(Cylinder::new(10.mm(), 5.mm()));

    write_stl(&plate.mesh(500.um()), "hello_world.stl")?;
    println!("wrote hello_world.stl");
    Ok(())
}
