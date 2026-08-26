# ferrocad

Ferrocad is a Rust library for creating parametric 2D sketches and 3D models directly in Rust and exporting them to formats commonly used for manufacturing and 3D printing.

The library treats models as ordinary Rust programs so you can write short, simple programs that produce high quality CAD models.

## Example

You can find this example as well as other examples in the example directory.

```rust
use ferrocad::{Cuboid, Cylinder, Solid, ToLength, write_stl};

fn main() -> std::io::Result<()> {
    let plate = Cuboid::new(40.mm(), 30.mm(), 4.mm()).difference(Cylinder::new(10.mm(), 5.mm()));

    write_stl(&plate.mesh(500.um()), "hello_world.stl")?;
    println!("wrote hello_world.stl");
    Ok(())
}
```

## Why

Traditional CAD tools are primarily designed around graphical interfaces.
Script-based CAD tools improve reproducibility, but often require learning a domain-specific language.
A ferrocad project is just ordinary Rust. 
This small difference enables some benefits:

- idiomatic Rust, not a custom language
- AI first - Agents are excellent at writing Rust 
- put your models into version control - no more `bracket-final-v2-actually-final.step`
- composable sketches, solids, transforms, booleans, and meshes
- direct export to STL, 3MF, OBJ, and similar formats

## Features

See [FEATURES.md](./FEATURES.md) in this repository.

## Changes 

Ferrocad is still very young so please expect breaking changes between versions.

## Contribution

Thanks for your help improving the project!
I am looking forward hearing your ideas how ferrocad can be improved.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Tokio by you shall be licensed as MIT, without any additional terms or conditions.

## Commercial Use

This library is free to use commercially and non-commercially.

If you're using it in a commercial project, I'd love to hear about it!
There's no obligation to contact me - I'm simply interested in knowing
where the library ends up being useful.

X: [@erik7354](https://x.com/Erik7354)
