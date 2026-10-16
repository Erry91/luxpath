//! `cargo xtask <command>`: repository automation.
//!
//! - `ci`: every gate CI runs (fmt, clippy -D warnings, tests, wasm check, layers, parity).
//! - `layers`: checks that crates only depend on crates in lower layers.
//! - `parity`: validates `parity/features.toml` and prints progress by milestone.

mod layers;
mod parity;

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("ci") => ci(),
        Some("layers") => layers::run(),
        Some("parity") => parity::run(),
        _ => Err("usage: cargo xtask <ci | layers | parity>".to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("xtask: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Crates that must build for `wasm32-unknown-unknown` (the browser build). Apps and xtask don't.
pub const WASM_CRATES: &[&str] =
    &["luxpath-geom", "luxpath-photometry", "luxpath-scene", "luxpath-calc", "luxpath-rules", "luxpath-report"];

fn ci() -> Result<(), String> {
    cargo(&["fmt", "--all", "--check"])?;
    cargo(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    cargo(&["test", "--workspace"])?;
    if wasm_target_installed() {
        let mut args = vec!["check", "--target", "wasm32-unknown-unknown"];
        for krate in WASM_CRATES {
            args.extend(["-p", krate]);
        }
        cargo(&args)?;
    } else {
        eprintln!("xtask: skipping the wasm check (rustup target add wasm32-unknown-unknown to run it)");
    }
    layers::run()?;
    parity::run()
}

fn cargo(args: &[&str]) -> Result<(), String> {
    eprintln!("$ cargo {}", args.join(" "));
    let status = Command::new(env!("CARGO")).args(args).status().map_err(|e| format!("could not run cargo: {e}"))?;
    if status.success() { Ok(()) } else { Err(format!("`cargo {}` failed", args.join(" "))) }
}

fn wasm_target_installed() -> bool {
    Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).lines().any(|l| l.trim() == "wasm32-unknown-unknown"))
        .unwrap_or(false)
}
