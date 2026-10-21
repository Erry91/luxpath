//! Geometry primitives shared by every luxpath crate.
//!
//! Conventions, used everywhere in the workspace:
//! - lengths are in metres;
//! - the scene frame is right-handed with **z up**; the floor is usually `z = 0`;
//! - angles in public APIs are in **degrees** (photometric files use degrees), and are converted to
//!   radians only inside calculations.
//!
//! Layer L0: depends on nothing.

/// A point or vector in the scene frame, in metres.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const UP: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn scale(self, k: f64) -> Vec3 {
        Vec3::new(self.x * k, self.y * k, self.z * k)
    }

    /// True when every component is finite (no NaN or infinity).
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// The unit vector in the same direction, or `None` for a zero-length or non-finite vector.
    pub fn normalized(self) -> Option<Vec3> {
        let len = self.length();
        (len.is_finite() && len > 0.0).then(|| self.scale(1.0 / len))
    }
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

/// A point or vector in plan (the floor plane), in metres.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// This point at height `z`.
    pub fn at(self, z: f64) -> Vec3 {
        Vec3::new(self.x, self.y, z)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

/// The edges of a closed polygon: each vertex with the next, the last with the first.
fn edges(polygon: &[Vec2]) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
    polygon.iter().zip(polygon.iter().cycle().skip(1)).map(|(a, b)| (*a, *b))
}

/// Whether `p` lies inside the closed `polygon` (even-odd rule; points exactly on an edge may go
/// either way). Fewer than three vertices enclose nothing.
pub fn point_in_polygon(p: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let mut inside = false;
    for (a, b) in edges(polygon) {
        if (a.y > p.y) != (b.y > p.y) {
            let x = a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if p.x < x {
                inside = !inside;
            }
        }
    }
    inside
}

/// The distance from `p` to the segment from `a` to `b`.
pub fn distance_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f64 {
    let ab = b - a;
    let len2 = ab.x * ab.x + ab.y * ab.y;
    let t = if len2 > 0.0 { (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / len2).clamp(0.0, 1.0) } else { 0.0 };
    (p - Vec2::new(a.x + ab.x * t, a.y + ab.y * t)).length()
}

/// The distance from `p` to the nearest edge of the closed `polygon` (infinite for no edges).
pub fn distance_to_boundary(p: Vec2, polygon: &[Vec2]) -> f64 {
    edges(polygon).map(|(a, b)| distance_to_segment(p, a, b)).fold(f64::INFINITY, f64::min)
}

/// Wraps an angle in degrees into `[0, 360)`. Non-finite input returns `0.0`.
pub fn wrap_degrees(deg: f64) -> f64 {
    if !deg.is_finite() {
        return 0.0;
    }
    let wrapped = deg.rem_euclid(360.0);
    // rem_euclid can round up to exactly 360.0 for tiny negative inputs.
    if wrapped >= 360.0 { 0.0 } else { wrapped }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizing_a_zero_or_non_finite_vector_returns_none() {
        assert_eq!(Vec3::default().normalized(), None);
        assert_eq!(Vec3::new(f64::NAN, 0.0, 0.0).normalized(), None);
        assert_eq!(Vec3::new(f64::INFINITY, 0.0, 0.0).normalized(), None);
    }

    #[test]
    fn normalized_vectors_have_unit_length() {
        let n = Vec3::new(3.0, 4.0, 12.0).normalized().unwrap();
        assert!((n.length() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn points_are_inside_or_outside_an_l_shaped_room() {
        // An L: a 4 × 2 rectangle with a 2 × 2 square on its left half.
        let l = [
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(4.0, 2.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(2.0, 4.0),
            Vec2::new(0.0, 4.0),
        ];
        assert!(point_in_polygon(Vec2::new(1.0, 3.0), &l));
        assert!(point_in_polygon(Vec2::new(3.0, 1.0), &l));
        assert!(!point_in_polygon(Vec2::new(3.0, 3.0), &l));
        assert!(!point_in_polygon(Vec2::new(-1.0, 1.0), &l));
        assert!(!point_in_polygon(Vec2::new(1.0, 1.0), &l[..2]));
        // The nearest edge of (1, 3) is x = 0 or y = 4 or x = 2: 1 m.
        assert_eq!(distance_to_boundary(Vec2::new(1.0, 3.0), &l), 1.0);
        assert_eq!(distance_to_boundary(Vec2::new(3.5, 1.0), &l), 0.5);
    }

    #[test]
    fn segment_distances_clamp_to_the_ends() {
        let (a, b) = (Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0));
        assert_eq!(distance_to_segment(Vec2::new(1.0, 3.0), a, b), 3.0);
        assert_eq!(distance_to_segment(Vec2::new(5.0, 4.0), a, b), 5.0);
        assert_eq!(distance_to_segment(Vec2::new(0.0, 1.0), a, a), 1.0);
    }

    #[test]
    fn degrees_wrap_into_the_half_open_range() {
        assert_eq!(wrap_degrees(360.0), 0.0);
        assert_eq!(wrap_degrees(-90.0), 270.0);
        assert_eq!(wrap_degrees(725.0), 5.0);
        assert!(wrap_degrees(-1e-20) < 360.0);
        assert_eq!(wrap_degrees(f64::NAN), 0.0);
    }
}
