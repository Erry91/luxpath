//! M2's performance budget: 100 luminaires × 10,000 points in under 100 ms, with a realistic table
//! (C every 5°, γ every 1°). Timing means something only in release builds, so this test is
//! ignored by default and `cargo xtask ci` runs it with `--release -- --ignored`.

#![allow(clippy::unwrap_used)]

use std::time::{Duration, Instant};

use luxpath_calc::{Grid, Luminaire, Stats, Wall, illuminance, illuminance_with_walls};
use luxpath_geom::{Vec2, Vec3};
use luxpath_photometry::IntensityTable;

fn scene() -> (Vec<Luminaire>, Grid) {
    let c: Vec<f64> = (0..72).map(|i| f64::from(i) * 5.0).collect();
    let g: Vec<f64> = (0..=180).map(f64::from).collect();
    let values = c.iter().flat_map(|c| g.iter().map(move |g| 100.0 + c / 10.0 + (180.0 - g))).collect();
    let table = IntensityTable::new(c, g, values).unwrap();
    let luminaires: Vec<Luminaire> = (0..100)
        .map(|i| Luminaire {
            position: Vec3::new(f64::from(i % 10) * 5.0, f64::from(i / 10) * 5.0, 3.0),
            rotation_deg: f64::from(i) * 7.0,
            tilt_deg: f64::from(i % 3) * 10.0,
            distribution: table.clone(),
            flux_lm: 200.0,
            maintenance_factor: 0.8,
        })
        .collect();
    let floor = [Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0), Vec2::new(50.0, 50.0), Vec2::new(0.0, 50.0)];
    let grid = Grid::area(&floor, 0.0, 0.5, 0.0).unwrap();
    assert_eq!(grid.points().len(), 10_000);
    (luminaires, grid)
}

/// The best of five runs of `f`.
fn best_of_five(f: impl Fn() -> Vec<Option<f64>>) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..5 {
        let start = Instant::now();
        let e = f();
        best = best.min(start.elapsed());
        assert!(Stats::of(&e).is_some());
    }
    best
}

#[test]
#[ignore = "timing: run in release (cargo xtask ci does)"]
fn a_hundred_luminaires_over_ten_thousand_points_within_budget() {
    let (luminaires, grid) = scene();
    let best = best_of_five(|| illuminance(&luminaires, &grid));
    eprintln!("100 luminaires × 10,000 points: {best:?} (best of 5)");
    assert!(best < Duration::from_millis(100), "{best:?} is over the 100 ms budget");
}

#[test]
#[ignore = "timing: run in release (cargo xtask ci does)"]
fn walls_stay_within_budget() {
    // The 50 m floor cut into 5 m rooms by nine wall lines each way, each in two pieces around a 2 m doorway: 36 walls.
    let (luminaires, grid) = scene();
    let mut walls = Vec::new();
    for line in 1..10 {
        let at = f64::from(line) * 5.0;
        for (from, to) in [(0.0, 23.0), (25.0, 50.0)] {
            walls.push(Wall {
                a: luxpath_geom::Vec2::new(at, from),
                b: luxpath_geom::Vec2::new(at, to),
                bottom_m: 0.0,
                top_m: 3.0,
            });
            walls.push(Wall {
                a: luxpath_geom::Vec2::new(from, at),
                b: luxpath_geom::Vec2::new(to, at),
                bottom_m: 0.0,
                top_m: 3.0,
            });
        }
    }
    let best = best_of_five(|| illuminance_with_walls(&luminaires, &walls, &grid));
    eprintln!("100 luminaires × 10,000 points × {} walls: {best:?} (best of 5)", walls.len());
    assert!(best < Duration::from_millis(500), "{best:?} is over the 500 ms budget");
}
