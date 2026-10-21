//! Illuminance calculations.
//!
//! The engine computes **direct** illuminance on a surface with normal `n` at a point, from each
//! luminaire, by the inverse-square cosine law, and sums it over luminaires:
//!
//! ```text
//! E = I(C, γ) · cos θ / d²
//! ```
//!
//! where `I` is the luminaire's intensity towards the point (cd), `d` the distance (m) and `θ` the
//! angle between the arriving light and `n` (light from behind the surface counts 0). A luminaire
//! whose line of sight to the point crosses a [`Wall`] contributes nothing there. Reflected light is
//! not included: emergency-lighting tools leave it out by default (see `docs/research.md`), and
//! inter-reflection is a later milestone (M9).
//!
//! - [`Luminaire`]: a placed, oriented (heading and tilt) distribution with its flux;
//!   [`Luminaire::from_photometry`] applies the file's tilt factors.
//! - [`Grid`]: points with one surface normal: over an area (a polygon, less a border), along a
//!   route (its full width), or given one by one (vertical illuminance at equipment).
//! - [`illuminance`] at every grid point, summed over luminaires ([`illuminance_with_walls`] with
//!   walls in the way); [`Stats`] (min, mean, max, the
//!   uniformity ratios).
//!
//! An undefined result (a point at a luminaire's centre, a non-finite input) is `None` for that
//! point, and statistics over it are `None`: a check reports "cannot verify", never a value.
//!
//! Layer L3: depends on `luxpath-geom` and `luxpath-photometry` (and later `luxpath-scene`).

mod grid;
mod stats;

pub use grid::{Grid, GridError, MAX_GRID_POINTS};
pub use stats::Stats;

pub use luxpath_geom::Wall;
use luxpath_geom::{Vec3, wrap_degrees};
use luxpath_photometry::{IntensityTable, Photometry};

/// One luminaire as the calculation sees it.
///
/// Orientation: the luminaire's photometric frame (C0 along its x axis, C90 along y, γ0 straight
/// down its −z axis) is first **tilted** by `tilt_deg` about its own y axis, so that the nadir swings
/// towards C0 (90° aims the light horizontally, as for a wall-mounted luminaire whose photometry
/// was measured pointing down), then turned by `rotation_deg` about the vertical (C0 then points
/// along that heading, counter-clockwise from +x).
#[derive(Debug, Clone)]
pub struct Luminaire {
    pub position: Vec3,
    pub rotation_deg: f64,
    pub tilt_deg: f64,
    pub distribution: IntensityTable,
    /// Luminous flux in the mode being checked, in lumens (in emergency mode often far below the
    /// mains-mode flux).
    pub flux_lm: f64,
    /// Maintenance factor (0–1] applied to the intensity.
    pub maintenance_factor: f64,
}

