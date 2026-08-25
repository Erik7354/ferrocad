use ferrocad::{
    Angle, Body, Color, Cuboid, Cylinder, Length, Model, Object, Polygon, Pose2, Pose3, Sketch,
    Solid, ToAngle, ToLength, write_3mf,
};

const STONE: Color = Color::rgb(55, 58, 62);
const WOOD: Color = Color::rgb(121, 85, 48);
const GRASS: Color = Color::rgba(92, 176, 64, 200);
const FROST: Color = Color::rgb(236, 236, 232);
const IRON: Color = Color::rgb(32, 32, 34);

fn main() -> std::io::Result<()> {
    let lamp = Lamp::new();

    write_3mf(&lamp.model(), "minecraft_lamp.3mf")?;
    println!("wrote minecraft_lamp.3mf");
    Ok(())
}

/// Printable Minecraft lantern on a voxel grid.
/// Created using AI.
///
/// Every large solid is `n × n × k` copies of one cube. Side panels are thinner
/// than one voxel so light can pass. Holes and T-slots are CSG cutouts.
#[derive(Clone, Copy)]
struct Lamp {
    /// Edge length of one Minecraft block at print scale.
    voxel: Length,
    /// Blocks along the square footprint.
    n: i64,
    /// Blocks in each corner post.
    cage: i64,
    tol: Length,
}

impl Lamp {
    fn new() -> Self {
        Self {
            voxel: 10.mm(),
            n: 8,
            cage: 8,
            tol: 400.um(),
        }
    }

    fn cube(&self) -> Cuboid {
        Cuboid::cube(self.voxel)
    }

    fn brick(&self, nx: i64, ny: i64, nz: i64) -> Cuboid {
        Cuboid::new(self.voxel * nx, self.voxel * ny, self.voxel * nz)
    }

    /// Center of an `nx × ny × nz` brick whose min corner is voxel `(ix, iy, iz)`.
    /// XY origin is the lamp center. Z origin is the bottom of the base.
    fn cell(
        &self,
        ix: i64,
        iy: i64,
        iz: i64,
        nx: i64,
        ny: i64,
        nz: i64,
    ) -> (Length, Length, Length) {
        (
            self.voxel * (ix * 2 + nx - self.n) / 2,
            self.voxel * (iy * 2 + ny - self.n) / 2,
            self.voxel * (iz * 2 + nz) / 2,
        )
    }

    fn put(&self, nx: i64, ny: i64, nz: i64, ix: i64, iy: i64, iz: i64) -> impl Solid {
        let (x, y, z) = self.cell(ix, iy, iz, nx, ny, nz);
        self.brick(nx, ny, nz).translate(x, y, z)
    }

    fn panel_t(&self) -> Length {
        self.voxel / 8
    }

    fn pixel_relief(&self) -> Length {
        self.voxel / 16
    }

    fn slot_clearance(&self) -> Length {
        200.um()
    }

    fn slot_depth(&self) -> Length {
        self.voxel / 3
    }

    fn slot_open(&self) -> Length {
        self.panel_t() + self.slot_clearance()
    }

    fn slot_head(&self) -> Length {
        self.voxel / 2
    }

    fn tenon_w(&self) -> Length {
        self.voxel * 3 / 5
    }

    fn socket_depth(&self) -> Length {
        self.voxel / 2
    }

    fn socket_w(&self) -> Length {
        self.tenon_w() + self.slot_clearance() * 2
    }

    fn pixel_gap(&self) -> Length {
        400.um()
    }

    fn hole_r(&self) -> Length {
        self.voxel * 2
    }

    fn lip_w(&self) -> Length {
        self.voxel / 3
    }

    fn bore_r(&self) -> Length {
        self.hole_r() - self.lip_w()
    }

    fn counterbore_h(&self) -> Length {
        self.voxel * 2 / 3
    }

    fn cable_w(&self) -> Length {
        self.voxel / 2
    }

