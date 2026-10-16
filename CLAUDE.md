# luxpath — instructions for agents

luxpath is an open-source, Rust-native lighting calculation tool. It starts with **emergency lighting to
EN 1838:2024** (escape routes, open areas, high-risk task areas, points of emphasis) and is designed to grow into
general interior (EN 12464-1) and road (EN 13201) lighting. It runs as a CLI first, then in the browser via WASM
and as a desktop app.

## Start every session here
1. Read `docs/status.md`: the current milestone, the next task, blockers and open questions.
2. Read `docs/gaps.md` (ranked shortfalls: the work list) and the milestone in `docs/roadmap.md`. Prefer the top
   of `gaps.md` over new features.
3. Read `docs/architecture.md` and the doc comment of the crate you are touching.
4. Work in this loop: **orient → plan → implement + test → verify → record → commit.** Recording means:
   update `docs/status.md`, the `parity/features.toml` entry, `docs/gaps.md` if a gap moved, and add a line to the
   progress log in `ROADMAP.md`.

`plan/` is git-ignored and local-only (purchased standards, observation notes of other tools). The
machine-readable checklist lives in `parity/` (committed).

## Non-negotiables
- **`AGENTS.md` first**: provenance (standards' text never committed, clean-room, no GPL/AGPL), life-safety
  honesty, never panic. It overrides this file.
- **Layering.** Crates depend only on lower layers (`cargo xtask layers`). Core crates do no I/O and never depend
  on UI crates.
- **Units and conventions.** Metres, lux, candela, lumens; z up; degrees in public APIs. Name units in field
  names or doc comments (`flux_lm`, `rotation_deg`).
- **Review every change as you make it, at four zooms.**
  - *Clause:* `get()` not `[i]`, NaN/inf guards, `<` against `<=` at every threshold (a 1.0 lx requirement:
    is 0.9999 a fail?), degrees against radians.
  - *Function:* is it total? Name an input that panics; walk empty, zero, max, malformed.
  - *Module:* one source of truth, no dead code, doc comment still true.
  - *Project:* layering, wasm, the parity entry, the docs.
- **Tests at the right scale:** unit → property → crate integration → end-to-end CLI. Requirement checks get
  tests on both sides of every threshold. **Prove a new test fails** by reverting the fix (or breaking the
  code) and watching it go red.
- **Hand-checkable numbers.** Prefer test cases whose expected value can be computed on paper (isotropic sources,
  points straight below) and write the arithmetic in a comment.

## Quality gates (before every commit)
`cargo xtask ci` runs them all: `cargo fmt --check`, clippy with `-D warnings`, the tests, the wasm check
(needs `rustup target add wasm32-unknown-unknown`), `xtask layers`, `xtask parity`. Scope to a crate while
iterating: `cargo test -p luxpath-calc`.

## Commits and PRs
`main` is the integration branch. Branch as `<type>/<slug>` (`feat/`, `fix/`, `docs/`, `chore/`, `refactor/`), one
task id per commit (e.g. `M1.2: EULUMDAT reader`), commit only green states, open a PR using
`.github/pull_request_template.md`, merge when CI is green.

## Running
- `cargo run -p luxpath-cli -- --help`
- `cargo run -p luxpath-cli -- point 2.5 3` (direct illuminance demo)
- `cargo xtask parity` (feature progress by milestone)
