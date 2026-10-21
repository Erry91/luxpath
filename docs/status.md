# Status

> **Last updated:** 2026-10-21

**Milestone:** M1 done (photometry readers). **Direction under study:** EN 1838 checks on IFC models
(docs/research.md §6–7). The IFC inspector (`luxpath inspect-ifc`, an early piece of M7) is done.

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
- **C. M2 engine:** grids, sums over luminaires, tilted luminaires, vertical illuminance (with M1's photometry).
- **D. Real files:** photometric files of luminaires the owner's practice uses, privately in `plan/` (gap 12).

## Blockers
- None for M2. **Before M4:** the EN 1838:2024 text (Italian edition UNI EN 1838:2025) is needed to verify values (gap 4).

## Open questions (owner)
- Practitioner questions in `docs/research.md` §5.
