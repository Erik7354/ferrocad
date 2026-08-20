use ferrocad::{Solid, Sphere, ToLength, write_stl};

fn main() -> std::io::Result<()> {
    let sphere = Sphere::new(10.mm());
    write_stl(&sphere.mesh(1.mm()), "sphere.stl")?;
    println!("wrote sphere.stl");
    Ok(())
}
