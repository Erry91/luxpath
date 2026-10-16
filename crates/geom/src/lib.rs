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

impl std::ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
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
    fn degrees_wrap_into_the_half_open_range() {
        assert_eq!(wrap_degrees(360.0), 0.0);
        assert_eq!(wrap_degrees(-90.0), 270.0);
        assert_eq!(wrap_degrees(725.0), 5.0);
        assert!(wrap_degrees(-1e-20) < 360.0);
        assert_eq!(wrap_degrees(f64::NAN), 0.0);
    }
}
