# Status

> **Last updated:** 2026-10-25

**Milestone:** M3 done (scene and `luxpath check`); M1–M2 done (photometry readers, engine). **Direction under study:** EN 1838 checks on IFC models
(docs/research.md §6–7). The IFC inspector (`luxpath inspect-ifc`, an early piece of M7) is done.

## EN 1838:2024 clause map (2026-10-25)
`docs/standards-parity.md` now cites the 2024 clause of every requirement, from the table of contents of the free
SIST EN 1838:2025 preview; the values are still from secondary sources (gap 4). The preview also shows what the 2024
edition added, now rows and parity entries: specific areas (5.4: toilets and changing rooms, public indoor swimming
pools, plant rooms), safety signs (6), local area lighting (7.1). Next: the scene's area kinds for 5.4 and 7.1, and a
finding per requirement, each "cannot verify" until its value is verified.

## Walls (2026-10-21)
Scenes have `[[wall]]`s (polylines with a height); a luminaire whose sight line to a point crosses one gives that
point nothing. In the corridor example the minimum on the route drops from 1.45 lx to 0.60 lx once light no longer
passes through the corner: the wrong pass gap 16 warned about. Next: Ashlar's emergency-lighting plugin (R2.9a)
builds scenes from BIM models with this engine.

## M3 (2026-10-21)
Scenes are TOML files, version 1 (`docs/scene-format.md`): photometric files with their emergency output,
luminaires (heading, tilt), escape routes, open and high-risk areas, points of emphasis, calculation settings.
Unknown fields are errors, and every error names its line and column; the maintenance factor and emergency outputs
have no defaults. `luxpath check scene.toml` prints illuminance per route, area and point with the assumptions, and
exits 0 pass, 1 fail, 2 input error, 3 cannot verify; until M4 checks requirements, it always answers 3. Three
example scenes in `examples/`.

## M2 (2026-10-21)
`luxpath-calc` places luminaires with a heading and a tilt (90° = wall-mounted; IES tilt factors applied, a tilt
outside the file's table refused), computes direct illuminance on any surface (horizontal, vertical, inclined),
builds grids over areas (less a border) and along routes (over their full width), sums every luminaire and gives
min, mean, max, max/min and min/mean; one undefined point makes the statistics unverifiable. Isotropic and
Lambertian sources match their closed forms within 1e-6; 100 luminaires × 10,000 points take about 55 ms.
`luxpath photometry` warns about implausible files. The engine still treats luminaires as points and ignores walls
(gap 16, an alpha blocker before M4).

## M1 (2026-10-21)
`luxpath-photometry` reads IES LM-63 (1986–2019, type C, relative and absolute, tilt tables) and EULUMDAT (every
symmetry indicator, Latin-1, decimal commas) into one full C-γ table; errors name their line. Synthetic files of
every symmetry read identically from both formats; a 10-minute fuzz run (`cargo xtask fuzz`) found no panic.
`luxpath photometry <file>` shows what a file says. Of two real files tried, both read; one (the LM-63 example)
emits 211 % of its lamp flux, which gap 13 will flag. No usable existing crate (docs/architecture.md).

## What the inspector showed
Public Revit models carry luminaires, spaces, doors and stairs, but no photometry, no emergency marking and no
FireExit flags. Luminaires collapse into few types (410 → 5), so a photometry mapping by type name is practical.

## Next task (to decide with the owner)
- **A. Evidence:** inspect one real project IFC from a practising office (privately, in `plan/`).
- **B. IFC scene:** read luminaire positions and space outlines (geometry) and define the mapping file
  (type name or product code → photometric file, emergency yes/no).
- **E. Public** since 2026-10-22 (`docs/publishing.md`); crates.io waits for alpha.
- **D. Real files:** photometric files of luminaires the owner's practice uses, privately in `plan/` (gap 12).

## Blockers
- None. **Before M4:** the EN 1838:2024 text (Italian edition UNI EN 1838:2025) is needed to verify values (gap 4).
  The free SIST preview (`plan/standards/`) gave the 2024 clause numbers and the list of changes, but no values.

## Open questions (owner)
- Practitioner questions in `docs/research.md` §5.
