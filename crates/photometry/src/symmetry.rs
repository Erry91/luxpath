//! Symmetry expansion: files store only the C planes a luminaire's symmetry does not repeat, and the
//! rest is their mirror image.

use crate::{IntensityTable, PhotometryError};

/// Which C planes a file stores. Angles are the file's own (C0 is the file's C0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symmetry {
    /// Every plane, from C0 to below C360 (a C360 plane, if present, repeats C0).
    None,
    /// One plane, the same all around the vertical axis.
    Rotational,
    /// C0 to C180, mirrored about the C0–C180 plane: C(360 − c) = C(c).
    PlaneC0C180,
    /// C90 to C270, mirrored about the C90–C270 plane: C(180 − c) = C(c).
    PlaneC90C270,
    /// C0 to C90, mirrored about both planes.
    Quadrants,
}

/// Angles closer than this are the same plane (files write angles with a few decimals).
const SAME_ANGLE_DEG: f64 = 1e-6;

impl Symmetry {
    /// The first and last stored C angle this symmetry expects.
    pub fn stored_range(self) -> (f64, f64) {
        match self {
            Self::None => (0.0, 360.0),
            Self::Rotational => (0.0, 0.0),
            Self::PlaneC0C180 => (0.0, 180.0),
            Self::PlaneC90C270 => (90.0, 270.0),
            Self::Quadrants => (0.0, 90.0),
        }
    }

    /// The angles an angle `c` (stored) also stands for, `c` included, each in `[0, 360)`.
    fn images(self, c: f64) -> Vec<f64> {
        let images = match self {
            Self::None | Self::Rotational => vec![c],
            Self::PlaneC0C180 => vec![c, 360.0 - c],
            Self::PlaneC90C270 => vec![c, 180.0 - c],
            Self::Quadrants => vec![c, 180.0 - c, 180.0 + c, 360.0 - c],
        };
        images.into_iter().map(|a| a.rem_euclid(360.0)).filter(|a| *a < 360.0 - SAME_ANGLE_DEG).collect()
    }

