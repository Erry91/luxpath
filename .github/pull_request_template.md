## What this changes

<!-- What behaviour is different afterwards, and why. `Fixes #123` if it closes an issue. -->

## How you verified it

- [ ] `cargo xtask ci` (or, while iterating, `cargo test -p <crate>` on the crates I changed)
- [ ] New behaviour has a test at the right scale; I watched it **fail** before the fix
- [ ] Requirement checks are tested on **both sides** of every threshold

## Life safety (AGENTS.md §2)

- [ ] No path reports "pass" when data is missing or ambiguous ("cannot verify" instead)
- [ ] Assumptions that changed are shown in reports

## Never panic (AGENTS.md §3)

- [ ] No new `unwrap`/`expect`/`panic!`/indexing on input outside tests; NaN/inf guarded; allocations capped
- [ ] Any crash this fixes has a synthetic regression test

## Provenance (AGENTS.md §1)

- [ ] No text, tables or figures from a standard; values cite edition and clause
- [ ] Nothing derived from the internals of closed lighting tools; no GPL/AGPL code
- [ ] No manufacturer photometric files or real building plans committed

## Records

- [ ] `parity/features.toml` updated (`notes` say what is still missing)
- [ ] `docs/status.md`, `docs/gaps.md` and the `ROADMAP.md` progress log updated where they changed
