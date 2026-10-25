//! End to end: `luxpath check` on the example scenes and on broken ones.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::Command;

fn luxpath(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_luxpath")).args(args).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into(),
        String::from_utf8_lossy(&out.stderr).into(),
    )
}

fn examples() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// A scratch directory with the examples' photometry next to the given scene.
fn scratch(name: &str, scene: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("luxpath-check-{name}-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("photometry")).unwrap();
    for file in ["bulkhead.ldt", "corridor.ies"] {
        std::fs::copy(examples().join("photometry").join(file), dir.join("photometry").join(file)).unwrap();
    }
    std::fs::write(dir.join("scene.toml"), scene).unwrap();
    dir
}

#[test]
fn every_example_is_calculated_and_cannot_be_verified_until_m4() {
    for (scene, ids) in [
        ("corridor.toml", &["R1", "E1", "D1", "F1"][..]),
        ("open-office.toml", &["R1", "A1", "E1", "C1"][..]),
        ("plant-room.toml", &["H1", "S1", "E1"][..]),
    ] {
        let path = examples().join(scene);
        let (code, out, err) = luxpath(&["check", path.to_str().unwrap()]);
        assert_eq!(code, 3, "{scene}: {err}");
        for id in ids {
            assert!(out.contains(&format!(" {id} ")), "{scene} should report {id}:\n{out}");
        }
        assert!(!out.contains("CANNOT VERIFY:") || out.contains("Verdict: CANNOT VERIFY"), "{out}");
        assert!(out.contains("min ") && out.contains("not a certificate"), "{out}");
        assert!(out.contains("walls block the light"), "the assumptions are stated:\n{out}");
        // Every element's requirements are listed with their clause, none passing while values are unverified.
        assert!(out.contains("Requirements (EN 1838:2024):"), "{out}");
        for id in ids {
            let cited = out.lines().any(|l| l.split_whitespace().nth(1) == Some(*id) && l.contains("CANNOT VERIFY"));
            assert!(cited, "{scene}: no requirement cited for {id}:\n{out}");
        }
    }
}

#[test]
fn a_luminaire_without_emergency_output_is_reported_not_guessed() {
    let scene = std::fs::read_to_string(examples().join("open-office.toml")).unwrap();
    // Drop the line whatever the checkout's line ends (Windows checks text out with CRLF).
    let without: String =
        scene.lines().filter(|l| l.trim() != "emergency = { flux_lm = 200 }").map(|l| format!("{l}\n")).collect();
    assert_ne!(without.lines().count(), scene.lines().count(), "the example declares the emergency output");
    let dir = scratch("no-emergency", &without);
    let (code, out, _) = luxpath(&["check", dir.join("scene.toml").to_str().unwrap()]);
    assert_eq!(code, 3);
    assert!(out.contains("CANNOT VERIFY: photometry `bulkhead` declares no emergency output"), "{out}");
    assert!(!out.contains(" lx,"), "no numbers when the flux is unknown:\n{out}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn broken_inputs_exit_with_2_and_say_where() {
    let scene = std::fs::read_to_string(examples().join("corridor.toml")).unwrap();
    let dir = scratch("broken", &scene.replace("width_m = 2.0", "width_m = \"wide\""));
    let (code, _, err) = luxpath(&["check", dir.join("scene.toml").to_str().unwrap()]);
    assert_eq!(code, 2);
    // The error points at the line of `width_m` in the example, wherever it is.
    let line = scene.lines().position(|l| l.starts_with("width_m = 2.0")).unwrap() + 1;
    assert!(err.contains(&format!("scene.toml: line {line}, column 11")), "{err}");

    std::fs::write(dir.join("scene.toml"), scene.replace("photometry/bulkhead.ldt", "photometry/missing.ldt")).unwrap();
    let (code, _, err) = luxpath(&["check", dir.join("scene.toml").to_str().unwrap()]);
    assert_eq!(code, 2);
    assert!(err.contains("missing.ldt"), "{err}");

    std::fs::write(dir.join("photometry/bulkhead.ldt"), "not a photometric file\n").unwrap();
    std::fs::write(dir.join("scene.toml"), &scene).unwrap();
    let (code, _, err) = luxpath(&["check", dir.join("scene.toml").to_str().unwrap()]);
    assert_eq!(code, 2);
    assert!(err.contains("bulkhead.ldt: line"), "{err}");
    std::fs::remove_dir_all(&dir).ok();
}
