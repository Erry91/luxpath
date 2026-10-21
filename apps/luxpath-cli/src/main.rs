//! `luxpath`: the command-line tool.
//!
//! Commands: `point` (direct illuminance under an isotropic source, a demonstration),
//! `photometry` (what a photometric file says, as luxpath reads it) and `inspect-ifc` (what an IFC
//! model contains for an emergency-lighting check). `luxpath check scene.toml` is M3–M4.

use std::process::ExitCode;

use luxpath_calc::{Luminaire, direct_horizontal_illuminance};
use luxpath_geom::Vec3;
use luxpath_photometry::{Format, IntensityTable, MAX_FILE_BYTES, Photometry, Symmetry};

const USAGE: &str = "\
luxpath — open emergency-lighting calculations

USAGE:
    luxpath point <mounting height m> <horizontal offset m> [flux lm]
        Direct illuminance on the floor from an isotropic 1000 cd/klm source (demonstration).
    luxpath photometry <file.ies|file.ldt>
        What a photometric file says, as luxpath reads it: flux, symmetry, peak, light output ratio.
    luxpath inspect-ifc <model.ifc> [--json]
        What an IFC model contains for an emergency-lighting check, and what is missing.
    luxpath --version
    luxpath --help

Results are calculations, not certificates: a qualified designer remains responsible.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("-h" | "--help" | "help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some("-V" | "--version") => {
            println!("luxpath {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("point") => match point(args.get(1..).unwrap_or_default()) {
            Ok(lux) => {
                println!("{lux:.2} lx");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("error: {message}");
                ExitCode::from(2)
            }
        },
        Some("photometry") => match photometry(args.get(1..).unwrap_or_default()) {
            Ok(text) => {
                print!("{text}");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("error: {message}");
                ExitCode::from(2)
            }
        },
        Some("inspect-ifc") => match inspect_ifc(args.get(1..).unwrap_or_default()) {
            Ok(text) => {
                print!("{text}");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("error: {message}");
                ExitCode::from(2)
            }
        },
        Some(other) => {
            eprintln!("error: unknown command `{other}`\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// Larger files are refused rather than read whole into memory.
const MAX_IFC_BYTES: u64 = 2 << 30;

fn inspect_ifc(args: &[String]) -> Result<String, String> {
    let json = args.iter().any(|a| a == "--json");
    let path = args.iter().find(|a| !a.starts_with("--")).ok_or("missing the IFC file path")?;
    let size = std::fs::metadata(path).map_err(|e| format!("{path}: {e}"))?.len();
    if size > MAX_IFC_BYTES {
        return Err(format!("{path}: {size} bytes is more than the {MAX_IFC_BYTES}-byte limit"));
    }
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let inspection = luxpath_ifc::inspect(&bytes).map_err(|e| format!("{path}: {e}"))?;
    if json {
        serde_json::to_string_pretty(&inspection).map(|s| s + "\n").map_err(|e| e.to_string())
    } else {
        Ok(inspection.to_text())
    }
}

fn photometry(args: &[String]) -> Result<String, String> {
    let path = args.first().ok_or("missing the photometric file path")?;
    let size = std::fs::metadata(path).map_err(|e| format!("{path}: {e}"))?.len();
    if size > MAX_FILE_BYTES as u64 {
        return Err(format!("{path}: {size} bytes is more than the {MAX_FILE_BYTES}-byte limit"));
    }
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let p = luxpath_photometry::read(&bytes).map_err(|e| format!("{path}: {e}"))?;
    Ok(describe(&p))
}

fn describe(p: &Photometry) -> String {
    let format = match p.format() {
        Format::Ies(edition) => format!("IES {edition:?}").replace("Lm63_", "LM-63-"),
        Format::Eulumdat => "EULUMDAT".to_string(),
    };
    let symmetry = match p.symmetry() {
        Symmetry::None => "none (every C plane stored)",
        Symmetry::Rotational => "rotational (one C plane)",
        Symmetry::PlaneC0C180 => "about the C0–C180 plane",
        Symmetry::PlaneC90C270 => "about the C90–C270 plane",
        Symmetry::Quadrants => "quadrants (C0–C90 stored)",
    };
    let t = p.table();
    let n_gamma = t.gamma_angles().len().max(1);
    let (peak_at, peak) =
        t.values().iter().enumerate().fold((0, 0.0), |best, (i, v)| if *v > best.1 { (i, *v) } else { best });
    let peak_c = t.c_angles().get(peak_at / n_gamma).copied().unwrap_or(0.0);
    let peak_gamma = t.gamma_angles().get(peak_at % n_gamma).copied().unwrap_or(0.0);
    let gamma = |i: Option<&f64>| i.copied().unwrap_or(0.0);
    let mut out = String::new();
    let mut line = |label: &str, value: String| {
        if !value.is_empty() {
            out += &format!("{label:<18} {value}\n");
        }
    };
    line("format", format);
    line("manufacturer", p.manufacturer.clone());
    line("luminaire", p.name.replace('\n', " "));
    line("catalogue", p.catalogue.clone());
    let kind = if p.is_absolute() { "luminaire flux, absolute photometry" } else { "lamp flux" };
    line("rated flux", format!("{:.1} lm ({kind})", p.flux_lm()));
    line("symmetry", symmetry.to_string());
    line(
        "table",
        format!(
            "{} C planes × {} γ angles (γ {}°–{}°)",
            t.c_angles().len(),
            t.gamma_angles().len(),
            gamma(t.gamma_angles().first()),
            gamma(t.gamma_angles().last())
        ),
    );
    line("peak", format!("{peak:.1} cd/klm at C{peak_c}° γ{peak_gamma}°"));
    line("light output", format!("{:.1} % of the rated flux", p.light_output_ratio() * 100.0));
    if let Some(tilt) = p.tilt() {
        line("tilt", format!("{} angles, lamp geometry {}", tilt.angles_deg().len(), tilt.lamp_geometry));
    }
    for warning in p.warnings() {
        line("warning", warning.to_string());
    }
    out
}

fn point(args: &[String]) -> Result<f64, String> {
    let number = |i: usize, name: &str| -> Result<Option<f64>, String> {
        args.get(i)
            .map(|s| {
                s.parse::<f64>().ok().filter(|v| v.is_finite()).ok_or(format!("{name} must be a number, got `{s}`"))
            })
            .transpose()
    };
    let height = number(0, "mounting height")?.ok_or("missing mounting height")?;
    let offset = number(1, "horizontal offset")?.ok_or("missing horizontal offset")?;
    let flux = number(2, "flux")?.unwrap_or(1000.0);
    if height <= 0.0 {
        return Err("mounting height must be above the floor".into());
    }
    let distribution = IntensityTable::isotropic(1000.0).map_err(|e| e.to_string())?;
    let luminaire = Luminaire {
        position: Vec3::new(0.0, 0.0, height),
        rotation_deg: 0.0,
        tilt_deg: 0.0,
        distribution,
        flux_lm: flux,
        maintenance_factor: 1.0,
    };
    direct_horizontal_illuminance(&luminaire, Vec3::new(offset, 0.0, 0.0))
        .ok_or_else(|| "the result is undefined".into())
}
