//! The IES LM-63 reader (editions 1986, 1991, 1995, 2002 and 2019), type C photometry.
//!
//! A file is: an edition line (none in 1986), `[KEYWORD] value` lines (`[MORE]` continues the
//! previous one), a `TILT=` line and, for `TILT=INCLUDE`, the tilt table; then numbers separated by
//! whitespace or commas, across lines as the writer pleased: lamps, lumens per lamp (−1 for
//! absolute photometry), candela multiplier, the vertical and horizontal angle counts, photometric
//! type, units, the luminous opening, ballast factor, a factor whose meaning changed between
//! editions, input watts, the vertical angles, the horizontal angles, and the candela values, one
//! horizontal angle after another.

use crate::model::{Format, Photometry, Tilt};
use crate::symmetry::{ExpandError, Symmetry};
use crate::text::{ReadError, ReadErrorKind, Tokens, decode, error, invalid, is_tilt_line, table_error};
use crate::{IntensityTable, MAX_TABLE_VALUES};

/// The LM-63 edition a file declares in its first line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IesEdition {
    /// No edition line.
    Lm63_1986,
    /// `IESNA91`.
    Lm63_1991,
    /// `IESNA:LM-63-1995`.
    Lm63_1995,
    /// `IESNA:LM-63-2002`.
    Lm63_2002,
    /// `IES:LM-63-2019`.
    Lm63_2019,
}

/// Tilt tables longer than this are refused (real ones have a few dozen angles).
const MAX_TILT_ANGLES: usize = 10_000;

/// Angles closer than this are equal (files write angles with a few decimals).
const SAME_ANGLE_DEG: f64 = 1e-6;

fn edition(first_line: &str) -> Result<Option<IesEdition>, ReadErrorKind> {
    // Writers vary the spacing (`IESNA: LM-63-2002`).
    let line: String = first_line.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_ascii_uppercase();
    let known = [
        ("IESNA:LM-63-1995", IesEdition::Lm63_1995),
        ("IESNA:LM-63-2002", IesEdition::Lm63_2002),
        ("IES:LM-63-2019", IesEdition::Lm63_2019),
        ("IESNA:LM-63-2019", IesEdition::Lm63_2019),
        ("IESNA:LM-63-1991", IesEdition::Lm63_1991),
        ("IESNA91", IesEdition::Lm63_1991),
    ];
    if let Some((_, e)) = known.iter().find(|(tag, _)| line.starts_with(tag)) {
        return Ok(Some(*e));
    }
    if line.starts_with("IESNA") || line.starts_with("IES:") {
        return Err(ReadErrorKind::Unsupported("this LM-63 edition"));
    }
    Ok(None)
}

