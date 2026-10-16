//! `luxpath`: the command-line tool.
//!
//! M0 has one demonstration command, `point`, which computes direct illuminance under an
//! isotropic source. `luxpath check scene.toml` is M3–M4.

use std::process::ExitCode;

use luxpath_calc::{Luminaire, direct_horizontal_illuminance};
use luxpath_geom::Vec3;
use luxpath_photometry::IntensityTable;

const USAGE: &str = "\
luxpath — open emergency-lighting calculations

USAGE:
    luxpath point <mounting height m> <horizontal offset m> [flux lm]
        Direct illuminance on the floor from an isotropic 1000 cd/klm source (demonstration).
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
        Some(other) => {
            eprintln!("error: unknown command `{other}`\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
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
        distribution,
        flux_lm: flux,
        maintenance_factor: 1.0,
    };
    direct_horizontal_illuminance(&luminaire, Vec3::new(offset, 0.0, 0.0))
        .ok_or_else(|| "the result is undefined".into())
}
