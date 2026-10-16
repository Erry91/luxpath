//! Illuminance calculations.
//!
//! M0 has the core of the engine: **direct** horizontal illuminance at a point from one
//! luminaire, by the inverse-square cosine law:
//!
//! ```text
//! E = I(C, γ) · cos θ / d²
//! ```
//!
//! where `I` is the luminaire's intensity towards the point (cd), `d` the distance (m) and `θ` the
//! angle between the light's direction and the surface normal. Reflected light is not included:
//! emergency-lighting tools leave it out by default (see `docs/research.md`), and inter-reflection
//! is a later milestone (M9).
//!
//! Next (M2): luminaire tilt, grids of points, sums over many luminaires, vertical planes.
//!
//! Layer L3: depends on `luxpath-geom` and `luxpath-photometry` (and later `luxpath-scene`).

use luxpath_geom::{Vec3, wrap_degrees};
use luxpath_photometry::IntensityTable;

/// One luminaire as the calculation sees it.
///
/// M0 supports luminaires pointing straight down (the common case for emergency luminaires),
/// turned about the vertical axis by `rotation_deg` (C0 then points along that heading from +x).
#[derive(Debug, Clone)]
pub struct Luminaire {
    pub position: Vec3,
    pub rotation_deg: f64,
    pub distribution: IntensityTable,
    /// Luminous flux in emergency mode, in lumens (often far below the mains-mode flux).
    pub flux_lm: f64,
    /// Maintenance factor (0–1] applied to the intensity.
    pub maintenance_factor: f64,
}

/// Direct illuminance (lux) on a horizontal surface facing up at `point`.
///
/// Returns `None` when the result is undefined: the point coincides with the luminaire, or an
/// input is not finite. Points above the luminaire's light (θ ≥ 90°) receive 0 lux.
pub fn direct_horizontal_illuminance(luminaire: &Luminaire, point: Vec3) -> Option<f64> {
    if !point.is_finite()
        || !luminaire.position.is_finite()
        || !luminaire.flux_lm.is_finite()
        || !luminaire.maintenance_factor.is_finite()
    {
        return None;
    }
    let to_point = point - luminaire.position;
    let d2 = to_point.length_squared();
    let dir = to_point.normalized()?;
    // Light reaches an upward-facing surface only when it travels downwards.
    let cos_theta = -dir.z;
    if cos_theta <= 0.0 {
        return Some(0.0);
    }
    // γ is measured from the nadir, which for a downward-pointing luminaire is the same angle as θ.
    let gamma = cos_theta.clamp(-1.0, 1.0).acos().to_degrees();
    let c = if dir.x == 0.0 && dir.y == 0.0 {
        0.0
    } else {
        wrap_degrees(dir.y.atan2(dir.x).to_degrees() - luminaire.rotation_deg)
    };
    let candela =
        luminaire.distribution.intensity(c, gamma) * luminaire.flux_lm / 1000.0 * luminaire.maintenance_factor;
    let e = candela * cos_theta / d2;
    e.is_finite().then_some(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn isotropic_at(height: f64) -> Luminaire {
        Luminaire {
            position: Vec3::new(0.0, 0.0, height),
            rotation_deg: 0.0,
            distribution: IntensityTable::isotropic(1000.0).unwrap(),
            flux_lm: 1000.0,
            maintenance_factor: 1.0,
        }
    }

    #[test]
    fn directly_below_is_intensity_over_distance_squared() {
        // 1000 cd at 2 m: E = 1000 / 4 = 250 lx.
        let e = direct_horizontal_illuminance(&isotropic_at(2.0), Vec3::new(0.0, 0.0, 0.0)).unwrap();
        assert!((e - 250.0).abs() < 1e-9);
    }

    #[test]
    fn off_axis_points_follow_the_cosine_law() {
        // Point 2 m across, 2 m down: d² = 8, cos θ = 1/√2 → E = 1000 · 0.7071 / 8.
        let e = direct_horizontal_illuminance(&isotropic_at(2.0), Vec3::new(2.0, 0.0, 0.0)).unwrap();
        assert!((e - 1000.0 * std::f64::consts::FRAC_1_SQRT_2 / 8.0).abs() < 1e-9);
    }

    #[test]
    fn flux_and_maintenance_factor_scale_linearly() {
        let mut lum = isotropic_at(2.0);
        lum.flux_lm = 200.0;
        lum.maintenance_factor = 0.8;
        let e = direct_horizontal_illuminance(&lum, Vec3::default()).unwrap();
        assert!((e - 250.0 * 0.2 * 0.8).abs() < 1e-9);
    }

    #[test]
    fn rotation_selects_the_c_plane() {
        // C0 = 200 cd/klm, C180 = 100 cd/klm, flat in γ.
        let table = IntensityTable::new(vec![0.0, 180.0], vec![0.0, 180.0], vec![200.0, 200.0, 100.0, 100.0]).unwrap();
        let mut lum = isotropic_at(1.0);
        lum.distribution = table;
        let east = Vec3::new(1.0, 0.0, 0.0);
        let unrotated = direct_horizontal_illuminance(&lum, east).unwrap();
        lum.rotation_deg = 180.0;
        let rotated = direct_horizontal_illuminance(&lum, east).unwrap();
        assert!((unrotated - 2.0 * rotated).abs() < 1e-9);
    }

    #[test]
    fn points_level_with_or_above_the_luminaire_get_nothing() {
        let lum = isotropic_at(2.0);
        assert_eq!(direct_horizontal_illuminance(&lum, Vec3::new(5.0, 0.0, 2.0)), Some(0.0));
        assert_eq!(direct_horizontal_illuminance(&lum, Vec3::new(0.0, 0.0, 3.0)), Some(0.0));
    }

    #[test]
    fn degenerate_inputs_are_none_not_a_panic() {
        let lum = isotropic_at(2.0);
        assert_eq!(direct_horizontal_illuminance(&lum, Vec3::new(0.0, 0.0, 2.0)), None);
        assert_eq!(direct_horizontal_illuminance(&lum, Vec3::new(f64::NAN, 0.0, 0.0)), None);
        let mut bad = isotropic_at(2.0);
        bad.flux_lm = f64::INFINITY;
        assert_eq!(direct_horizontal_illuminance(&bad, Vec3::default()), None);
    }
}
