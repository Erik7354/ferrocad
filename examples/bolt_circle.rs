use ferrocad::{Angle, Cuboid, Cylinder, Pose3, Solid, ToAngle, ToLength, write_stl};

fn main() -> std::io::Result<()> {
    let mut plate = Cuboid::new(40.mm(), 40.mm(), 4.mm()).bake(500.um());
    let hole = Cylinder::new(2.mm(), 10.mm()).translate(12.mm(), 0.mm(), 0.mm());
    for i in 0i64..6 {
        plate = plate.difference(hole.rotate(Angle::ZERO, Angle::ZERO, (i * 60).deg()));
    }

    write_stl(&plate.mesh(500.um()), "bolt_circle.stl")?;
    println!("wrote bolt_circle.stl");
    Ok(())
}
