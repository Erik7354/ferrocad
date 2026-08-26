# Features

## Units

[x] Length(um | mm | cm | m)
[x] Angle(deg | rad)

## 2D

[x] Circle(radius | diameter)
[x] Rectangle(width, depth) / Square(size)
[x] Polygon(points)
[] Polygon holes (`paths`)
[] Text
[] import
[] projection(cut)

## 3D

[x] Sphere(radius | diameter)
[x] Cuboid(width, depth, height) / Cube(size)
[x] Cylinder(height, radius | diameter) / Frustum(bottom_radius, top_radius, height) / Cone(radius, height)
[] Polyhedron
[] import
[x] Extrude(height)
[x] Extrude twist / slices
[x] RotateExtrude(angle)
[] surface

## Transformations

[x] translate(x, y) / translate(x, y, z)
[x] rotate(angle) / rotate(x, y, z) - Euler X, then Y, then Z
[x] rotate_axis(angle, x, y, z)
[x] scale(x, y) / scale(x, y, z)
[] resize
[x] mirror(x, y) / mirror(x, y, z)
[x] multmatrix(Affine2) / multmatrix(Affine3)
[] offset(r | delta, chamfer)
[] hull
[] minkowski

## Boolean operations

[x] Solid::bake(tolerance) → Body
[x] union / union_all
[x] difference / difference_all 
[x] intersection

## Mesh and export

[x] mesh(tolerance)
[x] write_stl(mesh, path)
[x] write_3mf(model, path)
[x] ThreeMfExport(with_title | with_designer | with_description)
[] OBJ

## Model and appearance

[x] Model(new | add)
[x] Object(name, mesh) | with_color
[x] Color(r, g, b) / Color(r, g, b, a) - writes `#RRGGBB[AA]` for 3MF
[] Color from a name
[] Color from a hex string
[] color as a transform
