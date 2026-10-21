# luxpath roadmap

**Stage: pre-alpha** · next: alpha (M1–M4 + M6 validation) · ~150–250 agent-assisted hours away

> **Last updated:** 2026-10-21 · **Change:** M3 (scene and `luxpath check`) done

luxpath's one-page summary. Details live in:

| Document | Holds |
|---|---|
| [docs/roadmap.md](docs/roadmap.md) | Milestones, exit criteria, estimates, risks |
| [docs/gaps.md](docs/gaps.md) | Ranked shortfalls: the work list |
| [docs/status.md](docs/status.md) | Current task and blockers |
| [docs/standards-parity.md](docs/standards-parity.md) | EN 1838:2024 coverage |

## Stages

| Stage | Gate |
|---|---|
| **pre-alpha** | Building the engine. Nothing is trustworthy yet. |
| **alpha** | `luxpath check` verifies a synthetic floor against every P0 EN 1838 requirement, and results agree with an independent tool on ≥10 synthetic scenes (M6). Every requirement value is checked against the standard's text. |
| **beta** | DXF plan import, SVG/PDF reports, web calculator. A real designer has used it on a real project, side by side with their usual tool. |
| **1.0** | Plan editor; no known wrong results; documentation a designer can follow. |

## Next up

1. **M4:** the EN 1838:2024 checks (needs the standard's text, gap 4).
2. **Real photometric files** from the owner's practice, kept in `plan/` (gap 12).
3. **Open question:** buy EN 1838:2024 (around €100 from a national standards body) to verify the requirement values.

## Progress log

| Date | Change |
|---|---|
| 2026-10-21 | Walls block light (gap 16): `[[wall]]` in scenes, sight lines tested against every wall piece; the corridor example's minimum drops from 1.45 to 0.60 lx once light no longer passes through its corner; 100 luminaires × 10,000 points × 36 walls in about 90 ms. |
| 2026-10-21 | Publishing prepared (docs/publishing.md): history scanned clean, `cargo deny` with a CI job, SECURITY.md, CONTRIBUTING.md, issue templates (wrong result, bug), `publish = false` until a crates.io decision. |
| 2026-10-21 | M3: scene format version 1 (docs/scene-format.md: photometry with emergency output, luminaires, routes, open and high-risk areas, points of emphasis; unknown fields refused, errors with line and column); `luxpath check` with exit codes 0/1/2/3, results per element with their assumptions, "cannot verify" when an emergency output is missing; three example scenes with synthetic photometry. |
| 2026-10-21 | M2: luminaires with heading and tilt (IES tilt factors applied), illuminance on any surface, area and full-width route grids summed over luminaires, statistics that are unverifiable when any point is; isotropic and Lambertian cases within 1e-6; 100 luminaires × 10,000 points in about 55 ms (release, in `cargo xtask ci`); photometry plausibility warnings. |
| 2026-10-21 | M1: IES LM-63 (1986–2019) and EULUMDAT readers, symmetry expansion, tilt data, emergency flux, located errors, property tests, a 10-minute fuzz run (22.8 million inputs) without a panic; `luxpath photometry`. Existing crates evaluated first: none usable (docs/architecture.md). |
| 2026-10-16 | M7 (early): `luxpath-ifc` and `luxpath inspect-ifc`; 9 public models inspected, findings in docs/research.md §7. |
| 2026-10-16 | M0: workspace, layering and parity gates, C-γ intensity table, direct illuminance, demo CLI, docs. |
