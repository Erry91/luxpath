use super::*;
use luxpath_photometry::read_ies;

/// 1000 cd/klm all round (an isotropic source) from 1000 lm lamps.
fn isotropic() -> Photometry {
    read_ies(b"IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 2 1 1 2 0 0 0\n1 1 0\n0 180\n0\n1000 1000\n").unwrap()
}

fn scene(emergency: &str) -> Scene {
    scene_with(emergency, "")
}

fn scene_with(emergency: &str, extra: &str) -> Scene {
    Scene::parse(&format!(
        r#"format = "luxpath-scene"
version = 1
[project]
name = "t"
maintenance_factor = 0.8
[calculation]
spacing_m = 1.0
route_points_across = 1
[[photometry]]
id = "iso"
file = "iso.ies"
{emergency}
[[luminaire]]
id = "L1"
photometry = "iso"
position = [0, 0, 2]
[[route]]
id = "R1"
path = [[0, 0], [2, 0]]
width_m = 1
[[point]]
id = "P1"
kind = "fire-fighting"
position = [3, 0, 2]
facing = [-1, 0, 0]
{extra}
"#
    ))
    .unwrap()
}

fn computed(o: &Outcome) -> Stats {
    match o {
        Outcome::Computed(s) => *s,
        Outcome::CannotVerify(why) => panic!("cannot verify: {why}"),
    }
}

#[test]
fn routes_and_points_are_computed_in_emergency_mode() {
    let e = evaluate(&scene("emergency = { flux_lm = 500 }"), &[isotropic()]).unwrap();
    // 1000 cd/klm at 500 lm × MF 0.8 is 400 cd. The route's centre line at 0, 1, 2 m from below
    // a 2 m mounting: E = 400 · h / d³, so 100, 400·2/5^1.5 = 71.55, 400·2/8^1.5 = 35.36 lx.
    let route = computed(&e.results[0].horizontal);
    assert!((route.max_lx - 100.0).abs() < 1e-9);
    assert!((route.min_lx - 400.0 * 2.0 / 8f64.powf(1.5)).abs() < 1e-9);
    assert_eq!(route.points, 3);
    // The point 3 m away at the luminaire's height, facing it: E = 400 / 9 lx vertically and 0
    // horizontally (the light arrives level).
    assert_eq!(computed(&e.results[1].horizontal).max_lx, 0.0);
    assert!((computed(e.results[1].vertical.as_ref().unwrap()).max_lx - 400.0 / 9.0).abs() < 1e-9);
    assert_eq!(e.verdict, Verdict::CannotVerify);
    assert!(e.assumptions.iter().any(|a| a.contains("500 lm in emergency mode")));
    // An isotropic table emits far more than its lamps: warned.
    assert!(e.warnings.iter().any(|w| w.contains("more light than the lamps")));
}

#[test]
fn a_missing_emergency_output_leaves_everything_unverifiable() {
    let e = evaluate(&scene(""), &[isotropic()]).unwrap();
    for r in &e.results {
        assert!(matches!(&r.horizontal, Outcome::CannotVerify(why) if why.contains("declares no emergency output")));
    }
    let e = evaluate(&scene("emergency = { ballast_lumen_factor = 1.5 }"), &[isotropic()]).unwrap();
    assert!(matches!(&e.results[0].horizontal, Outcome::CannotVerify(why) if why.contains("ballast lumen factor")));
}

#[test]
fn a_wall_between_luminaire_and_point_darkens_it() {
    // A 3 m wall along x = 2.5 hides the point at x = 3 from the only luminaire at x = 0.
    let wall = "[[wall]]\nid = \"W1\"\npath = [[2.5, -1], [2.5, 1]]\nheight_m = 3\n";
    let e = evaluate(&scene_with("emergency = { flux_lm = 500 }", wall), &[isotropic()]).unwrap();
    assert_eq!(computed(e.results[1].vertical.as_ref().unwrap()).max_lx, 0.0);
    // The route (x from 0 to 2) is on the luminaire's side and unchanged.
    assert!((computed(&e.results[0].horizontal).max_lx - 100.0).abs() < 1e-9);
    assert!(e.assumptions.iter().any(|a| a.contains("walls block the light (1 straight piece)")));
}

