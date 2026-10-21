# Architecture

> **Last updated:** 2026-10-16 · **Change:** initial

## Data flow

```text
 IES / LDT files ──► photometry ──┐
                                  ├──► calc ──► rules (EN 1838) ──► report ──► text / JSON / SVG / PDF
 scene.toml ──────► scene ────────┘
                     ▲
 DXF plans, IFC models (M7) ─────┘
```

The apps (`luxpath-cli` now; a web calculator and an editor later) read files, call the core crates, and write
results. Core crates take `&str`/`&[u8]` and return values: they do no file or network I/O, so the same code runs
natively, in tests and in the browser.

## Layers

A crate may depend only on crates in **strictly lower** layers. `cargo xtask layers` enforces this, and fails if a
new crate isn't listed in `xtask/src/layers.rs`.

| Layer | Crate | Responsibility |
|---|---|---|
| L0 | `luxpath-geom` | Vectors, angles, plan polygons. Conventions: metres, z up, degrees in APIs |
| L1 | `luxpath-photometry` | C-γ intensity tables (cd/klm), IES and EULUMDAT readers, symmetry expansion |
| L2 | `luxpath-scene` | Rooms, routes, areas, points of emphasis, placed luminaires; the TOML format |
| L3 | `luxpath-calc` | Direct illuminance at points and over grids; statistics |
| L3 | `luxpath-ifc` | Reads IFC (BIM) models through `ifc-lite-core`; today the inspector, later scenes built from models |
| L4 | `luxpath-rules` | Evaluates a scene (results per element, assumptions); from M4 standards checks with clause, value, threshold, verdict |
| L5 | `luxpath-report` | Text, JSON, SVG (later PDF) output |
| L9 | `luxpath-cli`, `xtask` | Applications and tooling; only these may do I/O or use UI crates |

## Decisions

- **Direct light only, by default.** Emergency-lighting practice (and the free tools' defaults) ignores reflections;
  this keeps results conservative and the maths checkable by hand. Reflections are an explicit later option (M9).
- **cd/klm internally.** Intensities are stored per 1000 lm (the EULUMDAT convention); flux and maintenance factor
  are applied at calculation time, so one table serves mains and emergency modes.
- **Verdicts are three-valued:** pass, fail, cannot verify. Missing data is never a pass.
- **Text formats for everything a user writes** (scenes in TOML), so designs can be reviewed and diffed.
- **Own photometry readers.** Re-evaluated 2026-10-21 before M1: `eulumdat` and `gldf-rs` are AGPL-3.0 and
  `l3d_rs` GPL-3.0 (excluded, AGENTS.md §1.2). `lidrs` (MIT) is the only permissive one, but its development
  stopped in 2022 (later commits are dependency bumps); it has `todo!()` for type A/B files, no LM-63-2019, panics
  on 126 of about 18,000 mutated inputs, and accepts 85 of 415 truncated IES files as valid. The formats are
  small, so luxpath has its own readers (`luxpath-photometry`), built to AGENTS.md §3.
