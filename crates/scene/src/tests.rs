use super::*;

const SCENE: &str = r#"format = "luxpath-scene"
version = 1

[project]
name = "Corridor"
maintenance_factor = 0.8

[calculation]
spacing_m = 0.5

[[photometry]]
id = "bulkhead"
file = "photometry/bulkhead.ldt"
emergency = { ballast_lumen_factor = 0.5 }

[[photometry]]
id = "spot"
file = "spot.ies"

[[luminaire]]
id = "L1"
photometry = "bulkhead"
position = [2.0, 1.0, 2.8]
rotation_deg = 90

[[route]]
id = "R1"
name = "Corridor to exit A"
path = [[0, 1], [10, 1], [10, 6]]
width_m = 2.0

[[area]]
id = "A1"
kind = "high-risk"
outline = [[0, 3], [6, 3], [6, 8], [0, 8]]
normal_lighting_lx = 500

[[point]]
id = "P1"
kind = "fire-fighting"
position = [10.5, 6.0, 1.2]
facing = [-2, 0, 0]
"#;

#[test]
fn a_scene_reads_into_its_model() {
    let s = Scene::parse(SCENE).unwrap();
    assert_eq!((s.name.as_str(), s.maintenance_factor), ("Corridor", 0.8));
    assert_eq!(s.calculation, Calculation { spacing_m: 0.5, ..Calculation::default() });
    assert_eq!(s.photometry[0].emergency, Some(EmergencyOutput::BallastLumenFactor(0.5)));
    assert_eq!(s.photometry[1].emergency, None);
    assert_eq!(s.luminaires[0].photometry, 0);
    assert_eq!(s.luminaires[0].rotation_deg, 90.0);
    assert_eq!(s.routes[0].path.len(), 3);
    assert_eq!(s.areas[0].kind, AreaKind::HighRisk);
    assert_eq!(s.areas[0].normal_lighting_lx, Some(500.0));
    assert_eq!(s.points[0].kind, PointKind::FireFighting);
    // The facing direction is normalised.
    assert_eq!(s.points[0].facing, Some(Vec3::new(-1.0, 0.0, 0.0)));
}

/// `SCENE` with `from` replaced by `to`.
fn with(from: &str, to: &str) -> String {
    assert!(SCENE.contains(from), "{from}");
    SCENE.replacen(from, to, 1)
}

#[test]
fn malformed_scenes_give_located_errors() {
    let cases: Vec<(String, usize, &str)> = vec![
        (with("version = 1", "version = 2"), 2, "version 2"),
        (with("maintenance_factor = 0.8", "maintenance_factor = 1.2"), 6, "at most 1"),
        (with("maintenance_factor = 0.8\n", ""), 4, "maintenance_factor"),
        (with("spacing_m = 0.5", "spacing = 0.5"), 9, "unknown field"),
        (with("spacing_m = 0.5", "spacing_m = 0"), 9, "above 0"),
        (with("photometry = \"bulkhead\"", "photometry = \"bulk\""), 22, "no [[photometry]] has the id `bulk`"),
        (with("id = \"R1\"", "id = \"L1\""), 27, "used twice"),
        (with("path = [[0, 1], [10, 1], [10, 6]]", "path = [[0, 1]]"), 29, "at least 2 points"),
        (with("width_m = 2.0", "width_m = -2.0"), 30, "above 0"),
        (with("kind = \"high-risk\"", "kind = \"stairs\""), 34, "not an area kind"),
        (with("kind = \"high-risk\"", "kind = \"open\""), 36, "high-risk areas only"),
        (with("kind = \"fire-fighting\"", "kind = \"extinguisher\""), 40, "not a point kind"),
        (with("facing = [-2, 0, 0]", "facing = [0, 0, 0]"), 42, "must not be zero"),
        (with("emergency = { ballast_lumen_factor = 0.5 }", "emergency = { }"), 14, "either"),
        (with("position = [2.0, 1.0, 2.8]", "position = [2.0, 1.0]"), 23, ""),
        (with("format = \"luxpath-scene\"", "format = \"dialux\""), 1, "luxpath-scene"),
        (with("[project]", "[project\n"), 4, ""),
    ];
    for (text, line, words) in cases {
        let e = Scene::parse(&text).unwrap_err();
        assert_eq!(e.line, line, "{e}");
        assert!(e.column > 0, "{e}");
        assert!(e.to_string().contains(words), "{e} should mention {words}");
    }
}

#[test]
fn huge_or_hostile_input_is_an_error_not_a_panic() {
    assert!(Scene::parse(&"#".repeat(MAX_SCENE_BYTES + 1)).is_err());
    for text in ["", "\u{0}", "[[[", "version = 1e999", "format = \"luxpath-scene\"\nversion = -1"] {
        assert!(Scene::parse(text).is_err(), "{text:?}");
    }
}
