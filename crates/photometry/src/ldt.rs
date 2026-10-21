//! The EULUMDAT (.ldt) reader.
//!
//! EULUMDAT is line-based: one item per line, in a fixed order. Lines 1–25 describe the file and the
//! luminaire (the symmetry indicator on line 3, the C-plane and γ counts on lines 4 and 6, the
//! intensity conversion factor on line 24); line 26 gives the number of lamp sets, each set six
//! lines (lamps, type, total flux, colour temperature, colour rendering, wattage); ten direct ratios
//! follow, then every C angle, every γ angle, and the intensities (cd/klm) of the planes the symmetry
//! indicator says are stored, one value per line. Decimal commas are accepted.

use crate::model::{Format, Photometry};
use crate::symmetry::{ExpandError, Symmetry};
use crate::text::{ReadError, ReadErrorKind, count, decode, error, invalid, number, table_error};
use crate::{MAX_TABLE_VALUES, PhotometryError};

/// The file's lines, read one item at a time.
struct Fields<'a> {
    lines: std::iter::Enumerate<std::str::Lines<'a>>,
    line: usize,
}

impl<'a> Fields<'a> {
    fn next(&mut self, what: &'static str) -> Result<(usize, &'a str), ReadError> {
        let (i, text) = self.lines.next().ok_or_else(|| error(self.line, ReadErrorKind::Missing(what)))?;
        self.line = i + 1;
        Ok((self.line, text.trim_matches(|c: char| c.is_whitespace() || c == '\u{1A}')))
    }

    fn text(&mut self, what: &'static str) -> Result<String, ReadError> {
        Ok(self.next(what)?.1.to_string())
    }

    fn number(&mut self, what: &'static str) -> Result<(usize, f64), ReadError> {
        let (line, text) = self.next(what)?;
        Ok((line, decimal(line, what, text)?))
    }

    fn count(&mut self, what: &'static str) -> Result<(usize, usize), ReadError> {
        let (line, text) = self.next(what)?;
        let text = if text.contains(',') && !text.contains('.') { text.replace(',', ".") } else { text.to_string() };
        Ok((line, count(line, what, &text)?))
    }
}

/// A number, with a decimal comma if the writer used one.
fn decimal(line: usize, what: &'static str, text: &str) -> Result<f64, ReadError> {
    if text.contains(',') && !text.contains('.') {
        number(line, what, &text.replace(',', "."))
    } else {
        number(line, what, text)
    }
}

