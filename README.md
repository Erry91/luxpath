# luxpath

**Open, scriptable emergency-lighting calculations, in pure Rust.**

luxpath checks emergency escape lighting designs against **EN 1838:2024**: escape routes, open (anti-panic) areas,
high-risk task areas and points of emphasis. You describe a floor in a readable text file and give it photometric
files (IES, EULUMDAT). luxpath computes the illuminance and tells you which requirements pass, which fail and why.

> **Status: pre-alpha (M0).** The calculation core exists and is tested. Reading photometric files, the scene
> format and the EN 1838 checks are next. See [ROADMAP.md](ROADMAP.md).

## Why

Lighting designers already have free tools (DIALux evo, ReluxDesktop), but they are closed, mostly Windows-only and
GUI-only. luxpath aims to be:

- **Open and transparent.** Every number can be traced to a formula and a clause of the standard.
- **Scriptable and reproducible.** Scenes are diffable text, results are JSON, and the CLI has CI-friendly exit codes.
- **Everywhere.** macOS, Linux, Windows, and the browser via WebAssembly. One core can also be embedded as a
  calculator on a manufacturer's website.
- **Current.** Built for the 2024 edition (full escape-route width) from day one.

## Try it

```sh
cargo run -p luxpath-cli -- point 2.5 3      # direct illuminance 3 m away from a source mounted at 2.5 m
cargo run -p luxpath-cli -- photometry crates/photometry/tests/fixtures/downlight.ldt   # read an IES or LDT file
cargo xtask ci                               # every quality gate
cargo xtask parity                           # feature progress by milestone
```

## Documentation

| Document | Holds |
|---|---|
| [ROADMAP.md](ROADMAP.md) | One-page summary: stage, next steps, progress log |
| [docs/roadmap.md](docs/roadmap.md) | Milestones M0–M9 with exit criteria and estimates |
| [docs/gaps.md](docs/gaps.md) | Every known shortfall, ranked: the work list |
| [docs/status.md](docs/status.md) | Where work stands right now, blockers, open questions |
| [docs/architecture.md](docs/architecture.md) | Crates, layers, data flow, design decisions |
| [docs/standards-parity.md](docs/standards-parity.md) | EN 1838:2024 requirements and their implementation status |
| [docs/research.md](docs/research.md) | Market and standards research behind the project |
| [parity/features.toml](parity/features.toml) | Machine-readable feature checklist (`cargo xtask parity`) |
| [AGENTS.md](AGENTS.md) / [CLAUDE.md](CLAUDE.md) | Rules and working instructions for contributors and agents |

## Disclaimer

luxpath produces calculations, not certificates. Emergency lighting is life-safety equipment: a competent designer
remains responsible for every design, and for checking it against the standard and national regulations in force.

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.
