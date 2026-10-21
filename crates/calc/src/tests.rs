use super::*;
use luxpath_geom::Vec2;

fn isotropic_at(height: f64) -> Luminaire {
    Luminaire {
        position: Vec3::new(0.0, 0.0, height),
        rotation_deg: 0.0,
        tilt_deg: 0.0,
        distribution: IntensityTable::isotropic(1000.0).unwrap(),
        flux_lm: 1000.0,
        maintenance_factor: 1.0,
    }
}

/// I = 1000 cd/klm × cos γ below the horizon, none above; γ every 0.1° so that linear
/// interpolation stays within 4e-7 of the cosine (error ≤ Δγ²/8 = (0.1° in rad)²/8).
fn lambertian_at(height: f64) -> Luminaire {
    let gammas: Vec<f64> = (0..=1800).map(|i| f64::from(i) / 10.0).collect();
    let values = gammas.iter().map(|g| if *g <= 90.0 { 1000.0 * g.to_radians().cos() } else { 0.0 }).collect();
    Luminaire { distribution: IntensityTable::new(vec![0.0], gammas, values).unwrap(), ..isotropic_at(height) }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-6 * b.abs().max(1e-12)
}

#[test]
fn directly_below_is_intensity_over_distance_squared() {
    // 1000 cd at 2 m: E = 1000 / 4 = 250 lx.
    let e = direct_horizontal_illuminance(&isotropic_at(2.0), Vec3::new(0.0, 0.0, 0.0)).unwrap();
    assert!((e - 250.0).abs() < 1e-9);
}

#[test]
fn off_axis_points_follow_the_cosine_law() {
    // Point 2 m across, 2 m down: d² = 8, cos θ = 1/√2 → E = 1000 · 0.7071 / 8.
    let e = direct_horizontal_illuminance(&isotropic_at(2.0), Vec3::new(2.0, 0.0, 0.0)).unwrap();
    assert!((e - 1000.0 * std::f64::consts::FRAC_1_SQRT_2 / 8.0).abs() < 1e-9);
}

#[test]
fn a_lambertian_source_matches_its_closed_form() {
    // I = I₀ cos γ, and on the floor cos γ = cos θ = h/d: E = I₀ h² / d⁴.
    let lum = lambertian_at(2.0);
    for r in [0.0, 0.5, 1.5, 3.0, 8.0] {
        let d2: f64 = 4.0 + r * r;
        let expected = 1000.0 * 4.0 / (d2 * d2);
        let e = direct_horizontal_illuminance(&lum, Vec3::new(r, 0.0, 0.0)).unwrap();
        assert!(close(e, expected), "r {r}: {e} against {expected}");
    }
}

#[test]
fn vertical_illuminance_follows_the_cosine_law_on_the_wall() {
    // Isotropic 1000 cd at 2 m; a wall surface at (3, 0, 0) facing back towards the luminaire
    // (normal −x): cos θ = 3/d, so E = 1000 · 3 / d³ with d = √13.
    let lum = isotropic_at(2.0);
    let d = 13f64.sqrt();
    let e = direct_illuminance(&lum, Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)).unwrap();
    assert!(close(e, 3000.0 / (d * d * d)));
    // Facing away, it gets nothing.
    assert_eq!(direct_illuminance(&lum, Vec3::new(3.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)), Some(0.0));
    assert_eq!(direct_illuminance(&lum, Vec3::new(3.0, 0.0, 0.0), Vec3::default()), None);
}

/// 1000 cd/klm over the lower hemisphere of its own frame, none above.
fn downlight() -> IntensityTable {
    IntensityTable::new(vec![0.0], vec![0.0, 90.0], vec![1000.0, 1000.0]).unwrap()
}

