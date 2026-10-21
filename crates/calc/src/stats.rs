//! Statistics over a grid's illuminances.

/// Minimum, mean and maximum illuminance (lux) over a grid's points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub min_lx: f64,
    pub mean_lx: f64,
    pub max_lx: f64,
    pub points: usize,
}

impl Stats {
    /// The statistics of `values`; `None` if there are none or any is undefined (a check over them
    /// cannot be verified).
    pub fn of(values: &[Option<f64>]) -> Option<Self> {
        let mut stats = Self { min_lx: f64::INFINITY, mean_lx: 0.0, max_lx: f64::NEG_INFINITY, points: 0 };
        let mut sum = 0.0;
        for v in values {
            let v = (*v).filter(|v| v.is_finite())?;
            stats.min_lx = stats.min_lx.min(v);
            stats.max_lx = stats.max_lx.max(v);
            sum += v;
            stats.points += 1;
        }
        if stats.points == 0 {
            return None;
        }
        stats.mean_lx = sum / stats.points as f64;
        Some(stats)
    }

    /// Max over min (EN 1838 limits it, e.g. 40:1); `None` when the minimum is 0 (unbounded).
    pub fn max_to_min(&self) -> Option<f64> {
        (self.min_lx > 0.0).then(|| self.max_lx / self.min_lx)
    }

    /// Min over mean (U₀); `None` when the mean is 0.
    pub fn min_to_mean(&self) -> Option<f64> {
        (self.mean_lx > 0.0).then(|| self.min_lx / self.mean_lx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statistics_by_hand() {
        // 1, 2, 3, 6: min 1, mean 3, max 6; max/min 6, min/mean 1/3.
        let s = Stats::of(&[Some(1.0), Some(2.0), Some(3.0), Some(6.0)]).unwrap();
        assert_eq!((s.min_lx, s.mean_lx, s.max_lx, s.points), (1.0, 3.0, 6.0, 4));
        assert_eq!(s.max_to_min(), Some(6.0));
        assert_eq!(s.min_to_mean(), Some(1.0 / 3.0));
    }

    #[test]
    fn undefined_or_empty_values_give_no_statistics() {
        assert_eq!(Stats::of(&[]), None);
        assert_eq!(Stats::of(&[Some(1.0), None]), None);
        assert_eq!(Stats::of(&[Some(f64::NAN)]), None);
        let dark = Stats::of(&[Some(0.0), Some(2.0)]).unwrap();
        assert_eq!(dark.max_to_min(), None);
        assert_eq!(Stats::of(&[Some(0.0)]).unwrap().min_to_mean(), None);
    }
}
