use crate::mesh::Point2;

/// Split a simple counterclockwise polygon into triangles.
///
/// Each triangle is three indices into `contour`. The function does not add
/// Steiner points. If the polygon is not simple, the result is empty.
pub(crate) fn triangulate(contour: &[Point2]) -> Vec<[usize; 3]> {
    let n = contour.len();
    if n < 3 {
        return Vec::new();
    }

    let mut remaining: Vec<usize> = (0..n).collect();
    let mut triangles = Vec::with_capacity(n - 2);

    while remaining.len() > 3 {
        let mut ear = None;
        for i in 0..remaining.len() {
            if is_ear(contour, &remaining, i) {
                ear = Some(i);
                break;
            }
        }
        let Some(i) = ear else {
            return Vec::new();
        };
        let len = remaining.len();
        let prev = remaining[(i + len - 1) % len];
        let curr = remaining[i];
        let next = remaining[(i + 1) % len];
        triangles.push([prev, curr, next]);
        remaining.remove(i);
    }

    triangles.push([remaining[0], remaining[1], remaining[2]]);
    triangles
}

/// Twice the signed area of a closed polygon. The value is positive when the
/// vertices go counterclockwise.
pub(crate) fn twice_signed_area(contour: &[Point2]) -> f64 {
    let n = contour.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..n {
        let a = contour[i];
        let b = contour[(i + 1) % n];
        area += a.x * b.y - b.x * a.y;
    }
    area
}

fn is_ear(contour: &[Point2], remaining: &[usize], i: usize) -> bool {
    let n = remaining.len();
    let prev = remaining[(i + n - 1) % n];
    let curr = remaining[i];
    let next = remaining[(i + 1) % n];
    let a = contour[prev];
    let b = contour[curr];
    let c = contour[next];

    if cross(a, b, c) <= 0.0 {
        return false;
    }

    for (j, &idx) in remaining.iter().enumerate() {
        if j == (i + n - 1) % n || j == i || j == (i + 1) % n {
            continue;
        }
        if point_in_triangle(contour[idx], a, b, c) {
            return false;
        }
    }
    true
}

fn cross(a: Point2, b: Point2, c: Point2) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn point_in_triangle(p: Point2, a: Point2, b: Point2, c: Point2) -> bool {
    let c1 = cross(a, b, p);
    let c2 = cross(b, c, p);
    let c3 = cross(c, a, p);
    c1 >= 0.0 && c2 >= 0.0 && c3 >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Point2 {
        Point2::new(x, y)
    }

    #[test]
    fn triangle_is_one_face() {
        let contour = [p(0.0, 0.0), p(10.0, 0.0), p(0.0, 10.0)];
        assert_eq!(triangulate(&contour), vec![[0, 1, 2]]);
    }

    #[test]
    fn convex_quad_splits_into_two_triangles() {
        let contour = [p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)];
        let triangles = triangulate(&contour);
        assert_eq!(triangles.len(), 2);
        assert_eq!(twice_signed_area(&contour), 200.0);
    }

    #[test]
    fn concave_c_has_six_triangles() {
        let contour = [
            p(0.0, 0.0),
            p(10.0, 0.0),
            p(10.0, 2.0),
            p(2.0, 2.0),
            p(2.0, 8.0),
            p(10.0, 8.0),
            p(10.0, 10.0),
            p(0.0, 10.0),
        ];
        let triangles = triangulate(&contour);
        assert_eq!(triangles.len(), 6);
        assert_eq!(twice_signed_area(&contour), 104.0);

        let mut area = 0.0;
        for &[a, b, c] in &triangles {
            area += cross(contour[a], contour[b], contour[c]);
        }
        assert!((area - 104.0).abs() < 1e-9);
    }

    #[test]
    fn fewer_than_three_points_is_empty() {
        assert!(triangulate(&[p(0.0, 0.0), p(1.0, 0.0)]).is_empty());
    }
}
