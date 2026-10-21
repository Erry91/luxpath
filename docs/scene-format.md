# Scene format, version 1

> **Last updated:** 2026-10-21 · **Change:** version 1 (M3)

A scene describes what is lit and with what: a TOML file a person can read, review and diff. `luxpath check
scene.toml` evaluates it. Examples: `examples/*.toml`.

Conventions: metres, degrees, lux, lumens; plan coordinates `[x, y]`, positions `[x, y, z]` with **z up** and the
floor usually at `z = 0`. Unknown fields are errors (a typo must not silently drop a value), and every error names
its line and column.

## Header and project

```toml
format = "luxpath-scene"      # required
version = 1                   # required: the format version; newer versions are refused, not guessed

[project]
name = "Office floor 2"
maintenance_factor = 0.8      # required, above 0 and at most 1: no default
```

## Calculation settings (optional)

```toml
[calculation]
spacing_m = 0.25              # grid spacing over areas and along routes (default 0.25)
route_points_across = 5       # points across a route's full width, edges included (default 5)
open_area_border_m = 0.5      # border left out of open areas (default 0.5)
```

The defaults are luxpath's choices, shown in every report. From M4 the checks will use the spacings and borders
EN 1838:2024 sets, verified against its text (docs/standards-parity.md).

## Photometry

```toml
[[photometry]]
id = "bulkhead"
file = "photometry/bulkhead.ldt"          # IES or EULUMDAT, relative to the scene file
emergency = { ballast_lumen_factor = 0.4 } # or { flux_lm = 120 }
```

`emergency` is the output in emergency mode: a ballast lumen factor in (0, 1], or lumens up to the rated flux
(measured like the file's flux: the lamps' for relative photometry, the luminaire's for absolute). Photometric
files do not hold it. **Without it, every result is "cannot verify"**: luxpath does not assume a value.

## Luminaires

```toml
[[luminaire]]
id = "L1"
photometry = "bulkhead"       # a [[photometry]] id
position = [2.0, 1.0, 2.7]    # the photometric centre
rotation_deg = 90             # optional: heading of C0, counter-clockwise from +x (default 0)
tilt_deg = 60                 # optional: tilt of the nadir towards C0 (default 0; 90 = aimed horizontally)
```

## Escape routes

```toml
[[route]]
id = "R1"
name = "Corridor to exit A"   # optional
path = [[0, 1], [15, 1], [15, 12]]   # the centre line, at least two points
width_m = 2.0
floor_m = 0.0                 # optional (default 0)
```

Illuminance is computed on the floor over the route's **full width** (`route_points_across` points across).

## Areas

```toml
[[area]]
id = "A1"
kind = "open"                 # "open" (anti-panic) or "high-risk" (high-risk task area)
outline = [[0, 0], [15, 0], [15, 10], [0, 10]]   # a polygon, at least three corners
floor_m = 0.0                 # optional
normal_lighting_lx = 300      # high-risk areas only: the normal lighting's maintained illuminance
```

Open areas are computed less `open_area_border_m` from their edges; high-risk areas over their whole outline.

## Points of emphasis

```toml
[[point]]
id = "F1"
name = "Fire extinguisher"    # optional
kind = "fire-fighting"        # exit, direction-change, level-change, intersection, fire-fighting,
                              # first-aid, call-point, other
position = [10.0, 0.05, 1.2]
facing = [0, 1, 0]            # optional: the surface's outward direction, for vertical illuminance
```

Illuminance is computed on a horizontal surface at `position` and, with `facing`, on the surface facing that way.

## Ids

Every `id` is unique across the whole scene and not empty: reports and later versions refer to elements by it.

## Compatibility

A scene states its `version`. A luxpath that reads version N refuses other versions with a clear message; when the
format changes, the version goes up and `docs/scene-format.md` says how to migrate.
