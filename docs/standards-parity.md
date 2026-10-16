# EN 1838:2024 coverage

> **Last updated:** 2026-10-16 · **Change:** initial, from secondary sources

The requirements luxpath checks, one row each. **Every value below is "unverified"**: it comes from manufacturer
guides and summaries (see `docs/research.md`), not from the standard's text. Clause numbers are left blank until
someone checks the purchased standard (gap 4). Values are facts restated in our own words; never paste the
standard's text here (`AGENTS.md` §1.1).

Context: EN 1838:2024 replaced the 2013 edition. National bodies publish the same text under their own number and
year: **UNI EN 1838:2025** in Italy (in force 6 Feb 2025, [UNI store](https://store.uni.com/uni-en-1838-2025)),
BS EN 1838:2024 in the UK (31 Dec 2024), NEN-EN 1838:2025 in the Netherlands. The requirements are identical. It is meant to be read with **EN 50172:2024** (system design, duration, maintenance). National rules may add
to it, for example longer durations.

## Requirements

| Requirement | Value (unverified) | Clause | Parity id | Status |
|---|---|---|---|---|
| Escape routes: minimum horizontal illuminance on the floor **over the full width** (new in 2024; 2013 used the centre line and a central band) | ≥ 1 lx | | `en1838.escape-route-full-width` | planned |
| Escape routes: uniformity | max/min ≤ 40:1 | | `en1838.escape-route-uniformity` | planned |
| Escape routes through open areas: new procedure in 2024 | to read | | `en1838.escape-route-full-width` | planned |
| Open (anti-panic) areas: minimum horizontal illuminance on the floor, excluding a border | ≥ 0.5 lx, border 0.5 m | | `en1838.open-area` | planned |
| Open areas: which areas need it (2013 summaries say ≥ 60 m², or by risk assessment) | to read | | `en1838.open-area` | planned |
| Open areas: uniformity | max/min ≤ 40:1 | | `en1838.open-area` | planned |
| High-risk task areas: maintained illuminance relative to normal lighting | ≥ 10 % of normal and ≥ 15 lx (2013 summaries) | | `en1838.high-risk` | planned |
| High-risk task areas: uniformity | ≥ 0.1 (2013 summaries) | | `en1838.high-risk` | planned |
| Points of emphasis and safety equipment (exits, changes of direction and level, fire-fighting, first aid, call points): illuminance near them; more detail in 2024 | to read | | `en1838.points-of-emphasis` | planned |
| Local area lighting (new in 2024) | to read | | — | not yet planned |
| Adaptive escape-route emergency lighting (in scope per UNI's summary) | to read | | — | not yet planned |
| Disability glare: maximum luminous intensity by mounting height, in the field of view | table by mounting height | | `en1838.glare-limits` | planned |
| Colour rendering of emergency light | Ra ≥ 40 | | `en1838.declarations` | planned |
| Response time: share of required illuminance within a time limit | 50 % within 5 s, 100 % within 60 s | | `en1838.declarations` | planned |
| Duration | minimum 1 h (often longer by national rules) | | `en1838.declarations` | planned |
| Annex C: lockdown and prolonged power loss (new in 2024) | informative | | — | out of scope |
| Safety signs (luminance, contrast, viewing distance) | to read | | — | later |

Declared values (duration, response time, colour rendering) can't be calculated from photometry: luxpath records
them as inputs, shows them in the report, and checks them against the requirement.