    fn flange_h(&self) -> Length {
        self.voxel / 6
    }

    fn ring_od(&self) -> Length {
        self.hole_r() - self.slot_clearance()
    }

    fn ring_wall(&self) -> Length {
        self.lip_w() / 2
    }

    fn pocket_r(&self) -> Length {
        self.ring_od() - self.ring_wall()
    }

    fn grille_r(&self) -> Length {
        self.pocket_r() - self.slot_clearance()
    }

    fn flange_r(&self) -> Length {
        self.bore_r() - self.slot_clearance()
    }

    fn grille_h(&self) -> Length {
        self.voxel / 5
    }

    fn ring_h(&self) -> Length {
        self.counterbore_h() - self.diffuser_t() - self.slot_clearance()
    }

    fn diffuser_t(&self) -> Length {
        self.voxel / 10
    }

    fn diffuser_w(&self) -> Length {
        self.voxel * self.inner() - 2.mm()
    }

    fn inner(&self) -> i64 {
        self.n - 2
    }

    fn object(&self, name: &str, solid: impl Solid, color: Color) -> Object {
        Object::new(name, solid.mesh(self.tol)).with_color(color)
    }

    fn model(&self) -> Model {
        let mut model = Model::new();
        let (px, py, pz) = self.cell(self.n - 1, self.n - 1, 1, 1, 1, self.cage);
        let post = self.post_body().translate(px, py, pz);
        let panel = self.panel_body().translate(Length::ZERO, py, pz);
        let clip = self.clip_body().translate(
            px - self.voxel / 2 + self.slot_depth() / 2,
            py,
            self.voxel * (1 + self.cage) - 4.mm(),
        );

        model.add(self.object("base", self.base(), STONE));
        model.add(self.object("lid", self.lid(), STONE));
        model.add(self.object("diffuser", self.diffuser(), FROST));
        model.add(self.object("ring", self.ring(), IRON));
        model.add(self.object("grille", self.grille(), IRON));

        for i in 0i64..4 {
            let yaw = (i * 90).deg();
            model.add(self.object(
                &format!("post_{i}"),
                post.clone().rotate(Angle::ZERO, Angle::ZERO, yaw),
                WOOD,
            ));
            model.add(self.object(
                &format!("panel_{i}"),
                panel.clone().rotate(Angle::ZERO, Angle::ZERO, yaw),
                GRASS,
            ));
            model.add(self.object(
                &format!("clip_{i}"),
                clip.clone().rotate(Angle::ZERO, Angle::ZERO, yaw),
                STONE,
            ));
        }

        model
    }

    fn base(&self) -> Body {
        let extra = 1.mm();
        let half = self.voxel * self.n / 2;
        let through = Cylinder::new(self.voxel + extra * 2, self.bore_r()).translate(
            Length::ZERO,
            Length::ZERO,
            self.voxel / 2,
        );
        let counterbore = Cylinder::new(self.counterbore_h() + extra, self.hole_r()).translate(
            Length::ZERO,
            Length::ZERO,
            self.voxel - self.counterbore_h() / 2 + extra / 2,
        );
        let canal = Cuboid::new(half + extra, self.cable_w(), self.voxel + extra * 2).translate(
            (half + extra) / 2,
            Length::ZERO,
            self.voxel / 2,
        );
        let recess = Cuboid::new(
            self.diffuser_w() + self.slot_clearance() * 2,
            self.diffuser_w() + self.slot_clearance() * 2,
            self.diffuser_t() + extra,
        )
        .translate(
            Length::ZERO,
            Length::ZERO,
            self.voxel - self.diffuser_t() / 2 + extra / 2,
        );
        let mut cuts: Vec<Body> = (0i64..4)
            .map(|i| {
                self.corner_socket()
                    .rotate(Angle::ZERO, Angle::ZERO, (i * 90).deg())
            })
            .collect();
        cuts.push(through.bake(self.tol));
        cuts.push(counterbore.bake(self.tol));
        cuts.push(canal.bake(self.tol));
        cuts.push(recess.bake(self.tol));
        Body::difference_all(self.put(self.n, self.n, 1, 0, 0, 0), cuts, self.tol)
    }

