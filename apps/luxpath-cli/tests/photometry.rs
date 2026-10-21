//! End to end: `luxpath photometry` on the committed synthetic fixtures and on a broken file.

#![allow(clippy::unwrap_used)]

use std::process::Command;

fn luxpath(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_luxpath")).args(args).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into(),
        String::from_utf8_lossy(&out.stderr).into(),
    )
}

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/photometry/tests/fixtures");

#[test]
fn describes_ies_and_eulumdat_files_alike() {
    for (file, format) in [("downlight.ies", "IES LM-63-2002"), ("downlight.ldt", "EULUMDAT")] {
        let (code, out, err) = luxpath(&["photometry", &format!("{FIXTURES}/{file}")]);
        assert_eq!(code, 0, "{err}");
        assert!(out.contains(format), "{out}");
        assert!(out.contains("1000.0 lm (lamp flux)"), "{out}");
        assert!(out.contains("rotational"), "{out}");
        assert!(out.contains("300.0 cd/klm at C0° γ0°"), "{out}");
        assert!(!out.contains("warning"), "{out}");
    }
}

#[test]
fn implausible_files_are_described_with_a_warning() {
    let dir = std::env::temp_dir().join(format!("luxpath-cli-warn-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("bright.ies");
    // 1000 cd all round from a 1000 lm lamp: 4π × 1000 lm, 1256.6 % of the lamp flux.
    std::fs::write(&path, "IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 2 1 1 2 0 0 0\n1 1 0\n0 180\n0\n1000 1000\n").unwrap();
    let (code, out, _) = luxpath(&["photometry", path.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(out.contains("warning") && out.contains("1256.6 %"), "{out}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_broken_file_is_an_error_with_its_line() {
    let dir = std::env::temp_dir().join(format!("luxpath-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("broken.ies");
    std::fs::write(&path, "IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 2 1 1 2 0 0 0\n1 1 0\n0 90\n0\n100\n").unwrap();
    let (code, _, err) = luxpath(&["photometry", path.to_str().unwrap()]);
    assert_eq!(code, 2);
    assert!(err.contains("line 7: the file ends before the candela values"), "{err}");
    std::fs::remove_dir_all(&dir).ok();
}
