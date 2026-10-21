//! Luminous intensity distributions in the C-γ coordinate system.
//!
//! A luminaire's light output is described by a table of luminous intensities over two angles:
//! - **C** (0–360°): the half-plane around the luminaire's vertical axis. C0 points along the
//!   luminaire's own x axis, C90 along its y axis.
//! - **γ** (0–180°): the angle from straight down (γ0 = nadir, γ90 = horizontal, γ180 = zenith).
//!
//! Values are stored **per 1000 lm of lamp flux** (cd/klm), the form EULUMDAT uses; IES files
//! (absolute candela) are normalised to it on read. Multiply by `flux / 1000` for candela.
//!
//! Files are read into a [`Photometry`] (the table, the flux it is relative to, tilt data) by
//! [`read`], [`read_ies`] (IES LM-63, 1986 to 2019) and [`read_eulumdat`] (EULUMDAT, .ldt); the part
//! of the distribution a file stores is mirrored into a full table ([`Symmetry`]). Every file is
//! untrusted: readers return a [`ReadError`] with the line it concerns, never panic, and cap what
//! they allocate.
//!
//! Layer L1: depends on `luxpath-geom`. No I/O: readers take the file's bytes.

mod ies;
mod ldt;
mod model;
mod symmetry;
mod text;

pub use ies::{IesEdition, read_ies};
pub use ldt::read_eulumdat;
pub use model::{EmergencyOutput, Format, Photometry, Tilt, read};
pub use symmetry::Symmetry;
pub use text::{MAX_FILE_BYTES, ReadError, ReadErrorKind};

use luxpath_geom::wrap_degrees;

/// Upper bound on table size, so a malformed file cannot make us allocate without limit.
/// Real files are far smaller (a full 5°×1° table is 72 × 181 = 13,032 values).
pub const MAX_TABLE_VALUES: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq)]
pub enum PhotometryError {
    /// No C planes or no γ angles.
    Empty,
    /// More values than [`MAX_TABLE_VALUES`].
    TooLarge { values: usize },
    /// `values.len()` is not `c_angles.len() * gamma_angles.len()`.
    ShapeMismatch { expected: usize, found: usize },
    /// An angle list is not strictly increasing, starts in the wrong place or leaves its range.
    BadAngles { which: &'static str, reason: &'static str },
    /// A value is negative, NaN or infinite.
    BadValue { index: usize },
    /// A flux or a factor is zero, negative, not a number, or out of its range.
    BadFlux { reason: &'static str },
}

impl std::fmt::Display for PhotometryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "the intensity table has no angles"),
            Self::TooLarge { values } => {
                write!(f, "the intensity table has {values} values (limit {MAX_TABLE_VALUES})")
            }
            Self::ShapeMismatch { expected, found } => {
                write!(f, "expected {expected} intensity values, found {found}")
            }
            Self::BadAngles { which, reason } => write!(f, "{which} angles: {reason}"),
            Self::BadValue { index } => write!(f, "intensity value #{index} is negative or not a number"),
            Self::BadFlux { reason } => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for PhotometryError {}

/// A complete (already symmetry-expanded) intensity table in cd/klm.
///
/// Invariants, checked by [`IntensityTable::new`]:
/// - `c_angles` is strictly increasing, starts at 0 and stays below 360 (C360 is C0). A single
///   plane means the distribution is rotationally symmetric.
/// - `gamma_angles` is strictly increasing within `[0, 180]`.
/// - `values` holds `c_angles.len() * gamma_angles.len()` finite, non-negative values, one C
///   plane after another.
#[derive(Debug, Clone, PartialEq)]
pub struct IntensityTable {
    c_angles: Vec<f64>,
    gamma_angles: Vec<f64>,
    values: Vec<f64>,
}

impl IntensityTable {
    pub fn new(c_angles: Vec<f64>, gamma_angles: Vec<f64>, values: Vec<f64>) -> Result<Self, PhotometryError> {
        if c_angles.is_empty() || gamma_angles.is_empty() {
            return Err(PhotometryError::Empty);
        }
        let expected =
            c_angles.len().checked_mul(gamma_angles.len()).ok_or(PhotometryError::TooLarge { values: usize::MAX })?;
        if expected > MAX_TABLE_VALUES {
            return Err(PhotometryError::TooLarge { values: expected });
        }
        if values.len() != expected {
            return Err(PhotometryError::ShapeMismatch { expected, found: values.len() });
        }
        check_angles("C", &c_angles, 0.0, 360.0, false)?;
        check_angles("gamma", &gamma_angles, 0.0, 180.0, true)?;
        if c_angles.first() != Some(&0.0) {
            return Err(PhotometryError::BadAngles { which: "C", reason: "the first C plane must be C0" });
        }
        if let Some(index) = values.iter().position(|v| !v.is_finite() || *v < 0.0) {
            return Err(PhotometryError::BadValue { index });
        }
        Ok(Self { c_angles, gamma_angles, values })
    }

