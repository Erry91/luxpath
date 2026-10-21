# Status

> **Last updated:** 2026-10-21

**Milestone:** M2 done (calculation engine); M1 done (photometry readers). **Direction under study:** EN 1838 checks on IFC models
(docs/research.md §6–7). The IFC inspector (`luxpath inspect-ifc`, an early piece of M7) is done.

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
- **C. M3 scene:** the TOML scene format and `luxpath check` (then obstruction, gap 16, before M4).
- **D. Real files:** photometric files of luminaires the owner's practice uses, privately in `plan/` (gap 12).

## Blockers
- None for M3. **Before M4:** the EN 1838:2024 text (Italian edition UNI EN 1838:2025) is needed to verify values (gap 4).

## Open questions (owner)
- Practitioner questions in `docs/research.md` §5.