/// Reads an IES LM-63 file.
pub fn read_ies(bytes: &[u8]) -> Result<Photometry, ReadError> {
    let text = decode(bytes)?;
    let mut edition_found = IesEdition::Lm63_1986;
    let mut keywords: Vec<(String, String)> = Vec::new();
    let mut tilt = None;
    let mut offset = 0;
    let mut last_line = 1;
    for (i, raw) in text.split_inclusive('\n').enumerate() {
        let n = i + 1;
        last_line = n;
        offset += raw.len();
        let line = raw.trim();
        if n == 1
            && let Some(e) = edition(line).map_err(|kind| error(n, kind))?
        {
            edition_found = e;
            continue;
        }
        if is_tilt_line(line.as_bytes()) {
            let value = line.split_once('=').map_or("", |(_, v)| v);
            tilt = Some((n, value.trim().to_ascii_uppercase()));
            break;
        }
        if let Some((key, value)) = line.strip_prefix('[').and_then(|kw| kw.split_once(']')) {
            let value = value.trim().to_string();
            match keywords.last_mut() {
                Some((_, previous)) if key == "MORE" => {
                    previous.push('\n');
                    previous.push_str(&value);
                }
                _ => keywords.push((key.trim().to_ascii_uppercase(), value)),
            }
        }
        // Other lines before TILT are free text (1986 files).
    }
    let (tilt_line, tilt_value) = tilt.ok_or_else(|| error(last_line, ReadErrorKind::Missing("TILT= line")))?;
    let mut tokens = Tokens::new(text.get(offset..).unwrap_or(""), tilt_line + 1);

    let tilt = match tilt_value.as_str() {
        "NONE" => None,
        "INCLUDE" => Some(read_tilt(&mut tokens)?),
        _ => return Err(error(tilt_line, ReadErrorKind::Unsupported("TILT data in a separate file"))),
    };

    let (line, lamps) = tokens.next_count("number of lamps")?;
    if lamps == 0 {
        return Err(invalid(line, "number of lamps", "is zero"));
    }
    let (lumens_line, lumens) = tokens.next_number("lumens per lamp")?;
    let (line, multiplier) = tokens.next_number("candela multiplier")?;
    if multiplier <= 0.0 {
        return Err(invalid(line, "candela multiplier", "is not positive"));
    }
    let (line, n_vertical) = tokens.next_count("number of vertical angles")?;
    if n_vertical == 0 {
        return Err(invalid(line, "number of vertical angles", "is zero"));
    }
    let (line, n_horizontal) = tokens.next_count("number of horizontal angles")?;
    if n_horizontal == 0 {
        return Err(invalid(line, "number of horizontal angles", "is zero"));
    }
    let n_values = n_vertical.checked_mul(n_horizontal).filter(|n| *n <= MAX_TABLE_VALUES);
    let Some(n_values) = n_values else {
        return Err(error(
            line,
            ReadErrorKind::Table(crate::PhotometryError::TooLarge { values: n_vertical.saturating_mul(n_horizontal) }),
        ));
    };
    let (line, photometric_type) = tokens.next_count("photometric type")?;
    match photometric_type {
        1 => {}
        2 | 3 => return Err(error(line, ReadErrorKind::Unsupported("type A or B photometry (only type C)"))),
        _ => return Err(invalid(line, "photometric type", "is not 1, 2 or 3")),
    }
    let (line, units) = tokens.next_count("units type")?;
    if !(1..=2).contains(&units) {
        return Err(invalid(line, "units type", "is not 1 (feet) or 2 (metres)"));
    }
    for what in ["luminous opening width", "luminous opening length", "luminous opening height"] {
        tokens.next_number(what)?;
    }
    let (line, ballast_factor) = tokens.next_number("ballast factor")?;
    if ballast_factor <= 0.0 {
        return Err(invalid(line, "ballast factor", "is not positive"));
    }
    // 1986–1995: the ballast-lamp photometric factor, which multiplies the candela values. 2002
    // made it "future use" and 2019 the file generation type: no longer a factor.
    let (line, second_factor) = tokens.next_number("ballast-lamp photometric factor")?;
    let photometric_factor = if edition_found <= IesEdition::Lm63_1995 {
        if second_factor <= 0.0 {
            return Err(invalid(line, "ballast-lamp photometric factor", "is not positive"));
        }
        second_factor
    } else {
        1.0
    };
    tokens.next_number("input watts")?;

    let vertical_line = tokens.next_line();
    let vertical = read_angles(&mut tokens, n_vertical, "vertical angles")?;
    let horizontal_line = tokens.next_line();
    let horizontal = read_angles(&mut tokens, n_horizontal, "horizontal angles")?;
    let values_line = tokens.next_line();
    let mut values = Vec::with_capacity(n_values);
    for _ in 0..n_values {
        let (line, v) = tokens.next_number("candela values")?;
        if v < 0.0 {
            return Err(invalid(line, "candela value", "is negative"));
        }
        values.push(v * multiplier * ballast_factor * photometric_factor);
    }
    if let Some((line, _)) = tokens.next() {
        return Err(invalid(line, "file", "has data after the candela values (are the angle counts right?)"));
    }

    let symmetry = symmetry(&horizontal).ok_or_else(|| {
        invalid(
            horizontal_line,
            "horizontal angles",
            "do not start and end where LM-63 allows (0–0, 0–90, 0–180, 90–270, 0–360)",
        )
    })?;
    let table = symmetry.expand(&horizontal, vertical, &values).map_err(|e| match e {
        ExpandError::Table(t) => table_error(t, horizontal_line, vertical_line, values_line),
        other => invalid(horizontal_line, "horizontal angles", other.reason()),
    })?;
    let absolute = lumens == -1.0;
    let flux_lm = if absolute {
        Some(table.integrated_flux())
            .filter(|flux| flux.is_finite() && *flux > 0.0)
            .ok_or_else(|| invalid(values_line, "candela values", "emit no light"))?
    } else if lumens > 0.0 {
        lamps as f64 * lumens
    } else {
        return Err(invalid(lumens_line, "lumens per lamp", "is neither positive nor −1 (absolute photometry)"));
    };
    let table: IntensityTable =
        table.scaled(1000.0 / flux_lm).map_err(|e| error(values_line, ReadErrorKind::Table(e)))?;
    let mut photometry = Photometry::new(table, flux_lm, absolute, symmetry, Format::Ies(edition_found))
        .map_err(|e| error(lumens_line, ReadErrorKind::Table(e)))?
        .with_tilt(tilt);
    let keyword = |k: &str| keywords.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone()).unwrap_or_default();
    photometry.manufacturer = keyword("MANUFAC");
    photometry.name = keyword("LUMINAIRE");
    photometry.catalogue = keyword("LUMCAT");
    Ok(photometry)
}

