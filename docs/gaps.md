# Gaps: what luxpath cannot do yet

> **Last updated:** 2026-10-21 · **Change:** M1 closed rows 1–2; added rows 12–15

Every known shortfall, ranked by user impact. This is the work list: pick from the top, and remove or update a row
in the same PR that closes it. **[alpha blocker]** rows fail the alpha gate (ROADMAP.md).

| # | Gap | Why it ranks here | Milestone | Est. h |
|---|---|---|---|---|
| 3 | **[alpha blocker]** No grids or multi-luminaire sums | Requirements are about areas and routes, not points | M2 | 8–12 |
| 4 | **[alpha blocker]** Requirement values not verified against EN 1838:2024 text | Values in `standards-parity.md` come from secondary sources | M4 | 3 + buying the standard |
| 5 | **[alpha blocker]** No scene format or `check` command | Users can't describe a floor | M3 | 15–20 |
| 6 | **[alpha blocker]** No EN 1838 checks | The project's purpose | M4 | 25–40 |
| 7 | **[alpha blocker]** No independent validation | Trust: nobody should rely on unvalidated numbers | M6 | 10–20 + human |
| 8 | Only downward-pointing luminaires | Wall-mounted luminaires and exit signs are common | M2 | 4–6 |
| 9 | No vertical illuminance | Needed for safety equipment checks | M2 | 2–4 |
| 10 | No reports beyond the terminal | Designers hand over documents | M5 | 15–25 |
| 11 | No CI on a hosted repository yet | The workflow exists but the repository isn't published | — | 1 |
| 12 | **[alpha blocker]** Readers tried on only two real files | Synthetic files test the formats as we read them; manufacturers' files (kept in `plan/`, never committed) test them as written. Wanted: a few dozen IES and LDT files, including emergency luminaires | M6 | 2–4 + files from the owner |
| 13 | No plausibility warnings on photometry | A relative file emitting more than its lamps (the LM-63 example file integrates to 211 %), or a declared LDT light output ratio far from the table's, should be reported, not silently used | M2 | 2–3 |
| 14 | Format details from our reading, not the specification texts | The LM-63 factor that changed meaning between editions, and the C0 orientation of each format, are unverified like the EN 1838 values (gap 4); LM-63-2019 is sold by the IES | M6 | 2 + buying LM-63-2019 |
| 15 | Refused file variants | IES type A and B photometry, `TILT=<file>`, and EULUMDAT files with several lamp sets are refused with a located error | M7 | 4–8 |