//! Validates `parity/features.toml` and prints progress.
//!
//! Rules: every feature has a known area, tier, milestone and status; ids are unique; a
//! `shipped` feature cites at least one test in `evidence` that exists as a file path prefix;
//! `partial` and `shipped` features say in `notes` what is still missing.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
struct Checklist {
    schema: Schema,
    feature: Vec<Feature>,
}

#[derive(Deserialize)]
struct Schema {
    statuses: Vec<String>,
    tiers: Vec<String>,
    areas: BTreeMap<String, String>,
    milestones: Vec<String>,
}

#[derive(Deserialize)]
struct Feature {
    id: String,
    title: String,
    area: String,
    tier: String,
    milestone: String,
    status: String,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    notes: String,
}

pub fn run() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().ok_or("xtask has no parent directory")?;
    let text = std::fs::read_to_string(root.join("parity/features.toml"))
        .map_err(|e| format!("cannot read parity/features.toml: {e}"))?;
    let list: Checklist = toml::from_str(&text).map_err(|e| format!("parity/features.toml: {e}"))?;

    let mut problems = Vec::new();
    let mut seen = HashSet::new();
    for f in &list.feature {
        let at = &f.id;
        if !seen.insert(f.id.as_str()) {
            problems.push(format!("{at}: duplicate id"));
        }
        if f.title.trim().is_empty() {
            problems.push(format!("{at}: empty title"));
        }
        if !list.schema.areas.contains_key(&f.area) {
            problems.push(format!("{at}: unknown area `{}`", f.area));
        }
        if !f.id.starts_with(&format!("{}.", f.area)) {
            problems.push(format!("{at}: id must start with its area `{}.`", f.area));
        }
        if !list.schema.tiers.contains(&f.tier) {
            problems.push(format!("{at}: unknown tier `{}`", f.tier));
        }
        if !list.schema.milestones.contains(&f.milestone) {
            problems.push(format!("{at}: unknown milestone `{}`", f.milestone));
        }
        if !list.schema.statuses.contains(&f.status) {
            problems.push(format!("{at}: unknown status `{}`", f.status));
        }
        if f.status == "shipped" && f.evidence.is_empty() {
            problems.push(format!("{at}: shipped needs at least one test in `evidence`"));
        }
        if matches!(f.status.as_str(), "partial" | "shipped") && f.notes.trim().is_empty() {
            problems.push(format!("{at}: say in `notes` what is still missing"));
        }
        for e in &f.evidence {
            let file = e.split("::").next().unwrap_or(e);
            if !root.join(file).is_file() {
                problems.push(format!("{at}: evidence file `{file}` does not exist"));
            }
        }
    }
    if !problems.is_empty() {
        return Err(format!("parity/features.toml:\n  {}", problems.join("\n  ")));
    }

    let mut by_milestone: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
    for f in &list.feature {
        let entry = by_milestone.entry(f.milestone.as_str()).or_default();
        entry.0 += 1;
        match f.status.as_str() {
            "shipped" => entry.1 += 1,
            "partial" => entry.2 += 1,
            _ => {}
        }
    }
    eprintln!("xtask parity: {} features", list.feature.len());
    for milestone in &list.schema.milestones {
        if let Some((total, shipped, partial)) = by_milestone.get(milestone.as_str()) {
            eprintln!("  {milestone:<4} {shipped:>3} shipped  {partial:>3} partial  of {total:>3}");
        }
    }
    Ok(())
}