    /// The full table from the stored planes: `c_angles` (stored, increasing) and their `values`
    /// (one plane of `gamma_angles.len()` values after another).
    ///
    /// Errors when the stored angles do not span what the symmetry needs. A full table without a C0
    /// plane (possible when C90–C270 planes do not include C180) gets one, interpolated across the
    /// wrap from the planes either side.
    pub fn expand(
        self,
        c_angles: &[f64],
        gamma_angles: Vec<f64>,
        values: &[f64],
    ) -> Result<IntensityTable, ExpandError> {
        let n_gamma = gamma_angles.len();
        let (Some(first), Some(last)) = (c_angles.first(), c_angles.last()) else { return Err(ExpandError::NoPlanes) };
        let (want_first, want_last) = self.stored_range();
        let near = |a: f64, b: f64| (a - b).abs() < SAME_ANGLE_DEG;
        let spans = match self {
            Self::Rotational => c_angles.len() == 1,
            // Every plane: from C0 up to C360 at most.
            Self::None => near(*first, want_first) && *last <= want_last + SAME_ANGLE_DEG,
            _ => near(*first, want_first) && near(*last, want_last),
        };
        if !spans {
            return Err(ExpandError::Span);
        }
        if c_angles.windows(2).any(|w| w.first() >= w.get(1)) {
            return Err(ExpandError::NotIncreasing);
        }
        let plane = |i: usize| -> Result<&[f64], ExpandError> {
            let start = i.checked_mul(n_gamma).ok_or(ExpandError::Shape)?;
            values.get(start..start.saturating_add(n_gamma)).ok_or(ExpandError::Shape)
        };
        if c_angles.len().checked_mul(n_gamma) != Some(values.len()) {
            return Err(ExpandError::Shape);
        }

        // Every angle with the stored plane it repeats; the first stored plane to claim an angle wins.
        let mut planes: Vec<(f64, usize)> = Vec::new();
        for (i, c) in c_angles.iter().enumerate() {
            let c = if self == Self::Rotational { 0.0 } else { *c };
            for image in self.images(c) {
                planes.push((image, i));
            }
        }
        planes.sort_by(|a, b| a.0.total_cmp(&b.0));
        planes.dedup_by(|later, earlier| near(later.0, earlier.0));

        let mut angles = Vec::with_capacity(planes.len() + 1);
        let mut full = Vec::with_capacity((planes.len() + 1).saturating_mul(n_gamma));
        if let (Some((lo, lo_i)), Some((hi, hi_i))) = (planes.first(), planes.last())
            && !near(*lo, 0.0)
        {
            // No C0 plane: interpolate between the last plane (as hi − 360) and the first.
            let t = (360.0 - hi) / (360.0 - hi + lo);
            let (lo_row, hi_row) = (plane(*lo_i)?, plane(*hi_i)?);
            angles.push(0.0);
            full.extend(hi_row.iter().zip(lo_row).map(|(h, l)| h + (l - h) * t));
        }
        for (angle, i) in &planes {
            angles.push(if near(*angle, 0.0) { 0.0 } else { *angle });
            full.extend_from_slice(plane(*i)?);
        }
        IntensityTable::new(angles, gamma_angles, full).map_err(ExpandError::Table)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpandError {
    NoPlanes,
    /// The stored C angles do not start and end where the symmetry needs.
    Span,
    NotIncreasing,
    /// The values are not one plane per angle.
    Shape,
    Table(PhotometryError),
}

impl ExpandError {
    pub(crate) fn reason(&self) -> &'static str {
        match self {
            Self::NoPlanes => "has no C planes",
            Self::Span => "do not span what the file's symmetry needs",
            Self::NotIncreasing => "are not strictly increasing",
            Self::Shape => "do not match the number of values",
            Self::Table(_) => "do not form a valid table",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(planes: &[f64]) -> Vec<f64> {
        // Two γ angles per plane; plane i has values 10i and 10i + 1.
        planes.iter().enumerate().flat_map(|(i, _)| [10.0 * i as f64, 10.0 * i as f64 + 1.0]).collect()
    }

    fn expand(s: Symmetry, c: &[f64]) -> IntensityTable {
        s.expand(c, vec![0.0, 90.0], &rows(c)).unwrap()
    }

    #[test]
    fn quadrants_mirror_into_four() {
        let t = expand(Symmetry::Quadrants, &[0.0, 30.0, 90.0]);
        assert_eq!(t.c_angles(), &[0.0, 30.0, 90.0, 150.0, 180.0, 210.0, 270.0, 330.0]);
        // C150 = C30 (plane 1), C180 = C0, C210 = C30, C330 = C30.
        let first: Vec<f64> = t.values().chunks(2).map(|p| p[0]).collect();
        assert_eq!(first, vec![0.0, 10.0, 20.0, 10.0, 0.0, 10.0, 20.0, 10.0]);
    }

    #[test]
    fn half_planes_mirror_about_their_plane() {
        let t = expand(Symmetry::PlaneC0C180, &[0.0, 90.0, 180.0]);
        assert_eq!(t.c_angles(), &[0.0, 90.0, 180.0, 270.0]);
        assert_eq!(t.intensity(270.0, 0.0), t.intensity(90.0, 0.0));
        let t = expand(Symmetry::PlaneC90C270, &[90.0, 135.0, 180.0, 225.0, 270.0]);
        // C0 = C180, C45 = C135, C315 = C225.
        assert_eq!(t.c_angles(), &[0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0]);
        assert_eq!(t.intensity(0.0, 0.0), t.intensity(180.0, 0.0));
        assert_eq!(t.intensity(45.0, 0.0), t.intensity(135.0, 0.0));
        assert_eq!(t.intensity(315.0, 0.0), t.intensity(225.0, 0.0));
    }

    #[test]
    fn a_c90_c270_file_without_c180_gets_an_interpolated_c0() {
        // Stored C90, C160, C200, C270: images C20 (of C160) and C340 (of C200), no C0.
        let t = expand(Symmetry::PlaneC90C270, &[90.0, 160.0, 200.0, 270.0]);
        assert_eq!(t.c_angles().first(), Some(&0.0));
        // C0 lies halfway between C340 (= C200, value 20) and C20 (= C160, value 10): 15.
        assert!((t.intensity(0.0, 0.0) - 15.0).abs() < 1e-9);
    }

    #[test]
    fn full_and_rotational() {
        let t = expand(Symmetry::None, &[0.0, 120.0, 240.0, 360.0]);
        assert_eq!(t.c_angles(), &[0.0, 120.0, 240.0]);
        let t = expand(Symmetry::Rotational, &[0.0]);
        assert_eq!(t.c_angles(), &[0.0]);
    }

    #[test]
    fn wrong_spans_are_errors() {
        let e = |s: Symmetry, c: &[f64]| s.expand(c, vec![0.0, 90.0], &rows(c)).unwrap_err();
        assert_eq!(e(Symmetry::Quadrants, &[0.0, 45.0]), ExpandError::Span);
        assert_eq!(e(Symmetry::PlaneC0C180, &[10.0, 180.0]), ExpandError::Span);
        assert_eq!(e(Symmetry::Rotational, &[0.0, 10.0]), ExpandError::Span);
        assert_eq!(e(Symmetry::None, &[0.0, 400.0]), ExpandError::Span);
        assert_eq!(e(Symmetry::Quadrants, &[0.0, 60.0, 30.0, 90.0]), ExpandError::NotIncreasing);
        assert_eq!(Symmetry::None.expand(&[0.0], vec![0.0], &[1.0, 2.0]).unwrap_err(), ExpandError::Shape);
        assert_eq!(Symmetry::None.expand(&[], vec![0.0], &[]).unwrap_err(), ExpandError::NoPlanes);
    }
}
