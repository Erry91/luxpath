# Gaps: what luxpath cannot do yet

> **Last updated:** 2026-10-16 · **Change:** initial list

Every known shortfall, ranked by user impact. This is the work list: pick from the top, and remove or update a row
in the same PR that closes it. **[alpha blocker]** rows fail the alpha gate (ROADMAP.md).

| # | Gap | Why it ranks here | Milestone | Est. h |
|---|---|---|---|---|
| 1 | **[alpha blocker]** No photometric file readers (IES, EULUMDAT) | Nothing real can be calculated without them | M1 | 15–25 |
| 2 | **[alpha blocker]** No symmetry expansion | Most files store a half or a quarter of the distribution | M1 | 3–5 |
| 3 | **[alpha blocker]** No grids or multi-luminaire sums | Requirements are about areas and routes, not points | M2 | 8–12 |
| 4 | **[alpha blocker]** Requirement values not verified against EN 1838:2024 text | Values in `standards-parity.md` come from secondary sources | M4 | 3 + buying the standard |
| 5 | **[alpha blocker]** No scene format or `check` command | Users can't describe a floor | M3 | 15–20 |
| 6 | **[alpha blocker]** No EN 1838 checks | The project's purpose | M4 | 25–40 |
| 7 | **[alpha blocker]** No independent validation | Trust: nobody should rely on unvalidated numbers | M6 | 10–20 + human |
| 8 | Only downward-pointing luminaires | Wall-mounted luminaires and exit signs are common | M2 | 4–6 |
| 9 | No vertical illuminance | Needed for safety equipment checks | M2 | 2–4 |
| 10 | No reports beyond the terminal | Designers hand over documents | M5 | 15–25 |
| 11 | No CI on a hosted repository yet | The workflow exists but the repository isn't published | — | 1 |