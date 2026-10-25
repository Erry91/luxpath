//! The scene model: what is being lit, and with what.
//!
//! A scene is a TOML file (format version 1, `docs/scene-format.md`): the project's maintenance
//! factor, the photometric files it uses (with each one's emergency output), placed luminaires,
//! **walls** (which block light; doorways are gaps between them),
//! **escape routes** (centre polylines with a width), **areas** (open/anti-panic and high-risk task
//! areas, as polygons), **points of emphasis** (exits, changes of direction and level, safety
//! equipment), and calculation settings. Scenes are human-readable and diffable, so a design can
//! live in version control and be checked in CI.
//!
//! [`Scene::parse`] checks everything it can without the photometric files: unknown fields,
//! missing values, references, geometry. Errors carry the line and column they concern.
//!
//! Core crates never touch the file system: photometric files are named here and read by the apps
//! (paths are relative to the scene file). That keeps this crate usable in the browser (WASM).
//!
//! Layer L2: depends on `luxpath-geom` and `luxpath-photometry`.

use std::collections::HashSet;
use std::ops::Range;

use luxpath_geom::{Vec2, Vec3, Wall};
use luxpath_photometry::EmergencyOutput;
use serde::Deserialize;
use toml::Spanned;

/// The scene format version this crate reads.
pub const FORMAT_VERSION: u32 = 1;

/// Scene files larger than this are refused (real ones are a few kilobytes).
pub const MAX_SCENE_BYTES: usize = 4 << 20;

/// Why a scene could not be read, and where (1-based line and column; 0 when unknown).
#[derive(Debug, Clone, PartialEq)]
pub struct SceneError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.message)
        } else {
            write!(f, "line {}, column {}: {}", self.line, self.column, self.message)
        }
    }
}

impl std::error::Error for SceneError {}

/// The line and column (1-based) of a byte offset.
fn locate(text: &str, offset: usize) -> (usize, usize) {
    let before = text.get(..offset.min(text.len())).unwrap_or(text);
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (line, column)
}