    fn corner_socket(&self) -> Body {
        let extra = 1.mm();
        let (x, y, _) = self.cell(self.n - 1, self.n - 1, 0, 1, 1, 1);
        let z = self.voxel - self.socket_depth() / 2 + extra / 2;
        Cuboid::new(
            self.socket_w(),
            self.socket_w(),
            self.socket_depth() + extra,
        )
        .translate(x, y, z)
        .bake(self.tol)
    }

    fn lid_socket(&self) -> Body {
        let extra = 1.mm();
        let (x, y, _) = self.cell(self.n - 1, self.n - 1, 0, 1, 1, 1);
        let bottom = self.voxel * (1 + self.cage);
        let z = bottom + self.socket_depth() / 2 - extra / 2;
        Cuboid::new(
            self.socket_w(),
            self.socket_w(),
            self.socket_depth() + extra,
        )
        .translate(x, y, z)
        .bake(self.tol)
    }

    fn lid(&self) -> Body {
        let z0 = 1 + self.cage;
        let mid = self.n - 2;
        let mid_i = (self.n - mid) / 2;
        let cap = Body::union_all(
            [
                self.put(self.n, self.n, 1, 0, 0, z0),
                self.put(mid, mid, 2, mid_i, mid_i, z0 + 1),
            ],
            self.tol,
        )
        .union(self.handle());
        let sockets: Vec<_> = (0i64..4)
            .map(|i| {
                self.lid_socket()
                    .rotate(Angle::ZERO, Angle::ZERO, (i * 90).deg())
            })
            .collect();
        Body::difference_all(cap, sockets, self.tol)
    }

    fn handle(&self) -> Body {
        let v = self.voxel;
        let lift = self.voxel * (1 + self.cage + 3);
        Body::union_all(
            [
                self.cube().translate(-v, Length::ZERO, v * 3 / 2),
                self.cube().translate(Length::ZERO, Length::ZERO, v * 3 / 2),
                self.cube().translate(v, Length::ZERO, v * 3 / 2),
                self.cube().translate(-v, Length::ZERO, v / 2),
                self.cube().translate(v, Length::ZERO, v / 2),
            ],
            self.tol,
        )
        .rotate(Angle::ZERO, Angle::ZERO, 45.deg())
        .translate(Length::ZERO, Length::ZERO, lift)
    }

    fn post_body(&self) -> Body {
        let height = self.voxel * self.cage;
        let slot = self.t_slot_cut(height + 2.mm());
        let post = self
            .brick(1, 1, self.cage)
            .bake(self.tol)
            .difference(slot.clone())
            .difference(slot.rotate(Angle::ZERO, Angle::ZERO, 90.deg()));
        let bottom = Cuboid::new(self.tenon_w(), self.tenon_w(), self.socket_depth()).translate(
            Length::ZERO,
            Length::ZERO,
            -(height / 2) - self.socket_depth() / 2,
        );
        let top = Cuboid::new(self.tenon_w(), self.tenon_w(), self.socket_depth()).translate(
            Length::ZERO,
            Length::ZERO,
            height / 2 + self.socket_depth() / 2,
        );
        post.union(bottom).union(top)
    }

    fn t_slot_cut(&self, height: Length) -> Body {
        let extra = 1.mm();
        let face = -(self.voxel / 2);
        let neck_x = self.slot_depth() + extra;
        let neck = Cuboid::new(neck_x, self.slot_open(), height).translate(
            face + self.slot_depth() / 2 - extra / 2,
            Length::ZERO,
            Length::ZERO,
        );
        let head_d = self.voxel / 5;
        let head = Cuboid::new(head_d, self.slot_head(), height).translate(
            face + self.slot_depth() - head_d / 2,
            Length::ZERO,
            Length::ZERO,
        );
        neck.bake(self.tol).union(head)
    }

