use ferrocad::{Circle, Length, Pose2, Sketch, Solid, ToAngle, ToLength, write_stl};

fn main() -> std::io::Result<()> {
    let torus = Circle::new(5.mm())
        .translate(20.mm(), Length::ZERO)
        .rotate_extrude(360.deg());

    write_stl(&torus.mesh(1.mm()), "torus.stl")?;
    println!("wrote torus.stl");
    Ok(())
}