/// Reads a EULUMDAT file.
pub fn read_eulumdat(bytes: &[u8]) -> Result<Photometry, ReadError> {
    let text = decode(bytes)?;
    let mut fields = Fields { lines: text.lines().enumerate(), line: 1 };
    let company = fields.text("company identification")?;
    fields.next("type indicator")?;
    let (line, indicator) = fields.count("symmetry indicator")?;
    let symmetry = match indicator {
        0 => Symmetry::None,
        1 => Symmetry::Rotational,
        2 => Symmetry::PlaneC0C180,
        3 => Symmetry::PlaneC90C270,
        4 => Symmetry::Quadrants,
        _ => return Err(invalid(line, "symmetry indicator", "is not 0 to 4")),
    };
    let (c_line, n_c) = fields.count("number of C planes")?;
    if n_c == 0 {
        return Err(invalid(c_line, "number of C planes", "is zero"));
    }
    fields.next("distance between C planes")?;
    let (line, n_gamma) = fields.count("number of intensities per C plane")?;
    if n_gamma == 0 {
        return Err(invalid(line, "number of intensities per C plane", "is zero"));
    }
    fields.next("distance between γ angles")?;
    fields.next("measurement report number")?;
    let name = fields.text("luminaire name")?;
    let catalogue = fields.text("luminaire number")?;
    for what in ["file name", "date and user"] {
        fields.next(what)?;
    }
    for what in [
        "luminaire length",
        "luminaire width",
        "luminaire height",
        "luminous area length",
        "luminous area width",
        "luminous area height at C0",
        "luminous area height at C90",
        "luminous area height at C180",
        "luminous area height at C270",
        "downward flux fraction",
    ] {
        fields.number(what)?;
    }
    let (_, declared_lor) = fields.number("light output ratio")?;
    let (line, conversion) = fields.number("intensity conversion factor")?;
    if conversion <= 0.0 {
        return Err(invalid(line, "intensity conversion factor", "is not positive"));
    }
    fields.number("tilt during measurement")?;
    let (line, sets) = fields.count("number of lamp sets")?;
    match sets {
        0 => return Err(invalid(line, "number of lamp sets", "is zero")),
        1 => {}
        _ => return Err(error(line, ReadErrorKind::Unsupported("more than one set of lamps"))),
    }
    // A negative number of lamps marks absolute photometry (a convention of some writers): the
    // flux is then the luminaire's.
    let (_, lamps) = fields.number("number of lamps")?;
    fields.next("type of lamps")?;
    let (flux_line, flux_lm) = fields.number("total luminous flux of lamps")?;
    if flux_lm <= 0.0 {
        return Err(invalid(flux_line, "total luminous flux of lamps", "is not positive"));
    }
    for what in ["colour temperature", "colour rendering", "wattage"] {
        fields.next(what)?;
    }
    for _ in 0..10 {
        fields.number("direct ratios")?;
    }

    let c_first_line = fields.line + 1;
    let mut c_angles = Vec::with_capacity(n_c.min(MAX_TABLE_VALUES));
    for _ in 0..n_c {
        c_angles.push(fields.number("C angles")?.1);
    }
    let gamma_first_line = fields.line + 1;
    let mut gamma_angles = Vec::with_capacity(n_gamma.min(MAX_TABLE_VALUES));
    for _ in 0..n_gamma {
        gamma_angles.push(fields.number("γ angles")?.1);
    }

    // The stored planes: all, the first, the first half (C0–C180), the half from C90 (C90–C270),
    // or the first quarter (C0–C90).
    let stored = match symmetry {
        Symmetry::None => Some(c_angles.as_slice()),
        Symmetry::Rotational => Some(&[0.0][..]),
        Symmetry::PlaneC0C180 => c_angles.get(..=n_c / 2),
        Symmetry::PlaneC90C270 => {
            let start = c_angles.iter().position(|c| (c - 90.0).abs() < 1e-6);
            start.and_then(|s| c_angles.get(s..=s.saturating_add(n_c / 2)))
        }
        Symmetry::Quadrants => c_angles.get(..=n_c / 4),
    }
    .filter(|s| !s.is_empty())
    .ok_or_else(|| invalid(c_first_line, "C angles", "do not match the symmetry indicator"))?;
    let n_values = stored.len().checked_mul(n_gamma).filter(|n| *n <= MAX_TABLE_VALUES).ok_or_else(|| {
        error(c_line, ReadErrorKind::Table(PhotometryError::TooLarge { values: stored.len().saturating_mul(n_gamma) }))
    })?;
    let values_line = fields.line + 1;
    let mut values = Vec::with_capacity(n_values);
    for _ in 0..n_values {
        let (line, v) = fields.number("intensities")?;
        if v < 0.0 {
            return Err(invalid(line, "intensity", "is negative"));
        }
        values.push(v * conversion);
    }
    while let Ok((line, rest)) = fields.next("") {
        if !rest.is_empty() {
            return Err(invalid(line, "file", "has data after the intensities (are the counts right?)"));
        }
    }

    let table = symmetry.expand(stored, gamma_angles, &values).map_err(|e| match e {
        ExpandError::Table(t) => table_error(t, c_first_line, gamma_first_line, values_line),
        other => invalid(c_first_line, "C angles", other.reason()),
    })?;
    let mut photometry = Photometry::new(table, flux_lm, lamps < 0.0, symmetry, Format::Eulumdat)
        .map_err(|e| error(flux_line, ReadErrorKind::Table(e)))?
        // A ratio of 0 is how some writers leave it out.
        .with_declared_lor((declared_lor > 0.0).then_some(declared_lor / 100.0));
    photometry.manufacturer = company;
    photometry.name = name;
    photometry.catalogue = catalogue;
    Ok(photometry)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A EULUMDAT file with symmetry indicator `isym`, `c` angles (all of them), `g` angles and
    /// the stored `values`, one lamp set of 2000 lm.
    fn ldt(isym: u8, c: &[f64], g: &[f64], values: &[f64]) -> String {
        let mut lines: Vec<String> = vec!["Example Lighting".into(), "1".into(), isym.to_string(), c.len().to_string()];
        lines.extend(["0".into(), g.len().to_string(), "0".into(), "R1".into(), "Panel".into(), "EX-2".into()]);
        lines.extend(["f.ldt".into(), "2026-10-21".into()]);
        lines.extend(
            ["600", "600", "50", "580", "580", "0", "0", "0", "0", "100", "80", "1", "0", "1"].map(String::from),
        );
        lines.extend(["1", "LED", "2000", "4000", "80", "20"].map(String::from));
        lines.extend((0..10).map(|_| "0.5".to_string()));
        lines.extend(c.iter().chain(g).chain(values).map(|v| v.to_string()));
        lines.join("\r\n") + "\r\n"
    }

    #[test]
    fn a_c0_c180_file_reads_into_a_full_table() {
        let c = [0.0, 90.0, 180.0, 270.0];
        let text = ldt(2, &c, &[0.0, 90.0], &[300.0, 100.0, 200.0, 50.0, 100.0, 0.0]);
        let p = read_eulumdat(text.as_bytes()).unwrap();
        assert_eq!(p.format(), Format::Eulumdat);
        assert_eq!(p.symmetry(), Symmetry::PlaneC0C180);
        assert_eq!(p.flux_lm(), 2000.0);
        assert_eq!(p.table().c_angles(), &c);
        assert_eq!(p.table().intensity(270.0, 0.0), 200.0);
        assert_eq!(p.table().intensity(180.0, 90.0), 0.0);
        assert_eq!(
            (p.manufacturer.as_str(), p.name.as_str(), p.catalogue.as_str()),
            ("Example Lighting", "Panel", "EX-2")
        );
    }

    #[test]
    fn every_symmetry_indicator_selects_its_planes() {
        let c: Vec<f64> = (0..8).map(|i| f64::from(i) * 45.0).collect();
        let g = [0.0, 90.0];
        // Stored planes per indicator: 8, 1, 5, 5, 3.
        for (isym, planes) in [(0, 8), (1, 1), (2, 5), (3, 5), (4, 3)] {
            let values: Vec<f64> = (0..planes * 2).map(|i| f64::from(i as u8)).collect();
            let p = read_eulumdat(ldt(isym, &c, &g, &values).as_bytes()).unwrap();
            let expected_planes = if isym == 1 { 1 } else { 8 };
            assert_eq!(p.table().c_angles().len(), expected_planes, "isym {isym}");
        }
    }

    #[test]
    fn decimal_commas_and_latin1_are_accepted() {
        let text = ldt(1, &[0.0], &[0.0, 90.0], &[100.0, 50.0])
            .replace("\r\n80\r\n1\r\n0\r\n1\r\n", "\r\n80\r\n1,5\r\n0\r\n1\r\n");
        let p = read_eulumdat(text.as_bytes()).unwrap();
        // Conversion factor 1,5: 100 × 1.5 = 150 cd/klm.
        assert_eq!(p.table().intensity(0.0, 0.0), 150.0);
        let mut bytes = ldt(1, &[0.0], &[0.0, 90.0], &[100.0, 50.0]).into_bytes();
        bytes.splice(0..7, b"Lumi\xE8re".iter().copied());
        assert!(read_eulumdat(&bytes).unwrap().manufacturer.starts_with("Lumière"));
    }

    #[test]
    fn errors_name_their_line() {
        let good = ldt(1, &[0.0], &[0.0, 90.0], &[100.0, 50.0]);
        let lines: Vec<&str> = good.lines().collect();
        let with = |n: usize, value: &str| {
            let mut l = lines.clone();
            l[n - 1] = value;
            l.join("\n")
        };
        let cases: Vec<(String, usize, &str)> = vec![
            (with(3, "7"), 3, "symmetry indicator"),
            (with(24, "0"), 24, "conversion factor"),
            (with(26, "2"), 26, "more than one set"),
            (with(29, "-5"), 29, "not positive"),
            (with(46, "-1"), 46, "negative"),
            (with(46, "abc"), 46, "number"),
            (lines[..46].join("\n"), 46, "ends before the intensities"),
            (good.clone() + "7\n", 48, "after the intensities"),
            (with(44, "200"), 44, "gamma angles"),
            (String::new(), 1, "company"),
        ];
        for (text, line, words) in cases {
            let e = read_eulumdat(text.as_bytes()).unwrap_err();
            assert_eq!(e.line, line, "{e}");
            assert!(e.to_string().contains(words), "{e} should mention {words}");
        }
    }
}