fn read_tilt(tokens: &mut Tokens<'_>) -> Result<Tilt, ReadError> {
    let (line, geometry) = tokens.next_count("lamp-to-luminaire geometry")?;
    let geometry = u8::try_from(geometry)
        .ok()
        .filter(|g| (1..=3).contains(g))
        .ok_or_else(|| invalid(line, "lamp-to-luminaire geometry", "is not 1, 2 or 3"))?;
    let (line, n) = tokens.next_count("number of tilt angles")?;
    if n == 0 || n > MAX_TILT_ANGLES {
        return Err(invalid(line, "number of tilt angles", "is zero or too large"));
    }
    let angles = read_angles(tokens, n, "tilt angles")?;
    let mut factors = Vec::with_capacity(n);
    for _ in 0..n {
        factors.push(tokens.next_number("tilt multiplying factors")?.1);
    }
    Tilt::new(geometry, angles, factors)
        .ok_or_else(|| invalid(line, "tilt table", "has angles that do not increase, or negative factors"))
}

fn read_angles(tokens: &mut Tokens<'_>, n: usize, what: &'static str) -> Result<Vec<f64>, ReadError> {
    let mut angles = Vec::with_capacity(n);
    for _ in 0..n {
        angles.push(tokens.next_number(what)?.1);
    }
    Ok(angles)
}

