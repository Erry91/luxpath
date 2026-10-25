//! Standards checks: turns a scene's calculated illuminances into findings with evidence.
//!
//! [`evaluate`] computes, for every escape route, area and point of emphasis in a scene, the
//! illuminance statistics in emergency mode, and lists the assumptions behind them. Anything that
//! cannot be computed honestly (a luminaire without a declared emergency output, a tilt outside its
//! file's table, an undefined point) makes the affected results "cannot verify", never a number.
//!
//! First standard (M4): **EN 1838:2024**, emergency lighting for buildings. Every requirement is
//! a row in `docs/standards-parity.md` with its clause number, and [`en1838`] lists those on routes,
//! areas and points: each element gets a [`Finding`] per requirement, citing the clause. Values come
//! from the standard itself, paraphrased: never copy the standard's text or tables into this
//! repository (`AGENTS.md` §1). Until the values are verified (gap 4) and M4 checks them, every
//! finding and the overall verdict are [`Verdict::CannotVerify`].
//!
//! Later: EN 12464-1 (interior workplaces) and EN 13201 (road lighting), as separate modules.
//!
//! Layer L4: depends on `luxpath-scene` and `luxpath-calc`.

use luxpath_calc::{Grid, Luminaire, Stats, illuminance_with_walls};
use luxpath_geom::Vec3;
use luxpath_photometry::Photometry;
use luxpath_scene::{AreaKind, PointKind, RouteShape, Scene};

pub mod en1838;

/// What one element's illuminance came to.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Computed(Stats),
    /// Why no number can be trusted.
    CannotVerify(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    Route,
    Area(AreaKind),
    Point(PointKind),
}

/// One route, area or point and its results.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementResult {
    pub id: String,
    pub name: String,
    pub kind: ElementKind,
    /// On the horizontal plane (the floor of routes and areas; at the point for points).
    pub horizontal: Outcome,
    /// On the surface a point faces (safety equipment), when it has a facing direction.
    pub vertical: Option<Outcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    CannotVerify,
}

/// One requirement applied to one element.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub requirement: &'static en1838::Requirement,
    /// The route, area or point it is about.
    pub element: String,
    pub verdict: Verdict,
    /// Why the verdict is what it is.
    pub reason: String,
}

/// Why no requirement can pass or fail yet.
const UNVERIFIED_VALUE: &str = "its value is not yet verified against the standard's text (gap 4)";

/// A scene's evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct Evaluation {
    pub results: Vec<ElementResult>,
    /// Each requirement on each element it applies to, in the order of [`Evaluation::results`].
    pub findings: Vec<Finding>,
    /// What the numbers assume, for the report.
    pub assumptions: Vec<String>,
    /// Implausible photometric files and the like: shown, not hidden.
    pub warnings: Vec<String>,
    pub verdict: Verdict,
    /// Why the verdict is what it is.
    pub verdict_reason: String,
}