#[test]
fn a_tilted_luminaire_aims_its_light_towards_c0() {
    // Tilted 90°: its nadir points along +x. A wall 2 m ahead facing back gets I/d² = 1000/4;
    // behind it, nothing (its upper hemisphere is dark).
    let lum = Luminaire { tilt_deg: 90.0, distribution: downlight(), ..isotropic_at(2.0) };
    let ahead = direct_illuminance(&lum, Vec3::new(2.0, 0.0, 2.0), Vec3::new(-1.0, 0.0, 0.0)).unwrap();
    assert!(close(ahead, 250.0));
    let behind = direct_illuminance(&lum, Vec3::new(-2.0, 0.0, 2.0), Vec3::new(1.0, 0.0, 0.0)).unwrap();
    assert_eq!(behind, 0.0);
    // Turned to a heading of 90°, it aims along +y instead.
    let turned = Luminaire { rotation_deg: 90.0, ..lum };
    let north = direct_illuminance(&turned, Vec3::new(0.0, 2.0, 2.0), Vec3::new(0.0, -1.0, 0.0)).unwrap();
    assert!(close(north, 250.0));
    // Behind it (−y) is its dark upper hemisphere.
    assert_eq!(direct_illuminance(&turned, Vec3::new(0.0, -2.0, 2.0), Vec3::new(0.0, 1.0, 0.0)), Some(0.0));
}

#[test]
fn flux_and_maintenance_factor_scale_linearly() {
    let mut lum = isotropic_at(2.0);
    lum.flux_lm = 200.0;
    lum.maintenance_factor = 0.8;
    let e = direct_horizontal_illuminance(&lum, Vec3::default()).unwrap();
    assert!((e - 250.0 * 0.2 * 0.8).abs() < 1e-9);
}

#[test]
fn rotation_selects_the_c_plane() {
    // C0 = 200 cd/klm, C180 = 100 cd/klm, flat in γ.
    let table = IntensityTable::new(vec![0.0, 180.0], vec![0.0, 180.0], vec![200.0, 200.0, 100.0, 100.0]).unwrap();
    let mut lum = isotropic_at(1.0);
    lum.distribution = table;
    let east = Vec3::new(1.0, 0.0, 0.0);
    let unrotated = direct_horizontal_illuminance(&lum, east).unwrap();
    lum.rotation_deg = 180.0;
    let rotated = direct_horizontal_illuminance(&lum, east).unwrap();
    assert!((unrotated - 2.0 * rotated).abs() < 1e-9);
}

#[test]
fn points_level_with_or_above_the_luminaire_get_nothing() {
    let lum = isotropic_at(2.0);
    assert_eq!(direct_horizontal_illuminance(&lum, Vec3::new(5.0, 0.0, 2.0)), Some(0.0));
    assert_eq!(direct_horizontal_illuminance(&lum, Vec3::new(0.0, 0.0, 3.0)), Some(0.0));
}

#[test]
fn degenerate_inputs_are_none_not_a_panic() {
    let lum = isotropic_at(2.0);
    assert_eq!(direct_horizontal_illuminance(&lum, Vec3::new(0.0, 0.0, 2.0)), None);
    assert_eq!(direct_horizontal_illuminance(&lum, Vec3::new(f64::NAN, 0.0, 0.0)), None);
    let mut bad = isotropic_at(2.0);
    bad.flux_lm = f64::INFINITY;
    assert_eq!(direct_horizontal_illuminance(&bad, Vec3::default()), None);
    bad = Luminaire { tilt_deg: f64::NAN, ..isotropic_at(2.0) };
    assert_eq!(direct_horizontal_illuminance(&bad, Vec3::default()), None);
}

#[test]
fn grids_sum_every_luminaire() {
    // Two isotropic 1000 cd sources at 2 m, at x = ±2; the point between them gets 2 × 1000 · (2/√8) / 8.
    let a = Luminaire { position: Vec3::new(-2.0, 0.0, 2.0), ..isotropic_at(2.0) };
    let b = Luminaire { position: Vec3::new(2.0, 0.0, 2.0), ..isotropic_at(2.0) };
    let grid = Grid::new(vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(-2.0, 0.0, 0.0)], Vec3::UP).unwrap();
    let e = illuminance(&[a.clone(), b.clone()], &grid);
    let one = 1000.0 * std::f64::consts::FRAC_1_SQRT_2 / 8.0;
    assert!(close(e[0].unwrap(), 2.0 * one));
    // Under a: 250 lx from a, plus b from 4 m across: 1000 · (2/√20) / 20.
    assert!(close(e[1].unwrap(), 250.0 + 1000.0 * 2.0 / 20f64.sqrt() / 20.0));
    // No luminaires: 0 lx everywhere (dark, not unknown).
    assert_eq!(illuminance(&[], &grid), vec![Some(0.0), Some(0.0)]);
}

