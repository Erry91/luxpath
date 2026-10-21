//! Mutation fuzzing of the readers (AGENTS.md §3: no input may crash them).
//!
//! Seeds are synthetic files of every symmetry, with and without tilt data; each round mutates one
//! (truncation, deleted, repeated or swapped lines, a number replaced by an awkward one, random
//! bytes) and feeds it to every reader. `fuzz_briefly` runs on every `cargo test`; the long run is
//!
//! ```text
//! LUXPATH_FUZZ_SECONDS=600 cargo test --release -p luxpath-photometry --test fuzz -- --ignored
//! ```
//!
//! (`cargo xtask fuzz [seconds]`). A crashing input is written to the temporary directory and named
//! in the failure.

#![allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::panic)]

mod common;

use std::time::{Duration, Instant};

use common::every_symmetry;
use luxpath_photometry::{read, read_eulumdat, read_ies};

const AWKWARD: &[&str] = &[
    "-1",
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "-0",
    "1e308",
    "-1e308",
    "1e-320",
    "99999999",
    "1000000",
    "4294967296",
    "nan",
    "inf",
    "",
    "x",
    "0.5",
    "89.9999999",
    "90",
    "180",
    "270",
    "360",
    "360.0000001",
    "720",
    "INCLUDE",
    "NONE",
    "TILT=",
    "[MORE]",
    ",",
    "1,5",
];

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        if n == 0 { 0 } else { (self.next() % n as u64) as usize }
    }

    /// A line index, half the time among the first 60, where the headers and counts are.
    fn line(&mut self, n: usize) -> usize {
        if self.next().is_multiple_of(2) { self.below(n.min(60)) } else { self.below(n) }
    }
}

fn seeds() -> Vec<Vec<u8>> {
    let mut seeds =
        vec![include_bytes!("fixtures/downlight.ies").to_vec(), include_bytes!("fixtures/downlight.ldt").to_vec()];
    for stored in every_symmetry() {
        let ies = stored.ies();
        seeds.push(ies.clone().into_bytes());
        seeds.push(ies.replace("TILT=NONE\n", "TILT=INCLUDE\n1\n3\n0 45 90\n1 0.9 0.8\n").into_bytes());
        seeds.push(stored.ldt().into_bytes());
    }
    seeds
}

fn mutate(rng: &mut Rng, seed: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(seed).into_owned();
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    let rounds = 1 + rng.below(3);
    let mut bytes = Vec::new();
    for _ in 0..rounds {
        match rng.below(7) {
            0 => {
                let keep = rng.below(lines.len() + 1);
                lines.truncate(keep);
            }
            1 if !lines.is_empty() => {
                let i = rng.line(lines.len());
                lines.remove(i);
            }
            2 if !lines.is_empty() => {
                let i = rng.line(lines.len());
                let line = lines[i].clone();
                lines.insert(i, line);
            }
            3 if lines.len() > 1 => {
                let (a, b) = (rng.line(lines.len()), rng.below(lines.len()));
                lines.swap(a, b);
            }
            4 | 5 if !lines.is_empty() => {
                // Replace one word of one line with an awkward value.
                let i = rng.line(lines.len());
                let mut words: Vec<String> = lines[i].split_whitespace().map(String::from).collect();
                let w = rng.below(words.len() + 1);
                let awkward = AWKWARD[rng.below(AWKWARD.len())].to_string();
                if w < words.len() {
                    words[w] = awkward;
                } else {
                    words.push(awkward);
                }
                lines[i] = words.join(" ");
            }
            _ => {
                // Random bytes somewhere in the file.
                bytes = lines.join("\n").into_bytes();
                for _ in 0..1 + rng.below(4) {
                    let at = rng.below(bytes.len() + 1);
                    bytes.insert(at, (rng.next() >> 24) as u8);
                }
                lines = String::from_utf8_lossy(&bytes).lines().map(String::from).collect();
            }
        }
    }
    if bytes.is_empty() || rng.below(2) == 0 {
        lines.join(if rng.below(2) == 0 { "\n" } else { "\r\n" }).into_bytes()
    } else {
        bytes
    }
}

/// Fuzzes for `duration`; returns the number of inputs tried.
fn fuzz(duration: Duration, seed: u64) -> usize {
    let seeds = seeds();
    let mut rng = Rng(seed | 1);
    let start = Instant::now();
    let mut tried = 0;
    while start.elapsed() < duration {
        let pick = rng.below(seeds.len());
        let input = mutate(&mut rng, &seeds[pick]);
        let outcome = std::panic::catch_unwind(|| {
            let _ = read(&input);
            let _ = read_ies(&input);
            let _ = read_eulumdat(&input);
        });
        if outcome.is_err() {
            let path = std::env::temp_dir().join(format!("luxpath-fuzz-crash-{seed}-{tried}.txt"));
            std::fs::write(&path, &input).unwrap();
            panic!("a reader panicked on the input saved at {}", path.display());
        }
        tried += 1;
    }
    tried
}

#[test]
fn fuzz_briefly() {
    let tried = fuzz(Duration::from_secs(2), 0x5eed);
    assert!(tried > 100, "only {tried} inputs in two seconds");
}

#[test]
#[ignore = "long run: LUXPATH_FUZZ_SECONDS=600 cargo test --release -p luxpath-photometry --test fuzz -- --ignored"]
fn fuzz_long() {
    let seconds = std::env::var("LUXPATH_FUZZ_SECONDS").ok().and_then(|s| s.parse().ok()).unwrap_or(600);
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
    let tried = fuzz(Duration::from_secs(seconds), seed);
    eprintln!("fuzzed {tried} inputs in {seconds} s (seed {seed}) without a panic");
}