/// Why a luminaire cannot be placed.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcError {
    /// The file's tilt table does not cover this tilt, so the light output there is unknown.
    TiltOutsideTable { tilt_deg: f64 },
    /// A factor scaled the table out of range.
    Photometry(luxpath_photometry::PhotometryError),
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TiltOutsideTable { tilt_deg } => {
                write!(f, "the photometric file's tilt table does not cover a tilt of {tilt_deg}°")
            }
            Self::Photometry(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CalcError {}

impl Luminaire {
    /// A luminaire from a file's photometry, with the file's tilt factor (IES `TILT=INCLUDE`) for
    /// `tilt_deg` applied to the distribution. A tilt the table does not cover is an error, not a
    /// guess.
    pub fn from_photometry(
        photometry: &Photometry,
        position: Vec3,
        rotation_deg: f64,
        tilt_deg: f64,
        flux_lm: f64,
        maintenance_factor: f64,
    ) -> Result<Self, CalcError> {
        let distribution = match photometry.tilt() {
            None => photometry.table().clone(),
            Some(tilt) => {
                let factor = tilt.factor(tilt_deg).ok_or(CalcError::TiltOutsideTable { tilt_deg })?;
                photometry.table().scaled(factor).map_err(CalcError::Photometry)?
            }
        };
        Ok(Self { position, rotation_deg, tilt_deg, distribution, flux_lm, maintenance_factor })
    }

    /// The precomputed numbers a calculation needs, or `None` if an input is not finite.
    fn frame(&self) -> Option<Frame> {
        let ok = self.position.is_finite()
            && self.rotation_deg.is_finite()
            && self.tilt_deg.is_finite()
            && self.flux_lm.is_finite()
            && self.maintenance_factor.is_finite();
        let (sin_r, cos_r) = self.rotation_deg.to_radians().sin_cos();
        let (sin_t, cos_t) = self.tilt_deg.to_radians().sin_cos();
        ok.then_some(Frame { sin_r, cos_r, sin_t, cos_t, scale: self.flux_lm / 1000.0 * self.maintenance_factor })
    }
}

struct Frame {
    sin_r: f64,
    cos_r: f64,
    sin_t: f64,
    cos_t: f64,
    /// cd per cd/klm: flux / 1000 × maintenance factor.
    scale: f64,
}

impl Frame {
    /// A world direction in the luminaire's own frame: undo the heading (about z), then the tilt
    /// (about the luminaire's y axis).
    fn to_local(&self, d: Vec3) -> Vec3 {
        // Rz(−rotation)
        let x = d.x * self.cos_r + d.y * self.sin_r;
        let y = -d.x * self.sin_r + d.y * self.cos_r;
        // Tilting by t maps the nadir (0, 0, −1) to (sin t, 0, −cos t); undo it with Ry(t):
        // x' = x cos t + z sin t, z' = −x sin t + z cos t.
        Vec3::new(x * self.cos_t + d.z * self.sin_t, y, -x * self.sin_t + d.z * self.cos_t)
    }
}

/// Direct illuminance (lux) at `point` on a surface facing `normal` (need not be unit length).
///
/// `None` when the result is undefined: the point coincides with the luminaire, the normal is zero,
/// or an input is not finite. Light arriving from behind the surface gives 0 lux.
pub fn direct_illuminance(luminaire: &Luminaire, point: Vec3, normal: Vec3) -> Option<f64> {
    let normal = normal.normalized()?;
    direct(luminaire, &luminaire.frame()?, point, normal, &[])
}

/// Direct illuminance (lux) on a horizontal surface facing up at `point`.
pub fn direct_horizontal_illuminance(luminaire: &Luminaire, point: Vec3) -> Option<f64> {
    direct_illuminance(luminaire, point, Vec3::UP)
}

fn direct(luminaire: &Luminaire, frame: &Frame, point: Vec3, unit_normal: Vec3, walls: &[Wall]) -> Option<f64> {
    if !point.is_finite() {
        return None;
    }
    let to_point = point - luminaire.position;
    let d2 = to_point.length_squared();
    let dir = to_point.normalized()?;
    // The light travels along `dir`; it lights the surface when it arrives against the normal.
    let cos_theta = -dir.dot(unit_normal);
    if cos_theta <= 0.0 || walls.iter().any(|w| w.blocks(luminaire.position, point)) {
        return Some(0.0);
    }
    let local = frame.to_local(dir);
    let gamma = (-local.z).clamp(-1.0, 1.0).acos().to_degrees();
    let c = if local.x == 0.0 && local.y == 0.0 { 0.0 } else { wrap_degrees(local.y.atan2(local.x).to_degrees()) };
    let candela = luminaire.distribution.intensity(c, gamma) * frame.scale;
    let e = candela * cos_theta / d2;
    e.is_finite().then_some(e)
}

/// Illuminance (lux) at every point of `grid`, summed over `luminaires`; `None` at a point where
/// any luminaire's contribution is undefined.
pub fn illuminance(luminaires: &[Luminaire], grid: &Grid) -> Vec<Option<f64>> {
    illuminance_with_walls(luminaires, &[], grid)
}

/// [`illuminance`] with `walls` blocking the light that would cross them.
pub fn illuminance_with_walls(luminaires: &[Luminaire], walls: &[Wall], grid: &Grid) -> Vec<Option<f64>> {
    let mut sums: Vec<Option<f64>> = vec![Some(0.0); grid.points().len()];
    let Some(normal) = grid.normal().normalized() else { return vec![None; grid.points().len()] };
    for luminaire in luminaires {
        let Some(frame) = luminaire.frame() else { return vec![None; grid.points().len()] };
        for (sum, point) in sums.iter_mut().zip(grid.points()) {
            if let Some(total) = sum {
                *sum = direct(luminaire, &frame, *point, normal, walls).map(|e| *total + e);
            }
        }
    }
    sums
}

#[cfg(test)]
mod tests;