/// The symmetry LM-63 implies by the first and last horizontal angle.
fn symmetry(horizontal: &[f64]) -> Option<Symmetry> {
    let (first, last) = (*horizontal.first()?, *horizontal.last()?);
    let near = |a: f64, b: f64| (a - b).abs() < SAME_ANGLE_DEG;
    if horizontal.len() == 1 {
        return Some(Symmetry::Rotational);
    }
    if near(first, 0.0) {
        if near(last, 90.0) {
            return Some(Symmetry::Quadrants);
        }
        if near(last, 180.0) {
            return Some(Symmetry::PlaneC0C180);
        }
        if last > 180.0 && last <= 360.0 + SAME_ANGLE_DEG {
            return Some(Symmetry::None);
        }
    }
    if near(first, 90.0) && near(last, 270.0) {
        return Some(Symmetry::PlaneC90C270);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small 2002 file: two lamps of 500 lm, quadrant symmetric, values doubled by the multiplier.
    const QUADRANT: &str = "IESNA:LM-63-2002\n[TEST] T1\n[MANUFAC] Example\n[LUMCAT] EX-1\n[LUMINAIRE] Downlight\n[MORE] white\nTILT=NONE\n2 500 2.0 3 2 1 2 0.1 0.2 0.0\n1.0 1.0 10\n0 45 90\n0 90\n100 50 0\n80 40, 0\n";

    #[test]
    fn a_quadrant_file_reads_into_a_full_table_in_cd_per_klm() {
        let p = read_ies(QUADRANT.as_bytes()).unwrap();
        assert_eq!(p.format(), Format::Ies(IesEdition::Lm63_2002));
        assert_eq!(p.symmetry(), Symmetry::Quadrants);
        // Flux 2 × 500 = 1000 lm; 100 cd × multiplier 2 = 200 cd, that is 200 cd/klm.
        assert_eq!(p.flux_lm(), 1000.0);
        assert_eq!(p.table().intensity(0.0, 0.0), 200.0);
        assert_eq!(p.table().intensity(90.0, 45.0), 80.0);
        assert_eq!(p.table().intensity(270.0, 45.0), 80.0);
        assert_eq!(p.table().c_angles(), &[0.0, 90.0, 180.0, 270.0]);
        assert_eq!((p.manufacturer.as_str(), p.catalogue.as_str()), ("Example", "EX-1"));
        assert_eq!(p.name, "Downlight\nwhite");
    }

    #[test]
    fn absolute_photometry_takes_its_flux_from_the_table() {
        // An isotropic 100 cd source: Φ = 4π × 100 lm, and 100 cd is 1000/(4π) cd/klm of it.
        let text = "IESNA:LM-63-2002\nTILT=NONE\n1 -1 1 2 1 1 2 0 0 0\n1 1 0\n0 180\n0\n100 100\n";
        let p = read_ies(text.as_bytes()).unwrap();
        assert!(p.is_absolute());
        let flux = 400.0 * std::f64::consts::PI;
        assert!((p.flux_lm() - flux).abs() < 1e-9);
        assert!((p.candela(0.0, 30.0, p.flux_lm()) - 100.0).abs() < 1e-9);
        assert!((p.light_output_ratio() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn older_editions_apply_the_ballast_lamp_factor_and_newer_ones_do_not() {
        let body = "TILT=NONE\n1 1000 1 2 1 1 2 0 0 0\n0.5 0.8 10\n0 90\n0\n100 100\n";
        let old = read_ies(format!("IESNA:LM-63-1995\n{body}").as_bytes()).unwrap();
        let new = read_ies(format!("IES:LM-63-2019\n{body}").as_bytes()).unwrap();
        // 100 cd × ballast 0.5 × photometric factor 0.8 = 40 cd/klm at 1000 lm; 2019: 100 × 0.5 = 50.
        assert!((old.table().intensity(0.0, 0.0) - 40.0).abs() < 1e-12);
        assert!((new.table().intensity(0.0, 0.0) - 50.0).abs() < 1e-12);
        let spaced = read_ies(format!("IESNA: LM-63-2002\n{body}").as_bytes()).unwrap();
        assert_eq!(spaced.format(), Format::Ies(IesEdition::Lm63_2002));
        let ancient = read_ies(format!("Some free text\n{body}").as_bytes()).unwrap();
        assert_eq!(ancient.format(), Format::Ies(IesEdition::Lm63_1986));
    }

    #[test]
    fn tilt_tables_are_read() {
        let text =
            "IESNA:LM-63-2002\nTILT=INCLUDE\n1\n3\n0 45 90\n1 0.9 0.8\n1 1000 1 2 1 1 2 0 0 0\n1 1 0\n0 90\n0\n100 0\n";
        let p = read_ies(text.as_bytes()).unwrap();
        let tilt = p.tilt().unwrap();
        assert_eq!(
            (tilt.lamp_geometry, tilt.angles_deg(), tilt.factors()),
            (1, &[0.0, 45.0, 90.0][..], &[1.0, 0.9, 0.8][..])
        );
    }

    #[test]
    fn errors_name_their_line() {
        let cases: &[(&str, usize, &str)] = &[
            ("IESNA:LM-63-2002\n[TEST] x\n", 2, "TILT"),
            ("IESNA:LM-63-2002\nTILT=other.tlt\n", 2, "separate file"),
            ("IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 2 1 2 2 0 0 0\n", 3, "type A or B"),
            (
                "IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 2 1 1 2 0 0 0\n1 1 0\n0 90\n0\n100\n",
                7,
                "ends before the candela values",
            ),
            ("IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 2 1 1 2 0 0 0\n1 1 0\n0 90\n0\n100 -5\n", 7, "negative"),
            ("IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 2 1 1 2 0 0 0\n1 1 0\n0 90\n0\n100 x\n", 7, "number"),
            ("IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 2 1 1 2 0 0 0\n1 1 0\n0 90\n0\n100 5\n7\n", 8, "after the candela"),
            (
                "IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 2 2 1 2 0 0 0\n1 1 0\n0 90\n0 45\n1 1\n1 1\n",
                6,
                "horizontal angles",
            ),
            ("IESNA:LM-63-2002\nTILT=NONE\n1 0 1 2 1 1 2 0 0 0\n1 1 0\n0 90\n0\n1 1\n", 3, "lumens per lamp"),
            ("IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 2 1 1 2 0 0 0\n1 1 0\n0 200\n0\n1 1\n", 5, "gamma angles"),
            ("IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 99999 99999 1 2 0 0 0\n", 3, "limit"),
            ("IESNA:LM-63-2025\nTILT=NONE\n", 1, "edition"),
        ];
        for (text, line, words) in cases {
            let e = read_ies(text.as_bytes()).unwrap_err();
            assert_eq!(e.line, *line, "{e}");
            assert!(e.to_string().contains(words), "{e} should mention {words}");
        }
    }
}
