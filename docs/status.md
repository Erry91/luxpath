# Status

> **Last updated:** 2026-10-16

**Milestone:** M0 done → starting **M1 (photometry)**.

## Next task
**M1.1: IES LM-63 reader** in `luxpath-photometry` (`ies` module): parse from `&str` the header keywords, TILT
(NONE/INCLUDE; FILE → "cannot verify"), lamp count and lumens, candela multiplier, angle lists, values. Normalise
to cd/klm (absolute photometry, lumens = -1, needs a stated rule). Errors carry line numbers. Synthetic fixtures
under `crates/photometry/tests/fixtures/`.

Then M1.2 EULUMDAT reader, M1.3 symmetry expansion (both formats), M1.4 emergency-mode flux, M1.5 property tests
and a fuzz target.

## Blockers
- None for M1. **Before M4:** the EN 1838:2024 text is needed to verify values (gap 4).

## Open questions (owner)
- Final project name (working name `luxpath`, free on crates.io as of 2026-10-16).
- Publish to GitHub now, or after M1?
- Practitioner questions in `docs/research.md` §5.
