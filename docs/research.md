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

## 6. Direction under study: emergency lighting checks on IFC (BIM) models

> Added 2026-10-16. The question: instead of competing with DIALux, could luxpath check emergency lighting directly
> on an IFC model exported from Revit, Archicad or Bonsai?

### What the IFC schema can carry (good)
- **Luminaires:** `IfcLightFixture` / `IfcLightFixtureType`. Since IFC4, `PredefinedType = SECURITYLIGHTING` means
  "directing occupants in an emergency" (exit signs, emergency lights).
- **`Pset_LightFixtureTypeSecurityLighting`:** SecurityLightingType, FixtureHeight, SelfTestFunction,
  BackupSupplySystem, PictogramEscapeDirection, Addressablility.
- **Photometry:** `IfcLightSourceGoniometric` (flux, colour temperature, emission source) with a distribution that is
  either an **external file** (EULUMDAT, IES, CIBSE TM14) or an **inline table** (`IfcLightIntensityDistribution`,
  normally cd/klm, our internal unit).
- **Escape-relevant building data:** `IfcSpace` with `Pset_SpaceFireSafetyRequirements.FireExit` (corridors used as
  exits), `Pset_DoorCommon.FireExit`, `IfcSpatialZone` FIRESAFETY (compartments), `IfcStair`. Fire equipment and
  call points are also IFC element types.
- **No dedicated escape-route entity.** Routes are computed from the space graph (TU Wien, FireBIM guideline,
  graph-based egress research), not stored.

### What exporters actually write (weak)
- DIAL's own documentation of DIALux Bridge for Revit: luminaires already placed in Revit arrive in DIALux as
  **placeholders without photometric data**.
- A DIALux forum user reports the reverse: IFC exported from DIALux, opened in Revit, showed no IES distribution.
- Sample check (BIMCompNet dataset, IFC4 components extracted from real models): `IfcLightFixture` with **no
  PredefinedType**, and only `Pset_LightFixtureTypeCommon.TotalWattage` and
  `Pset_ManufacturerTypeInformation.Manufacturer`. No goniometric source, no file reference.
