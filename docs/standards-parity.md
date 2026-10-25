# EN 1838:2024 coverage

> **Last updated:** 2026-10-25 · **Change:** clause numbers from the 2024 edition's table of contents; rows for
> specific areas (5.4), safety signs (6), local area and standby lighting (7)

The requirements luxpath checks, one row each. **Every value below is "unverified"**: it comes from manufacturer
guides and summaries (see `docs/research.md`), not from the standard's text (gap 4). Values are facts restated in
our own words; never paste the standard's text here (`AGENTS.md` §1.1).

**Clause numbers** come from the 2024 edition's table of contents, read in the free preview SIST and iTeh publish
(SIST EN 1838:2025, kept in the git-ignored `plan/standards/`). The preview ends inside clause 3 (terms and
definitions), so it confirms the structure and the list of changes, not a single requirement value. "Not in the
preview" means the table of contents does not show which clause holds the requirement.

Context: EN 1838:2024 replaced the 2013 edition. Every CEN member adopts the same text under its own number and
year (the European foreword names the countries, Italy and Slovenia among them): **UNI EN 1838:2025** in Italy (in
force 6 Feb 2025, [UNI store](https://store.uni.com/uni-en-1838-2025)), BS EN 1838:2024 in the UK (31 Dec 2024),
NEN-EN 1838:2025 in the Netherlands, SIST EN 1838:2025 in Slovenia. National legal deviations ("A-deviations") are
listed in Annex D, which the preview does not reach. It is read with **EN 50172:2024** (system design, duration,
maintenance). National rules may add to it, for example longer durations.

The changes from 2013 the foreword lists: escape-route requirements now cover the route's **full width**; more
detail for **points of emphasis** and **specific areas of hazard**; considerations for lockdowns and long periods
without power, as an informative annex.

## Requirements

| Requirement | Value (unverified) | Clause | Parity id | Status |
|---|---|---|---|---|
| Escape routes: minimum horizontal illuminance on the floor **over the full width** (new in 2024; 2013 used the centre line and a central band) | ≥ 1 lx | 5.1 | `en1838.escape-route-full-width` | planned |
| Escape routes: uniformity | max/min ≤ 40:1 | 5.1 | `en1838.escape-route-uniformity` | planned |
| Escape routes through open areas: new procedure in 2024 | to read | 5.1 or 5.2, not in the preview | `en1838.escape-route-full-width` | planned |
| Open (anti-panic) areas: minimum horizontal illuminance on the floor, excluding a border | ≥ 0.5 lx, border 0.5 m | 5.2 | `en1838.open-area` | planned |
| Open areas: which areas need it (2013 summaries say ≥ 60 m², or by risk assessment) | to read | 5.2 | `en1838.open-area` | planned |
| Open areas: uniformity | max/min ≤ 40:1 | 5.2 | `en1838.open-area` | planned |
| High-risk task areas: maintained illuminance relative to normal lighting | ≥ 10 % of normal and ≥ 15 lx (2013 summaries) | 5.3 | `en1838.high-risk` | planned |
| High-risk task areas: uniformity | ≥ 0.1 (2013 summaries) | 5.3 | `en1838.high-risk` | planned |
| Points of emphasis and safety equipment (exits, changes of direction and level, fire-fighting, first aid, call points): illuminance near them; more detail in 2024 | to read | 4.2 | `en1838.points-of-emphasis` | planned |
| Toilets and changing rooms (new in 2024) | to read | 5.4.1 | `en1838.specific-areas` | planned |
| Public indoor swimming pools (new in 2024) | to read | 5.4.2 | `en1838.specific-areas` | planned |
| Motor generator, control, switch and plant rooms (new in 2024) | to read | 5.4.3 | `en1838.specific-areas` | planned |
| Local area lighting (new in 2024) | to read | 7.1 (Annex C, informative) | `en1838.local-area` | planned |
| Standby lighting: lighting that lets normal activities continue | to read | 7.2 | — | out of scope (not escape lighting) |
| Adaptive emergency escape lighting (in the scope; techniques in CEN/TS 17951) | to read | 1 | — | not yet planned |
| Disability glare: maximum luminous intensity by mounting height, in the field of view | table by mounting height | not in the preview | `en1838.glare-limits` | planned |
| Colour rendering of emergency light | Ra ≥ 40 | not in the preview | `en1838.declarations` | planned |
| Response time: share of required illuminance within a time limit | 50 % within 5 s, 100 % within 60 s | Annex A (informative); the normative clause is not in the preview | `en1838.declarations` | planned |
| Duration | minimum 1 h (often longer by national rules) | Annex A (informative); the normative clause is not in the preview | `en1838.declarations` | planned |
| Safety signs: conformity, illumination, viewing distance and mounting height | to read | 6.1–6.4 | `en1838.safety-signs` | planned |
| Annex B: on-site illuminance and luminance measurement (methods A–D, initial and five-year verification) | informative | Annex B | — | later (comparing measurements with calculations) |
| Lockdown and prolonged power loss | informative | Annex C per the foreword; the table of contents calls Annex C "Local area lighting": to read | — | out of scope |
| National deviations (A-deviations) | to read, for Italy first | Annex D | — | to read |

Declared values (duration, response time, colour rendering) can't be calculated from photometry: luxpath records
them as inputs, shows them in the report, and checks them against the requirement.