/// Evaluates `scene` with `photometry[i]` read from `scene.photometry[i].file`.
///
/// Errors only when the inputs do not belong together (a different number of photometries).
pub fn evaluate(scene: &Scene, photometry: &[Photometry]) -> Result<Evaluation, String> {
    if photometry.len() != scene.photometry.len() {
        return Err(format!(
            "the scene names {} photometric files but {} were given",
            scene.photometry.len(),
            photometry.len()
        ));
    }
    let calc = scene.calculation;
    let mut assumptions = vec![
        "direct light only: no reflections from walls, floor or ceiling".to_string(),
        "luminaires are points".to_string(),
        if scene.walls.is_empty() {
            "nothing blocks the light: the scene has no walls".to_string()
        } else {
            let n = scene.walls.len();
            format!(
                "walls block the light ({n} straight piece{}); furniture, doors and other obstacles do not",
                if n == 1 { "" } else { "s" }
            )
        },
        format!("maintenance factor {}", scene.maintenance_factor),
        format!(
            "grids every {} m; {} points across each route's full width; open areas less a {} m border, other areas over their whole outline",
            calc.spacing_m, calc.route_points_across, calc.open_area_border_m
        ),
    ];
    let mut warnings = Vec::new();
    let mut problems = Vec::new();
    let mut emergency_flux: Vec<Option<f64>> = Vec::new();
    for (reference, p) in scene.photometry.iter().zip(photometry) {
        for w in p.warnings() {
            warnings.push(format!("{} ({}): {w}", reference.id, reference.file));
        }
        let flux = match reference.emergency {
            None => {
                problems.push(format!("photometry `{}` declares no emergency output", reference.id));
                None
            }
            Some(output) => match p.emergency_flux_lm(output) {
                Ok(lm) => {
                    assumptions.push(format!(
                        "`{}`: {:.0} lm in emergency mode (rated {:.0} lm)",
                        reference.id,
                        lm,
                        p.flux_lm()
                    ));
                    Some(lm)
                }
                Err(e) => {
                    problems.push(format!("photometry `{}`: {e}", reference.id));
                    None
                }
            },
        };
        emergency_flux.push(flux);
    }

    let mut luminaires = Vec::new();
    for placed in &scene.luminaires {
        let (Some(p), Some(Some(flux))) = (photometry.get(placed.photometry), emergency_flux.get(placed.photometry))
        else {
            continue;
        };
        match Luminaire::from_photometry(
            p,
            placed.position,
            placed.rotation_deg,
            placed.tilt_deg,
            *flux,
            scene.maintenance_factor,
        ) {
            Ok(l) => luminaires.push(l),
            Err(e) => problems.push(format!("luminaire `{}`: {e}", placed.id)),
        }
    }
    let unverifiable = (!problems.is_empty()).then(|| problems.join("; "));

    let outcome = |grid: Result<Grid, luxpath_calc::GridError>| -> Outcome {
        if let Some(reason) = &unverifiable {
            return Outcome::CannotVerify(reason.clone());
        }
        match grid {
            Err(e) => Outcome::CannotVerify(e.to_string()),
            Ok(grid) => match Stats::of(&illuminance_with_walls(&luminaires, &scene.walls, &grid)) {
                Some(stats) => Outcome::Computed(stats),
                None => Outcome::CannotVerify("a grid point is at a luminaire's centre".into()),
            },
        }
    };

    let mut results = Vec::new();
    for r in &scene.routes {
        results.push(ElementResult {
            id: r.id.clone(),
            name: r.name.clone(),
            kind: ElementKind::Route,
            horizontal: outcome(match &r.shape {
                RouteShape::Centre { path, width_m } => {
                    Grid::route(path, *width_m, r.floor_m, calc.spacing_m, calc.route_points_across)
                }
                RouteShape::Outline(outline) => Grid::area(outline, r.floor_m, calc.spacing_m, 0.0),
            }),
            vertical: None,
        });
    }
    for a in &scene.areas {
        // Only open areas are known to leave a border out; the others are computed over their whole outline.
        let border = if a.kind == AreaKind::Open { calc.open_area_border_m } else { 0.0 };
        results.push(ElementResult {
            id: a.id.clone(),
            name: a.name.clone(),
            kind: ElementKind::Area(a.kind),
            horizontal: outcome(Grid::area(&a.outline, a.floor_m, calc.spacing_m, border)),
            vertical: None,
        });
    }
    for p in &scene.points {
        results.push(ElementResult {
            id: p.id.clone(),
            name: p.name.clone(),
            kind: ElementKind::Point(p.kind),
            horizontal: outcome(Grid::new(vec![p.position], Vec3::UP)),
            vertical: p.facing.map(|facing| outcome(Grid::new(vec![p.position], facing))),
        });
    }

    let mut findings = Vec::new();
    for r in &results {
        // An element whose numbers cannot be trusted says why; otherwise the value is what is missing.
        let unverifiable = [Some(&r.horizontal), r.vertical.as_ref()].into_iter().flatten().find_map(|o| match o {
            Outcome::CannotVerify(why) => Some(why.clone()),
            Outcome::Computed(_) => None,
        });
        for requirement in en1838::requirements_for(r.kind) {
            findings.push(Finding {
                requirement,
                element: r.id.clone(),
                verdict: Verdict::CannotVerify,
                reason: unverifiable.clone().unwrap_or_else(|| UNVERIFIED_VALUE.into()),
            });
        }
    }

    Ok(Evaluation {
        results,
        verdict_reason: format!(
            "no {} requirement is checked yet: {} finding{} cannot be verified (milestone M4, gap 4); the values above are calculations only",
            en1838::EDITION,
            findings.len(),
            if findings.len() == 1 { "" } else { "s" }
        ),
        findings,
        assumptions,
        warnings,
        verdict: Verdict::CannotVerify,
    })
}

#[cfg(test)]
mod tests;
