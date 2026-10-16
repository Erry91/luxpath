# Status

> **Last updated:** 2026-10-16

**Milestone:** M0 done. **Direction under study:** EN 1838 checks on IFC models (docs/research.md §6–7). The IFC
inspector (`luxpath inspect-ifc`, an early piece of M7) is done.

## What the inspector showed
Public Revit models carry luminaires, spaces, doors and stairs, but no photometry, no emergency marking and no
FireExit flags. Luminaires collapse into few types (410 → 5), so a photometry mapping by type name is practical.

## Next task (to decide with the owner)
- **A. Evidence:** inspect one real project IFC from a practising office (privately, in `plan/`).
- **B. IFC scene:** read luminaire positions and space outlines (geometry) and define the mapping file
  (type name or product code → photometric file, emergency yes/no).
- **C. M1 photometry:** the IES/EULUMDAT readers. Needed by every direction.

## Blockers
- None for M1. **Before M4:** the EN 1838:2024 text (Italian edition UNI EN 1838:2025) is needed to verify values (gap 4).

## Open questions (owner)
- Practitioner questions in `docs/research.md` §5.