    /// A distribution with the same intensity in every direction (useful for tests and
    /// hand calculations).
    pub fn isotropic(cd_per_klm: f64) -> Result<Self, PhotometryError> {
        Self::new(vec![0.0], vec![0.0, 180.0], vec![cd_per_klm, cd_per_klm])
    }

    /// The C angles in degrees, from C0, strictly increasing, below 360.
    pub fn c_angles(&self) -> &[f64] {
        &self.c_angles
    }

    /// The γ angles in degrees, strictly increasing, within `[0, 180]`.
    pub fn gamma_angles(&self) -> &[f64] {
        &self.gamma_angles
    }

    /// The values, one C plane after another.
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// The same table with every value multiplied by `factor` (finite, non-negative).
    pub fn scaled(&self, factor: f64) -> Result<Self, PhotometryError> {
        if !factor.is_finite() || factor < 0.0 {
            return Err(PhotometryError::BadFlux { reason: "a scale factor is negative or not a number" });
        }
        Self::new(self.c_angles.clone(), self.gamma_angles.clone(), self.values.iter().map(|v| v * factor).collect())
    }

    /// The flux the table emits, ∫ I dΩ over the sphere: lumens for a table in candela, lumens per
    /// 1000 lamp lumens for a table in cd/klm (so 1000 × the light output ratio).
    ///
    /// Zonal sum: each γ row is averaged over C (trapezoids around the circle), and each band
    /// between two rows gets the mean of its rows times its solid angle, 2π (cos γ₁ − cos γ₂).
    /// Exact for distributions constant within each band (an isotropic I gives 4π I); otherwise the
    /// error shrinks with the angle step (about 1e-4 for a Lambertian table at 1° steps).
    pub fn integrated_flux(&self) -> f64 {
        let mean_over_c = |g: usize| -> f64 {
            let n = self.c_angles.len();
            if n == 1 {
                return self.value(0, g);
            }
            let mut sum = 0.0;
            for i in 0..n {
                let (c0, v0) = (self.c_angles.get(i).copied().unwrap_or(0.0), self.value(i, g));
                let (c1, v1) = match self.c_angles.get(i + 1) {
                    Some(c1) => (*c1, self.value(i + 1, g)),
                    None => (360.0, self.value(0, g)),
                };
                sum += (v0 + v1) / 2.0 * (c1 - c0);
            }
            sum / 360.0
        };
        let means: Vec<f64> = (0..self.gamma_angles.len()).map(mean_over_c).collect();
        let mut flux = 0.0;
        for (g, pair) in self.gamma_angles.windows(2).enumerate() {
            let (Some(g0), Some(g1)) = (pair.first(), pair.get(1)) else { continue };
            let (Some(m0), Some(m1)) = (means.get(g), means.get(g + 1)) else { continue };
            let solid_angle = std::f64::consts::TAU * (g0.to_radians().cos() - g1.to_radians().cos());
            flux += (m0 + m1) / 2.0 * solid_angle;
        }
        flux
    }

    /// Intensity in cd/klm towards `(c, gamma)` in degrees, bilinearly interpolated.
    ///
    /// `c` wraps around 360°; `gamma` is clamped to `[0, 180]`. Outside the table's γ range the
    /// intensity is 0: a file measured from γ0 to γ90 says the luminaire emits no upward light.
    /// Non-finite angles return 0.
    pub fn intensity(&self, c: f64, gamma: f64) -> f64 {
        if !c.is_finite() || !gamma.is_finite() {
            return 0.0;
        }
        let gamma = gamma.clamp(0.0, 180.0);
        let (Some(first), Some(last)) = (self.gamma_angles.first(), self.gamma_angles.last()) else { return 0.0 };
        if gamma < *first || gamma > *last {
            return 0.0;
        }
        let (g0, g1, gt) = bracket(&self.gamma_angles, gamma);
        let c = wrap_degrees(c);
        // Find the C planes either side of `c`, wrapping from the last plane back to C0 (= C360).
        let n_c = self.c_angles.len();
        let lower = self.c_angles.iter().rposition(|a| *a <= c).unwrap_or(0);
        let (upper, c_lo, c_hi) = match self.c_angles.get(lower + 1) {
            Some(hi) => (lower + 1, self.c_angles.get(lower).copied().unwrap_or(0.0), *hi),
            None => (0, self.c_angles.get(lower).copied().unwrap_or(0.0), 360.0),
        };
        let ct = if n_c == 1 || c_hi <= c_lo { 0.0 } else { ((c - c_lo) / (c_hi - c_lo)).clamp(0.0, 1.0) };
        let at_lower = lerp(self.value(lower, g0), self.value(lower, g1), gt);
        let at_upper = lerp(self.value(upper, g0), self.value(upper, g1), gt);
        lerp(at_lower, at_upper, ct)
    }

