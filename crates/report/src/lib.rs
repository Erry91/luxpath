//! Reports: what a designer hands over.
//!
//! Now (M3): [`text`], a plain-text summary for the terminal. Planned (M5): JSON for tools and CI,
//! and an SVG plan (false-colour illuminance, escape routes, luminaires, failing points marked).
//! Every report states the calculation's assumptions and that it is a calculation, not a
//! certificate (`AGENTS.md` §2).
//!
//! Layer L5: depends on `luxpath-scene` and `luxpath-rules`.

use luxpath_rules::{ElementKind, Evaluation, Outcome, Verdict};
use luxpath_scene::{AreaKind, Scene};

const DISCLAIMER: &str =
    "This is a calculation, not a certificate: a competent designer remains responsible for the design.";

fn outcome(o: &Outcome) -> String {
    match o {
        Outcome::Computed(s) => {
            let ratio = s.max_to_min().map_or("unbounded (minimum 0 lx)".to_string(), |r| format!("{r:.1}:1"));
            format!(
                "min {:.2} lx, mean {:.2} lx, max {:.2} lx, max/min {ratio} ({} points)",
                s.min_lx, s.mean_lx, s.max_lx, s.points
            )
        }
        Outcome::CannotVerify(why) => format!("CANNOT VERIFY: {why}"),
    }
}

/// The evaluation as text for the terminal.
pub fn text(scene: &Scene, evaluation: &Evaluation) -> String {
    let mut out = format!("luxpath: {}\n", scene.name);
    out += "Emergency mode, direct light.\n\n";
    for r in &evaluation.results {
        let kind = match r.kind {
            ElementKind::Route => "escape route".to_string(),
            ElementKind::Area(AreaKind::Open) => "open area".to_string(),
            ElementKind::Area(AreaKind::HighRisk) => "high-risk area".to_string(),
            ElementKind::Point(k) => format!("point ({})", k.name()),
        };
        let name = if r.name.is_empty() { String::new() } else { format!(" \"{}\"", r.name) };
        out += &format!("{} {}{name}\n", kind, r.id);
        let plane = if matches!(r.kind, ElementKind::Point(_)) { "horizontal" } else { "floor" };
        out += &format!("  {plane:<10} {}\n", outcome(&r.horizontal));
        if let Some(v) = &r.vertical {
            out += &format!("  {:<10} {}\n", "facing", outcome(v));
        }
    }
    if !evaluation.warnings.is_empty() {
        out += "\nWarnings:\n";
        for w in &evaluation.warnings {
            out += &format!("  - {w}\n");
        }
    }
    out += "\nAssumptions:\n";
    for a in &evaluation.assumptions {
        out += &format!("  - {a}\n");
    }
    let verdict = match evaluation.verdict {
        Verdict::Pass => "PASS",
        Verdict::Fail => "FAIL",
        Verdict::CannotVerify => "CANNOT VERIFY",
    };
    out += &format!("\nVerdict: {verdict}: {}\n{DISCLAIMER}\n", evaluation.verdict_reason);
    out
}
