//! M1's exit test: one distribution per symmetry, written as IES and as EULUMDAT, reads into the
//! same full table, and every direction carries the value of the stored plane it mirrors.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use common::{Stored, all_c, every_symmetry};
use luxpath_photometry::{Format, IesEdition, Symmetry, read, read_eulumdat, read_ies};
use proptest::prelude::*;

/// The stored C angle that `c` mirrors, by the symmetry's definition.
fn mirrored(symmetry: Symmetry, c: f64) -> f64 {
    match symmetry {
        Symmetry::None => c,
        Symmetry::Rotational => 0.0,
        Symmetry::PlaneC0C180 => {
            if c <= 180.0 {
                c
            } else {
                360.0 - c
            }
        }
        Symmetry::PlaneC90C270 => {
            if (90.0..=270.0).contains(&c) {
                c
            } else {
                (180.0 - c).rem_euclid(360.0)
            }
        }
        Symmetry::Quadrants => {
            let c = c % 180.0;
            if c > 90.0 { 180.0 - c } else { c }
        }
    }
}

fn check(stored: &Stored) {
    let ies = read_ies(stored.ies().as_bytes()).unwrap();
    let ldt = read_eulumdat(stored.ldt().as_bytes()).unwrap();
    assert_eq!(ies.format(), Format::Ies(IesEdition::Lm63_2002));
    assert_eq!(ldt.format(), Format::Eulumdat);
    assert_eq!((ies.symmetry(), ldt.symmetry()), (stored.symmetry, stored.symmetry));
    assert_eq!(ies.flux_lm(), stored.flux_lm);
    assert_eq!(ldt.flux_lm(), stored.flux_lm);
    assert_eq!(ies.table(), ldt.table(), "{:?}", stored.symmetry);

    let planes = stored.stored_c();
    let full_c = if stored.symmetry == Symmetry::Rotational { vec![0.0] } else { all_c(stored.c_step) };
    assert_eq!(ldt.table().c_angles(), &full_c[..]);
    for c in &full_c {
        let source = mirrored(stored.symmetry, *c);
        let p = planes.iter().position(|a| (a - source).abs() < 1e-9).unwrap();
        for (g, gamma) in stored.gamma.iter().enumerate() {
            let want = stored.values[p * stored.gamma.len() + g];
            assert_eq!(ldt.table().intensity(*c, *gamma), want, "{:?} C{c} γ{gamma}", stored.symmetry);
        }
    }
}

#[test]
fn every_symmetry_reads_the_same_from_ies_and_eulumdat() {
    for stored in every_symmetry() {
        check(&stored);
    }
}

#[test]
fn read_tells_the_formats_apart() {
    for stored in every_symmetry() {
        assert!(matches!(read(stored.ies().as_bytes()).unwrap().format(), Format::Ies(_)));
        assert_eq!(read(stored.ldt().as_bytes()).unwrap().format(), Format::Eulumdat);
    }
}

#[test]
fn the_committed_fixtures_read_and_agree() {
    let ies = read(include_bytes!("fixtures/downlight.ies")).unwrap();
    let ldt = read(include_bytes!("fixtures/downlight.ldt")).unwrap();
    assert_eq!(ies.table(), ldt.table());
    assert_eq!(ies.flux_lm(), 1000.0);
    // 300 cd/klm straight down at 1000 lm is 300 cd.
    assert_eq!(ldt.candela(0.0, 0.0, 1000.0), 300.0);
}

fn symmetry() -> impl Strategy<Value = Symmetry> {
    prop_oneof![
        Just(Symmetry::None),
        Just(Symmetry::Rotational),
        Just(Symmetry::PlaneC0C180),
        Just(Symmetry::PlaneC90C270),
        Just(Symmetry::Quadrants),
    ]
}

fn stored() -> impl Strategy<Value = Stored> {
    let c_step = prop::sample::select(vec![5.0, 7.5, 10.0, 15.0, 22.5, 30.0, 45.0, 90.0]);
    let gamma = prop_oneof![
        prop::sample::select(vec![1.0, 2.5, 5.0, 10.0, 45.0]).prop_map(|s| (0.0, 180.0, s)),
        prop::sample::select(vec![2.5, 5.0, 30.0]).prop_map(|s| (0.0, 90.0, s)),
        prop::sample::select(vec![5.0, 90.0]).prop_map(|s| (90.0, 180.0, s)),
    ]
    .prop_map(|(from, to, step): (f64, f64, f64)| {
        let n = ((to - from) / step).round() as usize;
        (0..=n).map(|i| from + i as f64 * step).collect::<Vec<f64>>()
    });
    (symmetry(), c_step, gamma, any::<u64>(), 1u32..100_000).prop_map(|(symmetry, c_step, gamma, seed, flux)| {
        Stored {
            flux_lm: f64::from(flux),
            ..Stored::new(symmetry, c_step, gamma, move |p, g| {
                // Values from 0 to 2000 in half candelas, which every step represents exactly.
                let h = seed.wrapping_mul(6364136223846793005).wrapping_add((p * 1000 + g) as u64).rotate_left(17);
                (h % 4001) as f64 / 2.0
            })
        }
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn any_distribution_reads_the_same_from_both_formats(stored in stored()) {
        // Flux values that do not halve exactly make IES and EULUMDAT differ in the last bit.
        let ies = read_ies(stored.ies().as_bytes()).unwrap();
        let ldt = read_eulumdat(stored.ldt().as_bytes()).unwrap();
        prop_assert_eq!(ies.table().c_angles(), ldt.table().c_angles());
        prop_assert_eq!(ies.table().gamma_angles(), ldt.table().gamma_angles());
        for (a, b) in ies.table().values().iter().zip(ldt.table().values()) {
            prop_assert!((a - b).abs() <= 1e-9 * b.max(1.0), "{} vs {}", a, b);
        }
    }

    #[test]
    fn mirrored_directions_have_equal_intensities(stored in stored(), c in 0.0f64..360.0, gamma in 0.0f64..180.0) {
        let t = read_eulumdat(stored.ldt().as_bytes()).unwrap();
        let t = t.table();
        let m = mirrored(stored.symmetry, c);
        prop_assert!((t.intensity(c, gamma) - t.intensity(m, gamma)).abs() < 1e-9);
    }

    #[test]
    fn arbitrary_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..2000)) {
        let _ = read(&bytes);
        let _ = read_ies(&bytes);
        let _ = read_eulumdat(&bytes);
    }
}
