//! Standards checks: turns calculated illuminances into pass/fail findings with evidence.
//!
//! First standard (M4): **EN 1838:2024**, emergency lighting for buildings. Every requirement is
//! a row in `docs/standards-parity.md` with its clause number; every check cites that clause in its
//! finding. Values come from the standard itself, paraphrased: never copy the standard's text or
//! tables into this repository (`AGENTS.md` §1).
//!
//! Later: EN 12464-1 (interior workplaces) and EN 13201 (road lighting), as separate modules.
//!
//! Layer L4: depends on `luxpath-scene` and `luxpath-calc`.
