# Contributing to luxpath

Thank you for helping. luxpath computes emergency lighting, a life-safety subject, so a few rules come before
everything else. They are in [AGENTS.md](AGENTS.md) (they bind people and AI agents alike); the short version:

- **Never commit a standard's text, tables or figures** (EN 1838, EN 50172, IES, CIE…). Requirement values may be
  encoded with their clause and edition, in our own words (`docs/standards-parity.md`).
- **Clean-room with respect to other lighting software.** Compare results with DIALux, Relux and others on your own
  synthetic scenes; never decompile them, and never copy their luminaire databases, artwork or sample projects.
- **No GPL or AGPL code**, not even as a dependency (`cargo deny check` enforces the licences).
- **Never commit manufacturers' photometric files or real building plans.** Test fixtures are synthetic and listed
  with their source in a `README.md` next to them.
- **A wrong "pass" is the worst bug.** Missing or ambiguous data gives "cannot verify". Requirement checks are
  tested on both sides of every threshold.
- **Never panic on input.** No `unwrap`, indexing or unchecked arithmetic on anything read from a file.

## Where help is most useful

- Photometric files that luxpath reads wrongly or refuses (gaps 12 and 15 in `docs/gaps.md`): a description, or a
  synthetic file reproducing the quirk.
- Comparing luxpath's results with DIALux evo or Relux on synthetic scenes (milestone M6, `docs/roadmap.md`).
- National editions and rules around EN 1838 (`docs/standards-parity.md`).
- The work list: `docs/gaps.md`, top rows first.

## How to work

1. Read `docs/status.md`, `docs/gaps.md` and `docs/architecture.md`; `CLAUDE.md` describes the day-to-day workflow.
2. Branch as `<type>/<slug>` (`feat/`, `fix/`, `docs/`, `chore/`, `refactor/`).
3. Run `cargo xtask ci` (format, clippy, tests, the performance budget, the WASM build, layers, the feature
   checklist) and, for licence changes, `cargo deny check`.
4. Open a pull request with the template; record what changed in `parity/features.toml`, `docs/status.md`,
   `docs/gaps.md` and the `ROADMAP.md` log.

By contributing, you agree that your contributions are licensed under the project's licence, MIT OR Apache-2.0.
