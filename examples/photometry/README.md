# Example photometry

Synthetic files for the example scenes, generated for luxpath from the formulas below. They describe no real
product, and they are under the repository's licence (MIT OR Apache-2.0). Real manufacturers' files are never
committed (AGENTS.md §1.3).

| File | Format | Distribution (γ from the nadir, in 5° steps) | Lamp flux | Light output ratio |
|---|---|---|---|---|
| `bulkhead.ldt` | EULUMDAT, rotationally symmetric | I ∝ (1 + 1.5 sin²γ) cos γ below the horizon (a batwing), none above | 250 lm | 80 % |
| `corridor.ies` | IES LM-63-2002, quadrant symmetric (C every 15°) | I ∝ cos γ (1 + 2 sin²γ cos²C) below the horizon: wider along C0 | 300 lm | 85 % |
