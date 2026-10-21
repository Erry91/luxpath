//! A luminaire's photometry as read from a file: the full table, the flux it is relative to, tilt
//! data, and where it came from.

use crate::{IesEdition, IntensityTable, PhotometryError, ReadError, Symmetry, read_eulumdat, read_ies};

/// The file format a photometry was read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Ies(IesEdition),
    Eulumdat,
}

/// How a luminaire's light varies with its tilt from the measured position (IES `TILT=INCLUDE`,
/// for lamps whose output depends on their burning position).
#[derive(Debug, Clone, PartialEq)]
pub struct Tilt {
    /// The file's lamp-to-luminaire geometry code: 1, 2 or 3.
    pub lamp_geometry: u8,
    angles_deg: Vec<f64>,
    factors: Vec<f64>,
}

impl Tilt {
    /// Checks that the angles increase strictly and the factors are finite and non-negative.
    pub fn new(lamp_geometry: u8, angles_deg: Vec<f64>, factors: Vec<f64>) -> Option<Self> {
        let ok = !angles_deg.is_empty()
            && angles_deg.len() == factors.len()
            && angles_deg.iter().all(|a| a.is_finite())
            && angles_deg.windows(2).all(|w| w.first() < w.get(1))
            && factors.iter().all(|f| f.is_finite() && *f >= 0.0);
        ok.then_some(Self { lamp_geometry, angles_deg, factors })
    }

    pub fn angles_deg(&self) -> &[f64] {
        &self.angles_deg
    }

    pub fn factors(&self) -> &[f64] {
        &self.factors
    }

    /// The multiplier at `tilt_deg`, linearly interpolated; `None` outside the measured angles
    /// (the file does not say, so a check must not guess).
    pub fn factor(&self, tilt_deg: f64) -> Option<f64> {
        let (first, last) = (self.angles_deg.first()?, self.angles_deg.last()?);
        if !tilt_deg.is_finite() || tilt_deg < *first || tilt_deg > *last {
            return None;
        }
        let hi = self.angles_deg.iter().position(|a| *a >= tilt_deg)?;
        let (a1, f1) = (self.angles_deg.get(hi)?, self.factors.get(hi)?);
        if hi == 0 || *a1 == tilt_deg {
            return Some(*f1);
        }
        let (a0, f0) = (self.angles_deg.get(hi - 1)?, self.factors.get(hi - 1)?);
        Some(f0 + (f1 - f0) * (tilt_deg - a0) / (a1 - a0))
    }
}

/// How much light a luminaire gives in emergency mode, relative to the flux in its photometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmergencyOutput {
    /// The emergency ballast lumen factor: emergency flux over rated flux, in `(0, 1]`.
    BallastLumenFactor(f64),
    /// The emergency flux in lumens, measured like [`Photometry::flux_lm`] (the lamps' flux for
    /// relative photometry, the luminaire's for absolute), at most the rated flux. A luminaire flux
    /// given for relative photometry counts the light output ratio twice: it underestimates, never
    /// overestimates.
    FluxLm(f64),
}

/// A luminaire's photometry.
#[derive(Debug, Clone, PartialEq)]
pub struct Photometry {
    table: IntensityTable,
    flux_lm: f64,
    absolute: bool,
    symmetry: Symmetry,
    format: Format,
    tilt: Option<Tilt>,
    /// The manufacturer, as the file names it (may be empty).
    pub manufacturer: String,
    /// The luminaire's name or description (may be empty).
    pub name: String,
    /// The catalogue or article number (may be empty).
    pub catalogue: String,
}

impl Photometry {
    /// `table` in cd/klm of `flux_lm` (finite and positive).
    pub(crate) fn new(
        table: IntensityTable,
        flux_lm: f64,
        absolute: bool,
        symmetry: Symmetry,
        format: Format,
    ) -> Result<Self, PhotometryError> {
        if !flux_lm.is_finite() || flux_lm <= 0.0 {
            return Err(PhotometryError::BadFlux { reason: "the rated flux is not a positive number" });
        }
        Ok(Self {
            table,
            flux_lm,
            absolute,
            symmetry,
            format,
            tilt: None,
            manufacturer: String::new(),
            name: String::new(),
            catalogue: String::new(),
        })
    }

    pub(crate) fn with_tilt(mut self, tilt: Option<Tilt>) -> Self {
        self.tilt = tilt;
        self
    }

    /// The full distribution, in cd per 1000 lm of [`Self::flux_lm`].
    pub fn table(&self) -> &IntensityTable {
        &self.table
    }

    /// The rated flux in mains operation, in lumens: the lamps' flux, or for absolute photometry
    /// (LED luminaires measured as a whole) the luminaire's own, integrated from the table.
    pub fn flux_lm(&self) -> f64 {
        self.flux_lm
    }

