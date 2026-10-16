# luxpath roadmap

**Stage: pre-alpha** · next: alpha (M1–M4 + M6 validation) · ~150–250 agent-assisted hours away

> **Last updated:** 2026-10-16 · **Change:** project created (M0 scaffold)

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

1. **M1:** IES LM-63 and EULUMDAT readers, symmetry expansion, emergency-mode flux.
2. **M2:** grids, sums over luminaires, tilt, vertical illuminance.
3. **Open question:** buy EN 1838:2024 (around €100 from a national standards body) to verify the requirement values.

## Progress log

| Date | Change |
|---|---|
| 2026-10-16 | M7 (early): `luxpath-ifc` and `luxpath inspect-ifc`; 9 public models inspected, findings in docs/research.md §7. |
| 2026-10-16 | M0: workspace, layering and parity gates, C-γ intensity table, direct illuminance, demo CLI, docs. |
