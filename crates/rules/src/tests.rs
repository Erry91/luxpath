use super::*;
use luxpath_photometry::read_ies;

/// 1000 cd/klm all round (an isotropic source) from 1000 lm lamps.
fn isotropic() -> Photometry {
    read_ies(b"IESNA:LM-63-2002\nTILT=NONE\n1 1000 1 2 1 1 2 0 0 0\n1 1 0\n0 180\n0\n1000 1000\n").unwrap()
}

fn scene(emergency: &str) -> Scene {
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
fn photometry_must_match_the_scene() {
    assert!(evaluate(&scene(""), &[]).is_err());
}