- GitHub code search: zero IFC files containing `IFCLIGHTSOURCEGONIOMETRIC` (it doesn't index large files).
- **Conclusion:** expect positions, orientations, types and a manufacturer or product code, but **no photometry**.
  It must be linked by us (product code → IES/LDT/GLDF file), or the user must assign it.

### Competition in BIM
- **DIALux Bridge for Revit** (DIALux Pro subscription, Revit 2025, IFC round trip, builds Revit families to
  ANSI/IES TM-32-24, the IES standard for BIM luminaire parameters). Revit-only and paid.
- No open-source tool found that checks lighting requirements on IFC.

### What the idea would look like
1. Read IFC (reuse `ifc-lite` crates, MPL-2.0).
2. Find emergency luminaires, then link photometry through a mapping file (type or product code → photometric file).
3. **Infer what EN 1838 needs from the model:** corridors flagged FireExit become escape routes checked over their
   **full floor area** (matching the 2024 full-width rule); large spaces become open areas with the border excluded;
   exit doors, stairs, fire equipment and call points become points of emphasis.
4. Run the luxpath engine.
5. Report failures as **BCF** issues that open in the architect's own tool, plus an IDS file describing the data
   the model must contain.

Risks: model quality varies (missing spaces, FireExit rarely set); the photometry mapping is manual work; DIALux
Bridge covers part of the Revit market; the users would be BIM coordinators and fire engineers more than lighting
designers.

Sources: [IfcLightSourceGoniometric](https://standards.buildingsmart.org/IFC/RELEASE/IFC4/FINAL/HTML/schema/ifcpresentationorganizationresource/lexical/ifclightsourcegoniometric.htm),
[IfcLightDistributionDataSourceSelect](https://standards.buildingsmart.org/IFC/RELEASE/IFC4/FINAL/HTML/schema/ifcpresentationorganizationresource/lexical/ifclightdistributiondatasourceselect.htm),
[Pset_LightFixtureTypeSecurityLighting](https://standards.buildingsmart.org/IFC/RELEASE/IFC4_3/HTML/lexical/Pset_LightFixtureTypeSecurityLighting.htm),
[IfcLightFixtureTypeEnum](https://www.nibs.org/files/docs/IFC4x2_Bridge/schema/ifcelectricaldomain/lexical/ifclightfixturetypeenum.htm),
[Pset_SpaceFireSafetyRequirements](https://standards.buildingsmart.org/IFC/RELEASE/IFC4_3/HTML/lexical/Pset_SpaceFireSafetyRequirements.htm),
[FireBIM modelling guideline](https://itea4.org/project/workpackage/deliverable/document/download/1121/firebim-modelling-guideline-snapshot-2026-09-07.html),
[TU Wien escape-route IFC models](https://researchdata.tuwien.ac.at/records/tp76y-pxw11),
[DIALux Bridge for Revit](https://www.dialux.com/en-GB/dialux-pro/dialux-bridge-for-revit),
[DIALux and Revit workflow](https://www.dial.de/en-GB/news/dialux-evo-14-and-revit-a-seamless-workflow-for-lighting-design),
[DIALux forum: IFC and IES](https://community.dialux.com/get-support-with-dialux/post/does-the-exported-ifc-file-contain-ies-files-ASl3tAkW2iXK8HR),
[Revit photometric webs](https://help.autodesk.com/cloudhelp/2023/ENU/Revit-DocumentPresent/files/GUID-21089EF8-4DD4-4DCE-A7F7-C21F7B239E21.htm),
[IES TM-32-24](https://lightnowblog.com/2024/08/two-new-ies-standards-for-bim-and-lighting-maintenance),
[BIMCompNet](https://github.com/Yang-Mingsong/BIMCompNet).

## 7. Inspecting public models (luxpath inspect-ifc)

> Added 2026-10-16. Nine public IFC files were run through `luxpath inspect-ifc`. They live in the git-ignored
> `plan/samples/` and come from [youshengCode/IfcSampleFiles](https://github.com/youshengCode/IfcSampleFiles),
> [andrewisen/bim-whale-ifc-samples](https://github.com/andrewisen/bim-whale-ifc-samples) (MIT) and
> [buildingSMART/Sample-Test-Files](https://github.com/buildingSMART/Sample-Test-Files).

| Model | Exporter | Luminaires (types) | Emergency | Product code | Photometry | Spaces (FireExit) | Doors (FireExit) | Stairs | Fire equip. |
|---|---|---|---|---|---|---|---|---|---|
| Revit MEP (IFC4) | Revit 2021 | 410 (5) | 0 | 0 | none | 163 (0) | 124 (0) | 8 | 6 |
| Revit ARC (IFC4) | Revit 2021 | 10 (7) | 0 | 0 (manufacturer only) | none | 0 | 16 (0) | 3 | 0 |
| Revit ARC + fire ratings (IFC4) | Revit 2021 | 10 (7) | 0 | 0 | none | 0 | 16 (0) | 3 | 0 |
| Duplex MEP (IFC2x3) | Revit MEP 2011 | 28 (3) | 0 | 0 | none* | 42 (0) | 0 | 0 | 0 |
| Duplex Electrical (IFC2x3) | Revit 2013 | 28 (3) | 0 | 0 | none* | 1 (0) | 0 | 0 | 0 |
| AdvancedProject (IFC2x3) | Revit 2021 | 0 | – | – | – | 8 (0) | 126 (0) | 9 | 0 |
| LargeBuilding (IFC2x3) | Revit 2021 | 0 | – | – | – | 8 (0) | 18 (0) | 0 | 0 |
| SampleHouse (IFC4) | Xbim | 0 | – | – | – | 4 (0) | 3 (0) | 0 | 0 |
| Building-Architecture (IFC4) | SketchUp IFC manager | 0 | – | – | – | 2 (0) | 0 | 0 | 0 |

\* Revit's own "Photometrics" property set is exported, but it describes a generic `Point+HemiSpherical`
source with intensity 0: a rendering placeholder, not photometric data.

**What this means:**
- **Photometry never survives export**, as predicted (§6). It has to come from a mapping.
- **Product codes are absent** in every Revit export. But luminaires collapse into **few types**: 410 → 5
  in the MEP model. So the practical mapping key is the **type name** (e.g. `600x600 - 277`), and a project
  needs a handful of photometric files, not hundreds.
- **Nothing is marked as emergency lighting** and **nothing is flagged FireExit**, in any sample. Escape routes,
  exits and emergency luminaires would have to be identified another way: by a mapping, by names, or through a
  model requirement the architect fills in (an IDS file luxpath could provide).
- **Spaces, doors and stairs are usually there**, which is what derived routes and open areas need.

Caveats: these are demonstration models, not production projects, and most come from Revit. A real project
from a practising office (kept private in `plan/`) is the next evidence needed.