#[test]
fn a_wall_hides_a_luminaire_from_the_points_behind_it() {
    // A corridor turning a corner: the luminaire at (0, 0, 2.5); a 3 m wall along x = 2 from y = −1 to 4.
    // The point (4, 0, 0) is behind it and gets nothing; (1, 0, 0) is in front and gets the
    // unobstructed 1000 · 2.5 / d³ with d² = 1 + 6.25.
    let lum = Luminaire { position: Vec3::new(0.0, 0.0, 2.5), ..isotropic_at(2.5) };
    let wall = Wall { a: Vec2::new(2.0, -1.0), b: Vec2::new(2.0, 4.0), bottom_m: 0.0, top_m: 3.0 };
    let grid = Grid::new(vec![Vec3::new(4.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)], Vec3::UP).unwrap();
    let e = illuminance_with_walls(std::slice::from_ref(&lum), &[wall], &grid);
    assert_eq!(e[0], Some(0.0));
    assert!(close(e[1].unwrap(), 1000.0 * 2.5 / 7.25f64.powf(1.5)));
    // Without the wall, the hidden point is lit.
    assert!(illuminance(&[lum], &grid)[0].unwrap() > 0.0);
}

#[test]
fn an_undefined_point_makes_the_statistics_unverifiable() {
    let lum = isotropic_at(2.0);
    let grid = Grid::new(vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 2.0)], Vec3::UP).unwrap();
    let e = illuminance(&[lum], &grid);
    assert_eq!(e[1], None);
    assert_eq!(Stats::of(&e), None);
}

#[test]
fn an_area_under_one_luminaire_matches_the_closed_form_point_by_point() {
    let lum = lambertian_at(2.5);
    let room = [Vec2::new(-3.0, -2.0), Vec2::new(3.0, -2.0), Vec2::new(3.0, 2.0), Vec2::new(-3.0, 2.0)];
    let grid = Grid::area(&room, 0.0, 0.25, 0.5).unwrap();
    let e = illuminance(&[lum], &grid);
    for (p, e) in grid.points().iter().zip(&e) {
        let d2 = p.x * p.x + p.y * p.y + 2.5 * 2.5;
        assert!(close(e.unwrap(), 1000.0 * 2.5 * 2.5 / (d2 * d2)));
    }
    let stats = Stats::of(&e).unwrap();
    assert!(stats.max_to_min().unwrap() > 1.0);
}

#[test]
fn tilt_factors_from_the_file_scale_the_light_and_unknown_tilts_are_errors() {
    let ies = "IESNA:LM-63-2002\nTILT=INCLUDE\n1\n3\n0 45 90\n1 0.9 0.8\n1 1000 1 2 1 1 2 0 0 0\n1 1 0\n0 180\n0\n1000 1000\n";
    let p = luxpath_photometry::read_ies(ies.as_bytes()).unwrap();
    let at = |tilt: f64| Luminaire::from_photometry(&p, Vec3::new(0.0, 0.0, 2.0), 0.0, tilt, 1000.0, 1.0);
    let level = direct_horizontal_illuminance(&at(0.0).unwrap(), Vec3::default()).unwrap();
    let tilted = direct_horizontal_illuminance(&at(45.0).unwrap(), Vec3::default()).unwrap();
    assert!(close(tilted, 0.9 * level));
    assert_eq!(at(100.0).unwrap_err(), CalcError::TiltOutsideTable { tilt_deg: 100.0 });
}