    /// Whether the file gave absolute intensities (IES lumens per lamp = −1).
    pub fn is_absolute(&self) -> bool {
        self.absolute
    }

    /// The symmetry the file declared (the table is already expanded).
    pub fn symmetry(&self) -> Symmetry {
        self.symmetry
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn tilt(&self) -> Option<&Tilt> {
        self.tilt.as_ref()
    }

    /// The luminaire's flux over the rated flux: 1 for absolute photometry.
    pub fn light_output_ratio(&self) -> f64 {
        self.table.integrated_flux() / 1000.0
    }

    /// Intensity in candela towards `(c, gamma)` (degrees) at a flux of `flux_lm`.
    pub fn candela(&self, c: f64, gamma: f64, flux_lm: f64) -> f64 {
        self.table.intensity(c, gamma) * flux_lm / 1000.0
    }

    /// The flux in emergency mode, in lumens. Data the file does not hold (emergency output is
    /// declared separately by the luminaire's maker), so the caller supplies it; out-of-range
    /// values are errors, never clamped.
    pub fn emergency_flux_lm(&self, output: EmergencyOutput) -> Result<f64, PhotometryError> {
        match output {
            EmergencyOutput::BallastLumenFactor(f) if f.is_finite() && f > 0.0 && f <= 1.0 => Ok(self.flux_lm * f),
            EmergencyOutput::BallastLumenFactor(_) => Err(PhotometryError::BadFlux {
                reason: "the emergency ballast lumen factor must be above 0 and at most 1",
            }),
            EmergencyOutput::FluxLm(lm) if lm.is_finite() && lm > 0.0 && lm <= self.flux_lm => Ok(lm),
            EmergencyOutput::FluxLm(_) => Err(PhotometryError::BadFlux {
                reason: "the emergency flux must be above 0 lm and at most the rated flux",
            }),
        }
    }
}

/// Reads an IES LM-63 or EULUMDAT file, telling them apart by content: IES files have a `TILT=`
/// line, EULUMDAT files never do.
pub fn read(bytes: &[u8]) -> Result<Photometry, ReadError> {
    if bytes.split(|b| *b == b'\n' || *b == b'\r').any(crate::text::is_tilt_line) {
        read_ies(bytes)
    } else {
        read_eulumdat(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn photometry(flux: f64) -> Photometry {
        Photometry::new(IntensityTable::isotropic(1000.0).unwrap(), flux, false, Symmetry::Rotational, Format::Eulumdat)
            .unwrap()
    }

    #[test]
    fn emergency_flux_from_a_factor_or_lumens_within_range() {
        let p = photometry(2000.0);
        assert_eq!(p.emergency_flux_lm(EmergencyOutput::BallastLumenFactor(0.25)), Ok(500.0));
        assert_eq!(p.emergency_flux_lm(EmergencyOutput::BallastLumenFactor(1.0)), Ok(2000.0));
        assert_eq!(p.emergency_flux_lm(EmergencyOutput::FluxLm(2000.0)), Ok(2000.0));
        for bad in [0.0, -0.1, 1.0001, f64::NAN] {
            assert!(p.emergency_flux_lm(EmergencyOutput::BallastLumenFactor(bad)).is_err(), "{bad}");
        }
        for bad in [0.0, 2000.5, f64::INFINITY] {
            assert!(p.emergency_flux_lm(EmergencyOutput::FluxLm(bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn candela_scales_with_flux() {
        // 1000 cd/klm at 500 lm is 500 cd.
        assert_eq!(photometry(2000.0).candela(10.0, 20.0, 500.0), 500.0);
    }

    #[test]
    fn rated_flux_must_be_positive() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(
                Photometry::new(IntensityTable::isotropic(1.0).unwrap(), bad, false, Symmetry::None, Format::Eulumdat)
                    .is_err()
            );
        }
    }

    #[test]
    fn tilt_factors_interpolate_inside_and_say_nothing_outside() {
        let t = Tilt::new(1, vec![0.0, 30.0, 90.0], vec![1.0, 0.9, 0.6]).unwrap();
        assert_eq!(t.factor(0.0), Some(1.0));
        assert!((t.factor(15.0).unwrap() - 0.95).abs() < 1e-12);
        assert!((t.factor(60.0).unwrap() - 0.75).abs() < 1e-12);
        assert_eq!(t.factor(90.0), Some(0.6));
        assert_eq!(t.factor(-1.0), None);
        assert_eq!(t.factor(90.1), None);
        assert_eq!(t.factor(f64::NAN), None);
        assert!(Tilt::new(1, vec![0.0, 0.0], vec![1.0, 1.0]).is_none());
        assert!(Tilt::new(1, vec![0.0], vec![-1.0]).is_none());
        assert!(Tilt::new(1, vec![], vec![]).is_none());
    }
}