#[test]
fn a_route_given_as_its_floor_is_computed_over_all_of_it() {
    // A 2 m × 1 m corridor floor at 1 m spacing: cell centres (0.5, 0.5) and (1.5, 0.5).
    // From 400 cd at (0, 0, 2): E = 400 · 2 / d³ with d² = 0.5 + 4 and 2.5 + 4.
    let route = "[[route]]\nid = \"R2\"\noutline = [[0, 0], [2, 0], [2, 1], [0, 1]]\n";
    let e = evaluate(&scene_with("emergency = { flux_lm = 500 }", route), &[isotropic()]).unwrap();
    let r2 = e.results.iter().find(|r| r.id == "R2").unwrap();
    let s = computed(&r2.horizontal);
    assert_eq!(s.points, 2);
    assert!((s.max_lx - 800.0 / 4.5f64.powf(1.5)).abs() < 1e-9);
    assert!((s.min_lx - 800.0 / 6.5f64.powf(1.5)).abs() < 1e-9);
}

#[test]
fn photometry_must_match_the_scene() {
    assert!(evaluate(&scene(""), &[]).is_err());
}

#[test]
fn every_element_gets_a_finding_per_requirement_citing_its_clause() {
    let areas = ["open", "high-risk", "toilets", "swimming-pool", "plant-room", "local-area"]
        .iter()
        .enumerate()
        .map(|(i, kind)| {
            let normal = if *kind == "high-risk" { "normal_lighting_lx = 300\n" } else { "" };
            format!("[[area]]\nid = \"A{i}\"\nkind = \"{kind}\"\noutline = [[0, 1], [2, 1], [2, 3], [0, 3]]\n{normal}")
        })
        .collect::<String>();
    let e = evaluate(&scene_with("emergency = { flux_lm = 500 }", &areas), &[isotropic()]).unwrap();
    let cited: Vec<(&str, &str)> = e.findings.iter().map(|f| (f.requirement.clause, f.element.as_str())).collect();
    assert_eq!(
        cited,
        [
            ("5.1", "R1"),
            ("5.1", "R1"),
            ("5.2", "A0"),
            ("5.3", "A1"),
            ("5.4.1", "A2"),
            ("5.4.2", "A3"),
            ("5.4.3", "A4"),
            ("7.1", "A5"),
            ("4.2", "P1"),
        ]
    );
    // Nothing passes or fails while the values are unverified (gap 4), and the verdict says how many wait.
    assert!(e.findings.iter().all(|f| f.verdict == Verdict::CannotVerify && f.reason.contains("gap 4")));
    assert_eq!(e.verdict, Verdict::CannotVerify);
    assert!(e.verdict_reason.contains("9 findings cannot be verified"), "{}", e.verdict_reason);
}

#[test]
fn a_finding_on_an_element_that_cannot_be_computed_says_why() {
    let e = evaluate(&scene(""), &[isotropic()]).unwrap();
    assert!(!e.findings.is_empty());
    for f in &e.findings {
        assert_eq!(f.verdict, Verdict::CannotVerify);
        assert!(f.reason.contains("declares no emergency output"), "{}", f.reason);
    }
}

/// The requirements in code and the two lists people read must agree: every requirement's id is a feature in
/// `parity/features.toml`, and its clause is on a row of `docs/standards-parity.md` with that id.
#[test]
fn every_requirement_is_in_the_parity_list_and_the_coverage_table_with_its_clause() {
    let features = include_str!("../../../parity/features.toml");
    let coverage = include_str!("../../../docs/standards-parity.md");
    for r in en1838::REQUIREMENTS {
        assert!(features.contains(&format!("id = \"{}\"", r.id)), "{} is not in parity/features.toml", r.id);
        let on_a_row = coverage
            .lines()
            .filter(|l| l.contains(&format!("`{}`", r.id)))
            .any(|l| l.split('|').nth(3).is_some_and(|clause| clause.split_whitespace().any(|w| w == r.clause)));
        assert!(on_a_row, "{} (clause {}) has no row in docs/standards-parity.md", r.id, r.clause);
    }
}