// ------------------------------------------------------------------------------------------------
// The file, as written
// ------------------------------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    format: Spanned<String>,
    version: Spanned<u32>,
    project: Spanned<ProjectFile>,
    #[serde(default)]
    calculation: Option<Spanned<CalculationFile>>,
    #[serde(default)]
    photometry: Vec<Spanned<PhotometryFile>>,
    #[serde(default)]
    luminaire: Vec<Spanned<LuminaireFile>>,
    #[serde(default)]
    wall: Vec<Spanned<WallFile>>,
    #[serde(default)]
    route: Vec<Spanned<RouteFile>>,
    #[serde(default)]
    area: Vec<Spanned<AreaFile>>,
    #[serde(default)]
    point: Vec<Spanned<PointFile>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectFile {
    name: String,
    maintenance_factor: Spanned<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CalculationFile {
    spacing_m: Option<Spanned<f64>>,
    route_points_across: Option<Spanned<u32>>,
    open_area_border_m: Option<Spanned<f64>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PhotometryFile {
    id: Spanned<String>,
    file: String,
    emergency: Option<Spanned<EmergencyFile>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmergencyFile {
    ballast_lumen_factor: Option<f64>,
    flux_lm: Option<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LuminaireFile {
    id: Spanned<String>,
    photometry: Spanned<String>,
    position: Spanned<[f64; 3]>,
    #[serde(default)]
    rotation_deg: f64,
    #[serde(default)]
    tilt_deg: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WallFile {
    id: Spanned<String>,
    path: Spanned<Vec<[f64; 2]>>,
    height_m: Spanned<f64>,
    #[serde(default)]
    floor_m: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteFile {
    id: Spanned<String>,
    #[serde(default)]
    name: String,
    path: Option<Spanned<Vec<[f64; 2]>>>,
    width_m: Option<Spanned<f64>>,
    outline: Option<Spanned<Vec<[f64; 2]>>>,
    #[serde(default)]
    floor_m: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AreaFile {
    id: Spanned<String>,
    #[serde(default)]
    name: String,
    kind: Spanned<String>,
    outline: Spanned<Vec<[f64; 2]>>,
    #[serde(default)]
    floor_m: f64,
    normal_lighting_lx: Option<Spanned<f64>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointFile {
    id: Spanned<String>,
    #[serde(default)]
    name: String,
    kind: Spanned<String>,
    position: Spanned<[f64; 3]>,
    facing: Option<Spanned<[f64; 3]>>,
}

// ------------------------------------------------------------------------------------------------
// The model
// ------------------------------------------------------------------------------------------------

/// A photometric file the scene uses.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotometryRef {
    pub id: String,
    /// Relative to the scene file.
    pub file: String,
    /// `None`: not declared, so emergency-mode results cannot be verified.
    pub emergency: Option<EmergencyOutput>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlacedLuminaire {
    pub id: String,
    /// Index into [`Scene::photometry`].
    pub photometry: usize,
    pub position: Vec3,
    pub rotation_deg: f64,
    pub tilt_deg: f64,
}

/// Where an escape route runs.
#[derive(Debug, Clone, PartialEq)]
pub enum RouteShape {
    /// A centre line and the route's full width.
    Centre { path: Vec<Vec2>, width_m: f64 },
    /// The route's floor as a polygon (a corridor as a BIM model draws it), computed over all of it.
    Outline(Vec<Vec2>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub id: String,
    pub name: String,
    pub shape: RouteShape,
    /// The floor's height, in metres.
    pub floor_m: f64,
}

/// What an area is, for EN 1838:2024: each kind has its own requirements (`docs/standards-parity.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AreaKind {
    /// An open (anti-panic) area (5.2).
    Open,
    /// A high-risk task area (5.3).
    HighRisk,
    /// Toilets and changing rooms (5.4.1).
    Toilets,
    /// A public indoor swimming pool (5.4.2).
    SwimmingPool,
    /// A motor generator, control, switch or plant room (5.4.3).
    PlantRoom,
    /// An area lit by local area lighting (7.1).
    LocalArea,
}

impl AreaKind {
    const NAMES: [(&'static str, AreaKind); 6] = [
        ("open", AreaKind::Open),
        ("high-risk", AreaKind::HighRisk),
        ("toilets", AreaKind::Toilets),
        ("swimming-pool", AreaKind::SwimmingPool),
        ("plant-room", AreaKind::PlantRoom),
        ("local-area", AreaKind::LocalArea),
    ];

    /// The name scenes use for this kind.
    pub fn name(self) -> &'static str {
        Self::NAMES.iter().find(|(_, k)| *k == self).map_or("open", |(n, _)| n)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Area {
    pub id: String,
    pub name: String,
    pub kind: AreaKind,
    pub outline: Vec<Vec2>,
    pub floor_m: f64,
    /// For high-risk task areas: the maintained illuminance of the normal lighting, in lux.
    pub normal_lighting_lx: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointKind {
    Exit,
    DirectionChange,
    LevelChange,
    Intersection,
    FireFighting,
    FirstAid,
    CallPoint,
    Other,
}

impl PointKind {
    const NAMES: [(&'static str, PointKind); 8] = [
        ("exit", PointKind::Exit),
        ("direction-change", PointKind::DirectionChange),
        ("level-change", PointKind::LevelChange),
        ("intersection", PointKind::Intersection),
        ("fire-fighting", PointKind::FireFighting),
        ("first-aid", PointKind::FirstAid),
        ("call-point", PointKind::CallPoint),
        ("other", PointKind::Other),
    ];

    pub fn name(self) -> &'static str {
        Self::NAMES.iter().find(|(_, k)| *k == self).map_or("other", |(n, _)| n)
    }
}

/// A point of emphasis: illuminance is computed on the floor below it (horizontal) and, when it
/// has a facing direction, on that surface (vertical, at safety equipment).
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub id: String,
    pub name: String,
    pub kind: PointKind,
    pub position: Vec3,
    pub facing: Option<Vec3>,
}

/// Calculation settings. Defaults are luxpath's, not the standard's; reports show them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calculation {
    /// Grid spacing over areas and along routes, in metres.
    pub spacing_m: f64,
    /// Points across a route's width, edges included.
    pub route_points_across: usize,
    /// Border left out of open areas, in metres.
    pub open_area_border_m: f64,
}

impl Default for Calculation {
    fn default() -> Self {
        Self { spacing_m: 0.25, route_points_across: 5, open_area_border_m: 0.5 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub name: String,
    pub maintenance_factor: f64,
    pub calculation: Calculation,
    pub photometry: Vec<PhotometryRef>,
    pub luminaires: Vec<PlacedLuminaire>,
    /// Every straight piece of every `[[wall]]`.
    pub walls: Vec<Wall>,
    pub routes: Vec<Route>,
    pub areas: Vec<Area>,
    pub points: Vec<Point>,
}

/// Errors located by span while the file is turned into a [`Scene`].
struct Checker<'t> {
    text: &'t str,
}

impl Checker<'_> {
    fn err(&self, span: Range<usize>, message: impl Into<String>) -> SceneError {
        let (line, column) = locate(self.text, span.start);
        SceneError { line, column, message: message.into() }
    }

    fn finite(&self, value: &Spanned<f64>, what: &str) -> Result<f64, SceneError> {
        let v = *value.get_ref();
        if v.is_finite() { Ok(v) } else { Err(self.err(value.span(), format!("{what} must be a finite number"))) }
    }

    fn positive(&self, value: &Spanned<f64>, what: &str) -> Result<f64, SceneError> {
        let v = self.finite(value, what)?;
        if v > 0.0 { Ok(v) } else { Err(self.err(value.span(), format!("{what} must be above 0"))) }
    }

    fn point2(&self, points: &Spanned<Vec<[f64; 2]>>, min: usize, what: &str) -> Result<Vec<Vec2>, SceneError> {
        let out: Vec<Vec2> = points.get_ref().iter().map(|[x, y]| Vec2::new(*x, *y)).collect();
        if out.len() < min {
            return Err(self.err(points.span(), format!("{what} needs at least {min} points")));
        }
        if !out.iter().all(|p| p.is_finite()) {
            return Err(self.err(points.span(), format!("{what} has a coordinate that is not a finite number")));
        }
        Ok(out)
    }

    fn point3(&self, p: &Spanned<[f64; 3]>, what: &str) -> Result<Vec3, SceneError> {
        let [x, y, z] = *p.get_ref();
        let v = Vec3::new(x, y, z);
        if v.is_finite() { Ok(v) } else { Err(self.err(p.span(), format!("{what} must be finite numbers"))) }
    }

    fn unique<'a>(&self, seen: &mut HashSet<&'a str>, id: &'a Spanned<String>) -> Result<(), SceneError> {
        if id.get_ref().trim().is_empty() {
            return Err(self.err(id.span(), "an id must not be empty"));
        }
        if !seen.insert(id.get_ref().as_str()) {
            return Err(self.err(id.span(), format!("the id `{}` is used twice", id.get_ref())));
        }
        Ok(())
    }
}

impl Scene {
    /// Reads a scene from TOML text.
    pub fn parse(text: &str) -> Result<Self, SceneError> {
        if text.len() > MAX_SCENE_BYTES {
            return Err(SceneError {
                line: 0,
                column: 0,
                message: format!("the scene is {} bytes, more than the {MAX_SCENE_BYTES}-byte limit", text.len()),
            });
        }
        let file: File = toml::from_str(text).map_err(|e| {
            let (line, column) = e.span().map_or((0, 0), |s| locate(text, s.start));
            SceneError { line, column, message: e.message().trim().to_string() }
        })?;
        let c = Checker { text };
        if file.format.get_ref() != "luxpath-scene" {
            return Err(c.err(file.format.span(), "`format` must be \"luxpath-scene\""));
        }
        if *file.version.get_ref() != FORMAT_VERSION {
            return Err(c.err(
                file.version.span(),
                format!(
                    "this is scene format version {}; this luxpath reads version {FORMAT_VERSION}",
                    file.version.get_ref()
                ),
            ));
        }

        let project = file.project.get_ref();
        let mf = c.finite(&project.maintenance_factor, "the maintenance factor")?;
        if !(mf > 0.0 && mf <= 1.0) {
            return Err(
                c.err(project.maintenance_factor.span(), "the maintenance factor must be above 0 and at most 1")
            );
        }

        let mut calculation = Calculation::default();
        if let Some(calc) = &file.calculation {
            let calc = calc.get_ref();
            if let Some(s) = &calc.spacing_m {
                calculation.spacing_m = c.positive(s, "the grid spacing")?;
            }
            if let Some(n) = &calc.route_points_across {
                if *n.get_ref() == 0 {
                    return Err(c.err(n.span(), "a route needs at least one point across"));
                }
                calculation.route_points_across = *n.get_ref() as usize;
            }
            if let Some(b) = &calc.open_area_border_m {
                let border = c.finite(b, "the open-area border")?;
                if border < 0.0 {
                    return Err(c.err(b.span(), "the open-area border must not be negative"));
                }
                calculation.open_area_border_m = border;
            }
        }

        let mut ids = HashSet::new();
        let mut photometry = Vec::new();
        for p in &file.photometry {
            let p = p.get_ref();
            c.unique(&mut ids, &p.id)?;
            let emergency = match &p.emergency {
                None => None,
                Some(e) => Some(match (e.get_ref().ballast_lumen_factor, e.get_ref().flux_lm) {
                    (Some(f), None) => EmergencyOutput::BallastLumenFactor(f),
                    (None, Some(lm)) => EmergencyOutput::FluxLm(lm),
                    _ => {
                        return Err(
                            c.err(e.span(), "give the emergency output as either `ballast_lumen_factor` or `flux_lm`")
                        );
                    }
                }),
            };
            photometry.push(PhotometryRef { id: p.id.get_ref().clone(), file: p.file.clone(), emergency });
        }

        let mut luminaires = Vec::new();
        for l in &file.luminaire {
            let l = l.get_ref();
            c.unique(&mut ids, &l.id)?;
            let index = photometry.iter().position(|p| &p.id == l.photometry.get_ref()).ok_or_else(|| {
                c.err(l.photometry.span(), format!("no [[photometry]] has the id `{}`", l.photometry.get_ref()))
            })?;
            if !l.rotation_deg.is_finite() || !l.tilt_deg.is_finite() {
                return Err(c.err(l.id.span(), "the rotation and tilt must be finite numbers"));
            }
            luminaires.push(PlacedLuminaire {
                id: l.id.get_ref().clone(),
                photometry: index,
                position: c.point3(&l.position, "the position")?,
                rotation_deg: l.rotation_deg,
                tilt_deg: l.tilt_deg,
            });
        }

        let mut walls = Vec::new();
        for w in &file.wall {
            let w = w.get_ref();
            c.unique(&mut ids, &w.id)?;
            if !w.floor_m.is_finite() {
                return Err(c.err(w.id.span(), "the floor height must be a finite number"));
            }
            let height = c.positive(&w.height_m, "the wall height")?;
            let path = c.point2(&w.path, 2, "a wall's path")?;
            for pair in path.windows(2) {
                if let (Some(a), Some(b)) = (pair.first(), pair.get(1)) {
                    walls.push(Wall { a: *a, b: *b, bottom_m: w.floor_m, top_m: w.floor_m + height });
                }
            }
        }

        let mut routes = Vec::new();
        for r in &file.route {
            let r = r.get_ref();
            c.unique(&mut ids, &r.id)?;
            if !r.floor_m.is_finite() {
                return Err(c.err(r.id.span(), "the floor height must be a finite number"));
            }
            let shape = match (&r.path, &r.width_m, &r.outline) {
                (Some(path), Some(width), None) => RouteShape::Centre {
                    path: c.point2(path, 2, "a route's path")?,
                    width_m: c.positive(width, "the route width")?,
                },
                (None, None, Some(outline)) => RouteShape::Outline(c.point2(outline, 3, "a route's outline")?),
                _ => {
                    return Err(c.err(
                        r.id.span(),
                        "give a route either `path` and `width_m` (its centre line) or `outline` (its floor)",
                    ));
                }
            };
            routes.push(Route { id: r.id.get_ref().clone(), name: r.name.clone(), shape, floor_m: r.floor_m });
        }

        let mut areas = Vec::new();
        for a in &file.area {
            let a = a.get_ref();
            c.unique(&mut ids, &a.id)?;
            let kind =
                AreaKind::NAMES.iter().find(|(n, _)| *n == a.kind.get_ref()).map(|(_, k)| *k).ok_or_else(|| {
                    let names: Vec<&str> = AreaKind::NAMES.iter().map(|(n, _)| *n).collect();
                    c.err(a.kind.span(), format!("`{}` is not an area kind ({})", a.kind.get_ref(), names.join(", ")))
                })?;
            let normal_lighting_lx = match &a.normal_lighting_lx {
                Some(lx) if kind == AreaKind::HighRisk => Some(c.positive(lx, "the normal lighting's illuminance")?),
                Some(lx) => return Err(c.err(lx.span(), "`normal_lighting_lx` belongs to high-risk areas only")),
                None => None,
            };
            if !a.floor_m.is_finite() {
                return Err(c.err(a.id.span(), "the floor height must be a finite number"));
            }
            areas.push(Area {
                id: a.id.get_ref().clone(),
                name: a.name.clone(),
                kind,
                outline: c.point2(&a.outline, 3, "an area's outline")?,
                floor_m: a.floor_m,
                normal_lighting_lx,
            });
        }

        let mut points = Vec::new();
        for p in &file.point {
            let p = p.get_ref();
            c.unique(&mut ids, &p.id)?;
            let kind =
                PointKind::NAMES.iter().find(|(n, _)| *n == p.kind.get_ref()).map(|(_, k)| *k).ok_or_else(|| {
                    let names: Vec<&str> = PointKind::NAMES.iter().map(|(n, _)| *n).collect();
                    c.err(p.kind.span(), format!("`{}` is not a point kind ({})", p.kind.get_ref(), names.join(", ")))
                })?;
            let facing = match &p.facing {
                None => None,
                Some(f) => Some(
                    c.point3(f, "the facing direction")?
                        .normalized()
                        .ok_or_else(|| c.err(f.span(), "the facing direction must not be zero"))?,
                ),
            };
            points.push(Point {
                id: p.id.get_ref().clone(),
                name: p.name.clone(),
                kind,
                position: c.point3(&p.position, "the position")?,
                facing,
            });
        }

        Ok(Self {
            name: project.name.clone(),
            maintenance_factor: mf,
            calculation,
            photometry,
            luminaires,
            walls,
            routes,
            areas,
            points,
        })
    }
}

#[cfg(test)]
mod tests;
