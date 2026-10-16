//! The scene model: what is being lit, and with what.
//!
//! Planned (M3): rooms as floor polygons with a height, **escape routes** as centre polylines
//! with a width, **open (anti-panic) areas**, **high-risk task areas**, **points of emphasis**
//! (exits, changes of direction and level, fire-fighting and first-aid equipment, call points),
//! and placed luminaires referring to photometric files. Scenes are stored as human-readable,
//! diffable TOML, so a design can live in version control and be checked in CI.
//!
//! Core crates never touch the file system: they parse from `&str`/`&[u8]` so they also run in
//! the browser (WASM). Reading files is the apps' job.
//!
//! Layer L2: depends on `luxpath-geom` and `luxpath-photometry`.
