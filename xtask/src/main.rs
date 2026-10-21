//! `cargo xtask <command>`: repository automation.
//!
//! - `ci`: every gate CI runs (fmt, clippy -D warnings, tests, the release-mode performance budget,
//!   wasm check, layers, parity).
//! - `layers`: checks that crates only depend on crates in lower layers.
//! - `parity`: validates `parity/features.toml` and prints progress by milestone.
//! - `fuzz [seconds]`: fuzzes the photometric file readers (default 600 s, M1's exit criterion).

mod layers;
mod parity;

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("ci") => ci(),
        Some("layers") => layers::run(),
        Some("parity") => parity::run(),
        Some("fuzz") => fuzz(args.get(1).map_or("600", String::as_str)),
        _ => Err("usage: cargo xtask <ci | layers | parity | fuzz [seconds]>".to_string()),
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
pub const WASM_CRATES: &[&str] = &[
    "luxpath-geom",
    "luxpath-photometry",
    "luxpath-scene",
    "luxpath-calc",
    "luxpath-rules",
    "luxpath-report",
    "luxpath-ifc",
];

fn ci() -> Result<(), String> {
    cargo(&["fmt", "--all", "--check"])?;
    cargo(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    cargo(&["test", "--workspace"])?;
    // M2's budget: timing is meaningful only in release.
    cargo(&["test", "--release", "-p", "luxpath-calc", "--test", "performance", "--", "--ignored"])?;
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

fn fuzz(seconds: &str) -> Result<(), String> {
    let seconds: u64 = seconds.parse().map_err(|_| format!("`{seconds}` is not a number of seconds"))?;
    eprintln!("$ LUXPATH_FUZZ_SECONDS={seconds} cargo test --release -p luxpath-photometry --test fuzz -- --ignored");
    let status = Command::new(env!("CARGO"))
        .args(["test", "--release", "-p", "luxpath-photometry", "--test", "fuzz", "--", "--ignored", "--nocapture"])
        .env("LUXPATH_FUZZ_SECONDS", seconds.to_string())
        .status()
        .map_err(|e| format!("could not run cargo: {e}"))?;
    if status.success() { Ok(()) } else { Err("the fuzzer found a panic".into()) }
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