    fn panel_body(&self) -> Body {
        let nx = self.inner();
        let nz = self.cage;
        let gap = self.pixel_gap();
        let w = self.voxel * nx + self.slot_depth() * 2 - self.slot_clearance() * 2;
        let h = self.voxel * nz - gap;
        let plate = Cuboid::new(w, self.panel_t(), h);
        let face = self.panel_t() / 2;
        let overlap = 50.um();
        let boss = self.voxel - gap;

        let mut pixels = Vec::with_capacity((nx * nz) as usize);
        for i in 0..nx {
            for k in 0..nz {
                let relief = self.pixel_relief() + 200.um() * ((i * 3 + k * 5) % 4);
                let x = self.voxel * (i * 2 - nx + 1) / 2;
                let z = self.voxel * (k * 2 - nz + 1) / 2;
                pixels.push(Cuboid::new(boss, relief, boss).translate(
                    x,
                    face + relief / 2 - overlap,
                    z,
                ));
            }
        }

        plate
            .bake(self.tol)
            .union(Body::union_all(pixels, self.tol))
    }

    fn diffuser(&self) -> Body {
        let t = self.diffuser_t();
        Cuboid::new(self.diffuser_w(), self.diffuser_w(), t)
            .translate(Length::ZERO, Length::ZERO, self.voxel - t / 2)
            .bake(self.tol)
    }

    fn ring(&self) -> Body {
        let h = self.ring_h();
        let od = self.ring_od();
        let z = self.voxel - self.counterbore_h() + h / 2;
        let pocket = Cylinder::new(h - self.flange_h() + 2.mm(), self.pocket_r()).translate(
            Length::ZERO,
            Length::ZERO,
            self.flange_h() / 2 + 1.mm(),
        );
        let cable =
            Cylinder::new(h + 2.mm(), self.cable_w() / 2).translate(od, Length::ZERO, Length::ZERO);
        Cylinder::new(h, od)
            .difference(Cylinder::new(h + 2.mm(), self.flange_r()))
            .difference(pocket)
            .difference(cable)
            .translate(Length::ZERO, Length::ZERO, z)
            .bake(self.tol)
    }

    fn grille(&self) -> Body {
        let h = self.grille_h();
        let r = self.grille_r();
        let z = self.voxel - self.counterbore_h() + self.flange_h() + h / 2;
        let arm = Cuboid::new(r * 2, self.voxel / 2, h);
        let spokes = Body::union_all(
            [
                arm.translate(Length::ZERO, Length::ZERO, Length::ZERO),
                arm.rotate(Angle::ZERO, Angle::ZERO, 90.deg()),
            ],
            self.tol,
        );
        let hub = Cylinder::new(h, self.voxel / 2);
        let disc = Cylinder::new(h, r);
        let bore = Cylinder::new(h + 2.mm(), self.voxel / 4);
        spokes
            .union(hub)
            .intersection(disc)
            .difference(bore)
            .translate(Length::ZERO, Length::ZERO, z)
    }

    fn clip_body(&self) -> Body {
        let w = self.voxel * 3 / 5;
        let h = self.voxel * 4 / 5;
        let t = self.panel_t();
        let bar = (h - t) / 2;
        Polygon::new(vec![
            [0.mm(), 0.mm()],
            [t, 0.mm()],
            [t, bar],
            [w - t, bar],
            [w - t, 0.mm()],
            [w, 0.mm()],
            [w, h],
            [w - t, h],
            [w - t, bar + t],
            [t, bar + t],
            [t, h],
            [0.mm(), h],
        ])
        .translate(-(w / 2), -(h / 2))
        .extrude(self.voxel / 5)
        .rotate(90.deg(), Angle::ZERO, Angle::ZERO)
        .bake(self.tol)
    }
}
