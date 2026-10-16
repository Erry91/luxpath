//! Layering check (docs/architecture.md §Layers).
//!
//! Each workspace crate has a layer. A crate may depend only on crates in strictly lower layers,
//! and core crates (below the apps) may not depend on UI or windowing crates, so the engine stays
//! usable headlessly and in the browser.

use std::process::Command;

use serde::Deserialize;

/// (crate, layer). A new crate must be added here, or the check fails.
const LAYERS: &[(&str, u8)] = &[
    ("luxpath-geom", 0),
    ("luxpath-photometry", 1),
    ("luxpath-scene", 2),
    ("luxpath-calc", 3),
    ("luxpath-rules", 4),
    ("luxpath-report", 5),
    ("luxpath-cli", 9),
    ("xtask", 9),
];

/// The first layer that may use UI crates.
const APP_LAYER: u8 = 9;

/// External crates that core crates must never depend on.
const UI_CRATES: &[&str] = &["egui", "eframe", "winit", "rfd", "wgpu"];

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    workspace_members: Vec<String>,
}

#[derive(Deserialize)]
struct Package {
    id: String,
    name: String,
    dependencies: Vec<Dependency>,
}

#[derive(Deserialize)]
struct Dependency {
    name: String,
    kind: Option<String>,
}

pub fn run() -> Result<(), String> {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| format!("could not run cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err("cargo metadata failed".into());
    }
    let meta: Metadata = serde_json::from_slice(&output.stdout).map_err(|e| format!("bad cargo metadata: {e}"))?;
    let layer_of = |name: &str| LAYERS.iter().find(|(n, _)| *n == name).map(|(_, l)| *l);

    let mut problems = Vec::new();
    for package in meta.packages.iter().filter(|p| meta.workspace_members.contains(&p.id)) {
        let Some(layer) = layer_of(&package.name) else {
            problems
                .push(format!("{} has no layer: add it to xtask/src/layers.rs and docs/architecture.md", package.name));
            continue;
        };
        // Dev-dependencies (tests) may reach anywhere.
        for dep in package.dependencies.iter().filter(|d| d.kind.as_deref() != Some("dev")) {
            if let Some(dep_layer) = layer_of(&dep.name)
                && dep_layer >= layer
            {
                problems.push(format!("{} (L{layer}) depends on {} (L{dep_layer})", package.name, dep.name));
            }
            if layer < APP_LAYER && UI_CRATES.contains(&dep.name.as_str()) {
                problems.push(format!("{} is a core crate but depends on the UI crate {}", package.name, dep.name));
            }
        }
    }
    if problems.is_empty() {
        eprintln!("xtask layers: ok");
        Ok(())
    } else {
        Err(format!("layering violations:\n  {}", problems.join("\n  ")))
    }
}
