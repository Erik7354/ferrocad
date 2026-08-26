use ferrocad::{Rectangle, Sketch, Solid, ToAngle, ToLength, write_stl};

fn main() -> std::io::Result<()> {
    let solid = Rectangle::new(20.mm(), 10.mm()).twist_extrude(40.mm(), 90.deg());

    write_stl(&solid.mesh(1.mm()), "twist.stl")?;
    println!("wrote twist.stl");
    Ok(())
}
