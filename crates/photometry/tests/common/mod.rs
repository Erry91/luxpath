//! Synthetic photometric files: one distribution written as IES LM-63 and as EULUMDAT, for every
//! symmetry. Written by our code, so they are free to commit and to generate (AGENTS.md §1.3).

#![allow(dead_code, clippy::indexing_slicing, clippy::unwrap_used)]

use luxpath_photometry::Symmetry;

/// A distribution as a file stores it.
#[derive(Debug, Clone)]
pub struct Stored {
    pub symmetry: Symmetry,
    /// Distance between C planes, in degrees; divides 90.
    pub c_step: f64,
    pub gamma: Vec<f64>,
    /// cd/klm, one stored plane after another (planes as [`Stored::stored_c`]).
    pub values: Vec<f64>,
    /// Lamp flux in lumens.
    pub flux_lm: f64,
}

/// Every C angle from 0 below 360 at `step`.
pub fn all_c(step: f64) -> Vec<f64> {
    let n = (360.0 / step).round() as usize;
    (0..n).map(|i| i as f64 * step).collect()
}

impl Stored {
    /// The C angles of the stored planes.
    pub fn stored_c(&self) -> Vec<f64> {
        let (first, last) = match self.symmetry {
            Symmetry::Rotational => return vec![0.0],
            Symmetry::None => (0.0, 360.0 - self.c_step),
            s => s.stored_range(),
        };
        all_c(self.c_step).into_iter().chain([360.0]).filter(|c| *c >= first - 1e-9 && *c <= last + 1e-9).collect()
    }

    /// A distribution whose value at plane `p`, γ index `g` comes from `f(p, g)`.
    pub fn new(symmetry: Symmetry, c_step: f64, gamma: Vec<f64>, f: impl Fn(usize, usize) -> f64) -> Self {
        let mut s = Self { symmetry, c_step, gamma, values: vec![], flux_lm: 2000.0 };
        let planes = s.stored_c().len();
        s.values = (0..planes).flat_map(|p| (0..s.gamma.len()).map(move |g| (p, g))).map(|(p, g)| f(p, g)).collect();
        s
    }

    fn plane(&self, p: usize) -> &[f64] {
        let n = self.gamma.len();
        &self.values[p * n..(p + 1) * n]
    }

    /// The IES LM-63-2002 file: one lamp of `flux_lm`, absolute candela values.
    pub fn ies(&self) -> String {
        let mut c = self.stored_c();
        let mut planes: Vec<&[f64]> = (0..c.len()).map(|p| self.plane(p)).collect();
        if self.symmetry == Symmetry::None {
            // LM-63 full distributions end with C360, a copy of C0.
            c.push(360.0);
            planes.push(self.plane(0));
        }
        let k = self.flux_lm / 1000.0;
        let mut out = String::from("IESNA:LM-63-2002\n[TEST] synthetic\n[MANUFAC] luxpath tests\nTILT=NONE\n");
        out += &format!("1 {} 1 {} {} 1 2 0.2 0.2 0\n1 1 10\n", self.flux_lm, self.gamma.len(), c.len());
        out += &join(&self.gamma);
        out += &join(&c);
        for p in planes {
            out += &join(&p.iter().map(|v| v * k).collect::<Vec<_>>());
        }
        out
    }

    /// The EULUMDAT file (CRLF line ends, as Windows writers produce).
    pub fn ldt(&self) -> String {
        let isym = match self.symmetry {
            Symmetry::None => 0,
            Symmetry::Rotational => 1,
            Symmetry::PlaneC0C180 => 2,
            Symmetry::PlaneC90C270 => 3,
            Symmetry::Quadrants => 4,
        };
        let c = if self.symmetry == Symmetry::Rotational { vec![0.0] } else { all_c(self.c_step) };
        let mut lines: Vec<String> = vec!["luxpath tests".into(), "1".into(), isym.to_string(), c.len().to_string()];
        lines.extend([self.c_step.to_string(), self.gamma.len().to_string(), "0".into(), "synthetic".into()]);
        lines.extend(["Synthetic luminaire", "SYN-1", "synthetic.ldt", "2026-10-21"].map(String::from));
        lines.extend(
            ["600", "600", "50", "580", "580", "0", "0", "0", "0", "100", "80", "1", "0", "1"].map(String::from),
        );
        lines.extend(["1".into(), "LED".into(), self.flux_lm.to_string(), "4000".into(), "80".into(), "20".into()]);
        lines.extend((0..10).map(|_| "0.5".to_string()));
        lines.extend(c.iter().chain(&self.gamma).chain(&self.values).map(|v| v.to_string()));
        lines.join("\r\n") + "\r\n"
    }
}

fn join(values: &[f64]) -> String {
    // Ten values a line, as writers wrap them.
    values.chunks(10).map(|chunk| chunk.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ") + "\n").collect()
}

/// One distribution per symmetry, uneven enough that a mirrored plane in the wrong place shows.
pub fn every_symmetry() -> Vec<Stored> {
    let gamma: Vec<f64> = (0..=36).map(|i| f64::from(i) * 5.0).collect();
    let value = |p: usize, g: usize| (100 * p + 3 * g) as f64 + 0.5;
    [Symmetry::None, Symmetry::Rotational, Symmetry::PlaneC0C180, Symmetry::PlaneC90C270, Symmetry::Quadrants]
        .into_iter()
        .map(|s| Stored::new(s, 15.0, gamma.clone(), value))
        .collect()
}
