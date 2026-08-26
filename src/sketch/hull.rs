use std::cmp::Ordering;

use crate::ToLength;
use crate::length::Length;
use crate::mesh::Point2;
use crate::sketch::Sketch;
use crate::sketch::polygon::Polygon;
use crate::sketch::triangulate::twice_signed_area;

/// Convex hull of every sketch, tessellated at `tolerance`.
///
/// An empty iterator or fewer than three unique points yields an empty polygon.
pub fn hull_sketches<I>(sketches: I, tolerance: Length) -> Polygon
where
    I: IntoIterator,
    I::Item: Sketch,
{
    let mut points = Vec::new();
    for sketch in sketches {
        points.extend(sketch.contour(tolerance));
    }
    hull_points(points)
}

pub(crate) fn hull_points(points: impl IntoIterator<Item = Point2>) -> Polygon {
    let hull = convex_hull(points);
    if hull.len() < 3 {
        return Polygon::new(Vec::new());
    }
    Polygon::new(hull.into_iter().map(|p| [p.x.mm(), p.y.mm()]).collect())
}

/// Andrew's monotone chain. The result is counterclockwise and has no
/// repeated first point.
fn convex_hull(points: impl IntoIterator<Item = Point2>) -> Vec<Point2> {
    let mut pts: Vec<Point2> = points.into_iter().collect();
    if pts.len() < 3 {
        return Vec::new();
    }

    pts.sort_by(
        |a, b| match a.x.partial_cmp(&b.x).unwrap_or(Ordering::Equal) {
            Ordering::Equal => a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal),
            other => other,
        },
    );
    pts.dedup();
    if pts.len() < 3 {
        return Vec::new();
    }

    let mut lower = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }

    let mut upper = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }

    lower.pop();
    upper.pop();
    lower.extend(upper);

    if lower.len() < 3 {
        return Vec::new();
    }
    if twice_signed_area(&lower) < 0.0 {
        lower.reverse();
    }
    lower
}

fn cross(o: Point2, a: Point2, b: Point2) -> f64 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToLength;
    use crate::sketch::circle::Circle;
    use crate::sketch::polygon::Polygon;
    use crate::sketch::rectangle::Rectangle;
    use crate::solid::Solid;
    use crate::transform::Pose2;

    fn c_polygon() -> Polygon {
        Polygon::new(vec![
            [0.mm(), 0.mm()],
            [10.mm(), 0.mm()],
            [10.mm(), 2.mm()],
            [2.mm(), 2.mm()],
            [2.mm(), 8.mm()],
            [10.mm(), 8.mm()],
            [10.mm(), 10.mm()],
            [0.mm(), 10.mm()],
        ])
    }

    fn area(contour: &[Point2]) -> f64 {
        twice_signed_area(contour) / 2.0
    }

    #[test]
    fn rectangle_hull_is_itself() {
        let contour = Rectangle::new(20.mm(), 10.mm())
            .hull(1.mm())
            .contour(1.mm());

        assert_eq!(contour.len(), 4);
        assert!((area(&contour) - 200.0).abs() < 1e-9);
        assert!(twice_signed_area(&contour) > 0.0);
    }

    #[test]
    fn c_polygon_hull_is_the_bounding_square() {
        let contour = c_polygon().hull(1.mm()).contour(1.mm());

        assert_eq!(contour.len(), 4);
        assert!((area(&contour) - 100.0).abs() < 1e-9);
        assert!(twice_signed_area(&contour) > 0.0);
    }

    #[test]
    fn two_translated_circles_make_a_stadium() {
        let r = 5.0;
        let d = 20.0;
        let contour = hull_sketches(
            [
                Circle::new(5.mm()).posed(),
                Circle::new(5.mm()).translate(20.mm(), Length::ZERO),
            ],
            200.um(),
        )
        .contour(200.um());

        let expected = std::f64::consts::PI * r * r + 2.0 * r * d;
        let actual = area(&contour);
        assert!(twice_signed_area(&contour) > 0.0);
        assert!(
            (actual - expected).abs() < expected * 0.05,
            "stadium area {actual} is not near {expected}"
        );
    }

    #[test]
    fn extruded_circle_hull_has_positive_volume() {
        let mesh = hull_sketches(
            [
                Circle::new(5.mm()).posed(),
                Circle::new(5.mm()).translate(20.mm(), Length::ZERO),
            ],
            200.um(),
        )
        .extrude(8.mm())
        .mesh(200.um());

        assert!(mesh.signed_volume() > 0.0);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.triangles.is_empty());
    }

    #[test]
    fn fewer_than_three_points_yields_an_empty_polygon() {
        let hull = Polygon::new(vec![[0.mm(), 0.mm()], [1.mm(), 0.mm()]]).hull(1.mm());
        assert!(hull.contour(1.mm()).is_empty());
        assert!(
            hull_sketches(Vec::<Rectangle>::new(), 1.mm())
                .contour(1.mm())
                .is_empty()
        );
    }
}
