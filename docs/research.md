# Research

> **Last updated:** 2026-10-16 · **Change:** initial findings (web research; vendor sources marked)

Why luxpath exists, and what it has to beat. Sources are listed at the end. Many are vendor pages, so their
claims are marketing until checked.

## 1. Does the emergency calculation use direct light only?

- **In practice, yes by default.** DIALux evo's emergency lighting (since evo 10, 2021) excludes room-surface
  reflections and furniture by default. It can include the first reflection for indirect luminaires as an option.
  ReluxDesktop and Litestar also offer EN 1838 workflows.
- **The standard's own wording is not confirmed.** No source found states the clause. To verify against the text
  (gap 4).
- **Design consequence:** direct light only is the default and is stated in every report. First reflection is an
  explicit option (`calc.first-reflection`, M9).

## 2. The 2024 edition

- EN 1838:2024 replaces EN 1838:2013 (BS EN 1838:2024 in force 31 Dec 2024; the UK NSI certification transition
  runs to 31 May 2027). A Dutch guide cites NEN-EN 1838:2025. In Italy it is **UNI EN 1838:2025**, in force
  since 6 Feb 2025; UNI EN 1838:2013 is withdrawn on 18 June 2027 ([Certifico](https://www.certifico.com/normazione/news-normazione/uni-en-1838-2025-illuminazione-di-emergenza-per-gli-edifici)).
- **Getting the text:** the standard is sold by national bodies (UNI, BSI, DIN Media, AFNOR…). Buying it is a
  licence to read the official text; it is not a certification. We need it so that the values we encode come from
  the source and not from summaries (gap 4).
- **The main change:** escape-route illuminance must cover the **full route width**, not only the centre line.
  Designs made to the 2013 centre-line rule may no longer comply. This is a concrete reason for a new tool.
- Also new: a procedure for escape routes through open areas, local area lighting, more detail for points of
  emphasis and high-risk areas, and an informative Annex C (lockdown and prolonged power loss).
- Read with EN 50172:2024.

## 3. Existing tools

| Tool | Cost | Open | Platform | EN 1838 | Notes |
|---|---|---|---|---|---|
| DIALux evo (DIAL) | free | no | Windows (macOS unconfirmed) | yes, based on the 2019 version of EN 1838 per DIAL's evo 10 announcement | Escape routes, anti-panic, high-risk areas, scenes, live false colour |
| ReluxDesktop | free | no | Windows | yes | Auto-placement, live results, visibility range tool |
| ReluxEmergencyCalc | API for manufacturers | no | web service | 2018 or **2024** | Manufacturers embed it in their websites and datasheets |
| LITESTAR 4D Litecalc (Oxytech) | free tier; auto-placement paid | no | Windows | yes | Common in Italy |
| Zumtobel RESCLITE PRO | free | no | iPad | yes | Zumtobel products only |
| Radiance | free | yes (C) | CLI | no | General ray-tracing engine; no standard checks |

**Conclusion:** free closed tools cover GUI planning well. luxpath's openings are where they are weak:
1. **Open and transparent:** traceable numbers, reviewable code.
2. **Scriptable:** text scenes, JSON results, CI.
3. **Embeddable:** an open WASM alternative to calculator APIs like ReluxEmergencyCalc, for manufacturers and websites.
4. **Cross-platform and web.**
5. **2024 edition first.**

## 4. Rust ecosystem

| Crate | What | Licence | Use? |
|---|---|---|---|
| `eulumdat` (holg) | LDT/IES parse/write, diagrams | **AGPL-3.0** | no (AGENTS.md §1.2) |
| `gldf-rs` (holg) | GLDF container | MIT | maybe, M7 |
| `lidrs` | IES/LDT read/write | check | unmaintained since 2022 |

I found no open-source calculation tool for EN 1838 or EN 12464, in Rust or any other language.

## 5. Open questions for practitioners

To ask architects and lighting designers (for example, through a contact who uses AutoCAD daily):
- Which tool does your office use for emergency lighting, on which OS, and who does the calculation (in-house,
  the electrical engineer, the manufacturer)?
- What annoys you about it? What do you hand over (PDF report, DWG with symbols)?
- Would plan import from DWG/DXF matter more than a better calculator?
- Have you had to redo designs for the 2024 full-width rule?

## Sources
- DIALux: [emergency lighting](https://www.dialux.com/en-GB/emergency-lighting), [press release](https://dialux.com/en-GB/press/emergency-lighting-with-dialux-evo), [evo 10 download](https://www.dial.de/en-GB/news/now-available-for-download-dialux-evo-10)
- Relux: [EmergencyCalc](https://relux.com/manufacturers/oem/emergencycalc), [ReluxDesktop](https://relux.com/assets/static/desktop22_en.html)
- [LITESTAR 4D emergency](https://oxytech.it/en/software/litestar-4d-suite/litecalc/emergency/?idC=61848), [Zumtobel RESCLITE PRO](https://apps.apple.com/us/app/id439253350)
- EN 1838:2024: [NSI notice](https://documents.nsi.org.uk/wp-content/uploads/2025/01/NSI-002-25-Publication-of-BS-EN-1838_2024.pdf), [Ansell Lighting summary](https://ansell-lighting.com/en/articles/technical/emergency-lighting-en-18382024-bs-en-18382024), [BSI](https://knowledge.bsigroup.com/products/lighting-applications-emergency-lighting-for-buildings), [NEN 1010 §560 guide](https://nen.sulerr.com/en/guides/nen-1010/560-noodverlichting)
- Values: [TRILUX escape routes](https://trilux.com/en/lighting-practice/indoor-lighting/general-requirements/emergency-lighting/safety-lighting-for-escape-routes), [LEDiL](https://www.ledil.com/news_all/articles-and-whitepapers/this-is-an-emergency-how-the-right-light-can-help-you-out/), [Fagerhult](https://www.fagerhult.com/Support-center/Emergency-lighting/)
- Crates: [eulumdat](https://docs.rs/crate/eulumdat/latest), [gldf-rs](https://docs.rs/crate/gldf-rs/latest), [lidrs](https://docs.rs/crate/lidrs/latest)