    fn value(&self, c_index: usize, gamma_index: usize) -> f64 {
        c_index
            .checked_mul(self.gamma_angles.len())
            .and_then(|row| row.checked_add(gamma_index))
            .and_then(|i| self.values.get(i))
            .copied()
            .unwrap_or(0.0)
    }
}

fn check_angles(
    which: &'static str,
    angles: &[f64],
    min: f64,
    max: f64,
    max_inclusive: bool,
) -> Result<(), PhotometryError> {
    for (i, a) in angles.iter().enumerate() {
        let in_range = a.is_finite() && *a >= min && if max_inclusive { *a <= max } else { *a < max };
        if !in_range {
            return Err(PhotometryError::BadAngles { which, reason: "an angle is outside the allowed range" });
        }
        if i > 0 && angles.get(i - 1).is_some_and(|prev| a <= prev) {
            return Err(PhotometryError::BadAngles { which, reason: "angles must be strictly increasing" });
        }
    }
    Ok(())
}

/// The indices either side of `x` in the increasing list `angles`, and the fraction between them.
fn bracket(angles: &[f64], x: f64) -> (usize, usize, f64) {
    let last = angles.len().saturating_sub(1);
    let (Some(first_a), Some(last_a)) = (angles.first(), angles.last()) else { return (0, 0, 0.0) };
    if x <= *first_a {
        return (0, 0, 0.0);
    }
    if x >= *last_a {
        return (last, last, 0.0);
    }
    let hi = angles.iter().position(|a| *a > x).unwrap_or(last);
    let lo = hi.saturating_sub(1);
    let (a_lo, a_hi) = (angles.get(lo).copied().unwrap_or(0.0), angles.get(hi).copied().unwrap_or(0.0));
    let t = if a_hi > a_lo { (x - a_lo) / (a_hi - a_lo) } else { 0.0 };
    (lo, hi, t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_plane_table() -> IntensityTable {
        // C0 and C180, γ 0/90/180. C0 is twice as bright as C180.
        IntensityTable::new(vec![0.0, 180.0], vec![0.0, 90.0, 180.0], vec![200.0, 100.0, 0.0, 100.0, 50.0, 0.0])
            .unwrap()
    }

    #[test]
    fn isotropic_tables_are_the_same_everywhere() {
        let t = IntensityTable::isotropic(500.0).unwrap();
        for (c, g) in [(0.0, 0.0), (123.0, 45.0), (359.9, 180.0), (-30.0, 90.0)] {
            assert_eq!(t.intensity(c, g), 500.0);
        }
    }

    #[test]
    fn gamma_is_interpolated_and_clamped() {
        let t = two_plane_table();
        assert_eq!(t.intensity(0.0, 45.0), 150.0);
        assert_eq!(t.intensity(0.0, -10.0), 200.0);
        assert_eq!(t.intensity(0.0, 200.0), 0.0);
    }

    #[test]
    fn there_is_no_light_outside_the_measured_gamma_range() {
        // Measured from γ0 to γ90 only, with light at γ90: none above it.
        let down = IntensityTable::new(vec![0.0], vec![0.0, 90.0], vec![100.0, 40.0]).unwrap();
        assert_eq!(down.intensity(0.0, 90.0), 40.0);
        assert_eq!(down.intensity(0.0, 90.5), 0.0);
        // Measured from γ90 to γ180 only (an uplight): none below γ90.
        let up = IntensityTable::new(vec![0.0], vec![90.0, 180.0], vec![40.0, 100.0]).unwrap();
        assert_eq!(up.intensity(0.0, 89.5), 0.0);
        assert_eq!(up.intensity(0.0, 135.0), 70.0);
    }

    #[test]
    fn integrated_flux_is_exact_for_isotropic_and_close_for_lambertian() {
        // Isotropic: Φ = 4π I.
        let iso = IntensityTable::isotropic(100.0).unwrap();
        assert!((iso.integrated_flux() - 400.0 * std::f64::consts::PI).abs() < 1e-9);
        // Lambertian, I = I₀ cos γ below the horizon: Φ = π I₀ (∫₀^{π/2} cos γ sin γ dγ · 2π = π).
        let gammas: Vec<f64> = (0..=180).map(f64::from).collect();
        let values: Vec<f64> =
            gammas.iter().map(|g| if *g <= 90.0 { 1000.0 * g.to_radians().cos() } else { 0.0 }).collect();
        let lambert = IntensityTable::new(
            vec![0.0, 90.0, 180.0, 270.0],
            gammas.clone(),
            [values.clone(), values.clone(), values.clone(), values].concat(),
        )
        .unwrap();
        let expected = 1000.0 * std::f64::consts::PI;
        assert!((lambert.integrated_flux() - expected).abs() / expected < 2e-4, "{}", lambert.integrated_flux());
        // Doubling C0 and C180 against C90 and C270 averages around the circle by trapezoids:
        // the mean over C of 2, 1, 2, 1 is 1.5.
        let uneven = IntensityTable::new(
            vec![0.0, 90.0, 180.0, 270.0],
            vec![0.0, 180.0],
            vec![2.0, 2.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0],
        )
        .unwrap();
        assert!((uneven.integrated_flux() - 1.5 * 4.0 * std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn scaling_multiplies_every_value_and_refuses_bad_factors() {
        let t = two_plane_table().scaled(0.5).unwrap();
        assert_eq!(t.values(), &[100.0, 50.0, 0.0, 50.0, 25.0, 0.0]);
        assert!(two_plane_table().scaled(-1.0).is_err());
        assert!(two_plane_table().scaled(f64::NAN).is_err());
    }

    #[test]
    fn c_is_interpolated_and_wraps_back_to_c0() {
        let t = two_plane_table();
        assert_eq!(t.intensity(90.0, 0.0), 150.0);
        // Between C180 (100) and C360 = C0 (200).
        assert_eq!(t.intensity(270.0, 0.0), 150.0);
        assert_eq!(t.intensity(-90.0, 0.0), 150.0);
    }

    #[test]
    fn non_finite_angles_give_zero() {
        let t = two_plane_table();
        assert_eq!(t.intensity(f64::NAN, 0.0), 0.0);
        assert_eq!(t.intensity(0.0, f64::INFINITY), 0.0);
    }

    #[test]
    fn malformed_tables_are_rejected() {
        use PhotometryError as E;
        assert_eq!(IntensityTable::new(vec![], vec![0.0], vec![]), Err(E::Empty));
        assert!(matches!(IntensityTable::new(vec![0.0], vec![0.0, 90.0], vec![1.0]), Err(E::ShapeMismatch { .. })));
        assert!(matches!(IntensityTable::new(vec![0.0, 0.0], vec![0.0], vec![1.0, 1.0]), Err(E::BadAngles { .. })));
        assert!(matches!(IntensityTable::new(vec![10.0], vec![0.0], vec![1.0]), Err(E::BadAngles { .. })));
        assert!(matches!(IntensityTable::new(vec![0.0, 360.0], vec![0.0], vec![1.0, 1.0]), Err(E::BadAngles { .. })));
        assert!(matches!(IntensityTable::new(vec![0.0], vec![0.0, 181.0], vec![1.0, 1.0]), Err(E::BadAngles { .. })));
        assert_eq!(IntensityTable::new(vec![0.0], vec![0.0], vec![-1.0]), Err(E::BadValue { index: 0 }));
        assert_eq!(IntensityTable::new(vec![0.0], vec![0.0], vec![f64::NAN]), Err(E::BadValue { index: 0 }));
    }

    #[test]
    fn oversized_tables_are_rejected_before_allocating() {
        let c: Vec<f64> = (0..2000).map(|i| f64::from(i) * 0.18).collect();
        let g: Vec<f64> = (0..1000).map(|i| f64::from(i) * 0.18).collect();
        assert!(matches!(IntensityTable::new(c, g, vec![]), Err(PhotometryError::TooLarge { .. })));
    }
}
