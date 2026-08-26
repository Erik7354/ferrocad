use ferrocad::{Body, Length, Pose3, Solid, Sphere, ToLength, write_stl};

fn main() -> std::io::Result<()> {
    let capsule = Body::hull_all(
        [
            Sphere::new(10.mm()).posed(),
            Sphere::new(10.mm()).translate(40.mm(), Length::ZERO, Length::ZERO),
        ],
        500.um(),
    );

    write_stl(&capsule.mesh(500.um()), "capsule.stl")?;
    println!("wrote capsule.stl");
    Ok(())
}
