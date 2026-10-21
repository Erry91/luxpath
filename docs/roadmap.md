# Milestones

> **Last updated:** 2026-10-21 · **Change:** M1–M3 done

Estimates are agent-assisted hours (a human directing an agent, reviewing every change) and are rough until M2
calibrates them. Each milestone ends with its exit criteria met, `cargo xtask ci` green, and the parity entries
updated.

| M | Name | Scope | Exit criteria | Est. h | Status |
|---|---|---|---|---|---|
| M0 | Foundations | Workspace, lints, CI, `xtask` gates (ci, layers, parity), docs, intensity table, direct point illuminance, demo CLI | All gates green; one hand-checked calculation end to end | 5–8 | **done** |
| M1 | Photometry | IES LM-63 (1995/2002/2019) and EULUMDAT readers; symmetry expansion; tilt data; emergency-mode flux; validation errors with line numbers; property tests and fuzzing | Synthetic fixtures for every symmetry type read identically from IES and LDT; fuzzing finds no panic in 10 min | 20–35 | **done** |
| M2 | Engine | Calculation grids (areas, route bands), sums over luminaires, tilted/wall-mounted luminaires, vertical illuminance, min/avg/max/uniformity statistics | Analytic test cases (isotropic, Lambertian) match to 1e-6; performance: 100 luminaires × 10k points < 100 ms | 20–30 | **done** |
| M3 | Scene and CLI | TOML scene format (versioned), 2D geometry (polygons, routes with width), `luxpath check scene.toml` with exit codes | Three example scenes in `examples/`; malformed scenes give located errors | 20–30 | **done** |
| M4 | EN 1838:2024 | Every requirement in `docs/standards-parity.md`: escape routes over the full width, uniformity, open areas, high-risk areas, points of emphasis, glare limits, declared duration/response | Each P0 requirement tested on both sides of its threshold; values verified against the standard's text | 25–40 | planned |
| M5 | Reports | Text, JSON (stable schema), SVG plan with false colours and failing points; PDF later | A report a designer can read without knowing the tool | 15–25 | planned |
| M6 | Validation | Black-box comparison with an independent tool (DIALux evo, Relux) on ≥10 synthetic scenes; divergences explained in `docs/validation.md` | Agreement within a stated tolerance, or a documented reason | 10–20 + human | planned |
| M7 | Plans | IFC models (inspector done; scenes from models with a photometry mapping by product code or type name), DXF import, automatic placement suggestions, GLDF reading | A DXF floor plan becomes a checkable scene in under 10 minutes | 30–50 | planned |
| M8 | Apps | WASM web calculator (single route/area, embeddable), then a plan editor (egui, desktop and web) | Web calculator deployed; editor can draw, place, compute, report | 60–100 | planned |
| M9 | Beyond emergency | First reflection, then inter-reflection (radiosity); EN 12464-1 interior checks with UGR; EN 13201 road lighting | Each standard gets its own parity doc before work starts | 100+ | planned |

**Alpha** = M1–M4 + M6. **Beta** = + M5, M7, the web calculator from M8.

## Risks

| Risk | Mitigation |
|---|---|
| A wrong "pass" in a life-safety context | Both-sides threshold tests, "cannot verify" instead of guessing, M6 cross-validation, report disclaimers (AGENTS.md §2) |
| Standard text not available to contributors | Values with clause references in our own words; unverified values are marked; buy the standard before alpha |
| Free closed tools are "good enough" for most users | Focus where they are weak: open, scriptable, CI, cross-platform, web-embeddable, 2024 edition |
| Photometric files in the wild are messy | Lenient readers that record every repair; fuzzing; a private corpus in `plan/` (never committed) |
| Scope creep into a full DIALux | Emergency lighting first; M9 only after beta |
