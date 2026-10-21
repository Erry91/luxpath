//! Calculation grids: points where illuminance is computed, all on surfaces with one normal.

use luxpath_geom::{Vec2, Vec3, distance_to_boundary, point_in_polygon};

/// Grids larger than this are refused (a 100 m × 100 m floor at 0.1 m is a million points).
pub const MAX_GRID_POINTS: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    points: Vec<Vec3>,
    normal: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GridError {
    /// A spacing, width, border or height is not a finite number in range.
    BadDimension(&'static str),
    /// The polygon has fewer than three vertices or a non-finite one; the route fewer than two.
    BadOutline,
    /// More than [`MAX_GRID_POINTS`].
    TooManyPoints,
    /// No point is left (the area is smaller than its border, or the spacing): nothing to check.
    Empty,
}

impl std::fmt::Display for GridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadDimension(what) => write!(f, "the grid's {what} is not a usable number"),
            Self::BadOutline => write!(f, "the outline needs at least three finite corners (two for a route)"),
            Self::TooManyPoints => write!(f, "the grid would have more than {MAX_GRID_POINTS} points"),
            Self::Empty => write!(f, "the grid has no points: the area is smaller than its border or spacing"),
        }
    }
}

impl std::error::Error for GridError {}

fn positive(value: f64, what: &'static str) -> Result<f64, GridError> {
    if value.is_finite() && value > 0.0 { Ok(value) } else { Err(GridError::BadDimension(what)) }
}

impl Grid {
    /// Points given one by one, on surfaces facing `normal` (vertical illuminance at a fire
    /// extinguisher: the point on the equipment, the normal towards the room).
    pub fn new(points: Vec<Vec3>, normal: Vec3) -> Result<Self, GridError> {
        if points.len() > MAX_GRID_POINTS {
            return Err(GridError::TooManyPoints);
        }
        if points.is_empty() {
            return Err(GridError::Empty);
        }
        if !points.iter().all(|p| p.is_finite()) || normal.normalized().is_none() {
            return Err(GridError::BadDimension("point or normal"));
        }
        Ok(Self { points, normal })
    }

    /// A horizontal grid facing up over the `polygon` at `height_m`: the centres of `spacing_m`
    /// cells across its bounding box that lie inside it and at least `border_m` from its edges.
    pub fn area(polygon: &[Vec2], height_m: f64, spacing_m: f64, border_m: f64) -> Result<Self, GridError> {
        let spacing = positive(spacing_m, "spacing")?;
        if !height_m.is_finite() {
            return Err(GridError::BadDimension("height"));
        }
        if !border_m.is_finite() || border_m < 0.0 {
            return Err(GridError::BadDimension("border"));
        }
        if polygon.len() < 3 || !polygon.iter().all(|p| p.is_finite()) {
            return Err(GridError::BadOutline);
        }
        let (min, max) = polygon.iter().fold(
            (Vec2::new(f64::INFINITY, f64::INFINITY), Vec2::new(f64::NEG_INFINITY, f64::NEG_INFINITY)),
            |(lo, hi), p| (Vec2::new(lo.x.min(p.x), lo.y.min(p.y)), Vec2::new(hi.x.max(p.x), hi.y.max(p.y))),
        );
        let (nx, ny) = (((max.x - min.x) / spacing).ceil(), ((max.y - min.y) / spacing).ceil());
        if !(nx * ny).is_finite() || nx * ny > MAX_GRID_POINTS as f64 {
            return Err(GridError::TooManyPoints);
        }
        // Finite and at most MAX_GRID_POINTS: the casts are exact.
        let (nx, ny) = (nx as usize, ny as usize);
        let mut points = Vec::new();
        for j in 0..ny {
            for i in 0..nx {
                let p = Vec2::new(min.x + (i as f64 + 0.5) * spacing, min.y + (j as f64 + 0.5) * spacing);
                if point_in_polygon(p, polygon) && distance_to_boundary(p, polygon) >= border_m {
                    points.push(p.at(height_m));
                }
            }
        }
        Self::new(points, Vec3::UP)
    }

    /// A horizontal grid facing up along a route: its centre line `centre` at `height_m`, sampled
    /// every `spacing_m` along each segment (both ends included), with `across` points evenly over
    /// the full `width_m` (edges included; one point is the centre line).
    pub fn route(
        centre: &[Vec2],
        width_m: f64,
        height_m: f64,
        spacing_m: f64,
        across: usize,
    ) -> Result<Self, GridError> {
        let spacing = positive(spacing_m, "spacing")?;
        let width = positive(width_m, "width")?;
        if !height_m.is_finite() {
            return Err(GridError::BadDimension("height"));
        }
        if across == 0 || across > MAX_GRID_POINTS {
            return Err(GridError::BadDimension("number of points across"));
        }
        if centre.len() < 2 || !centre.iter().all(|p| p.is_finite()) {
            return Err(GridError::BadOutline);
        }
        let offsets: Vec<f64> = if across == 1 {
            vec![0.0]
        } else {
            (0..across).map(|k| -width / 2.0 + width * k as f64 / (across - 1) as f64).collect()
        };
        let mut points = Vec::new();
        for (index, pair) in centre.windows(2).enumerate() {
            let (Some(a), Some(b)) = (pair.first(), pair.get(1)) else { continue };
            let along = *b - *a;
            let length = along.length();
            if length == 0.0 {
                continue;
            }
            let (ux, uy) = (along.x / length, along.y / length);
            let steps = (length / spacing).ceil();
            if !steps.is_finite() || steps * across as f64 + points.len() as f64 > MAX_GRID_POINTS as f64 {
                return Err(GridError::TooManyPoints);
            }
            // Finite and bounded by MAX_GRID_POINTS: the cast is exact.
            let steps = steps as usize;
            // Each segment's start is the previous segment's end: include it only once.
            let first = if index == 0 { 0 } else { 1 };
            for s in first..=steps {
                let t = length * s as f64 / steps as f64;
                for offset in &offsets {
                    // The perpendicular to the left of the direction of travel is (−uy, ux).
                    let p = Vec2::new(a.x + ux * t - uy * offset, a.y + uy * t + ux * offset);
                    points.push(p.at(height_m));
                }
            }
        }
        Self::new(points, Vec3::UP)
    }

    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    pub fn normal(&self) -> Vec3 {
        self.normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(side: f64) -> Vec<Vec2> {
        vec![Vec2::new(0.0, 0.0), Vec2::new(side, 0.0), Vec2::new(side, side), Vec2::new(0.0, side)]
    }

    #[test]
    fn an_area_grid_takes_cell_centres_inside_less_the_border() {
        // A 4 m square at 1 m spacing: 16 centres at 0.5, 1.5, 2.5, 3.5.
        let g = Grid::area(&square(4.0), 0.0, 1.0, 0.0).unwrap();
        assert_eq!(g.points().len(), 16);
        assert_eq!(g.points()[0], Vec3::new(0.5, 0.5, 0.0));
        // A 0.5 m border keeps every centre (they are exactly 0.5 m in); 0.6 m keeps the inner 4.
        assert_eq!(Grid::area(&square(4.0), 0.0, 1.0, 0.5).unwrap().points().len(), 16);
        assert_eq!(Grid::area(&square(4.0), 0.0, 1.0, 0.6).unwrap().points().len(), 4);
        assert_eq!(Grid::area(&square(1.0), 0.0, 1.0, 0.6), Err(GridError::Empty));
    }

    #[test]
    fn a_route_grid_covers_the_full_width_and_each_corner_once() {
        // An L-shaped route, 2 m then 2 m, 1 m wide, every 1 m, three points across.
        let route = [Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0), Vec2::new(2.0, 2.0)];
        let g = Grid::route(&route, 1.0, 0.0, 1.0, 3).unwrap();
        // Stations: 0, 1, 2 on the first leg, then 1, 2 on the second: five, times three across.
        assert_eq!(g.points().len(), 15);
        // The first station spans y = −0.5 to 0.5 (left of travel along +x is +y).
        assert_eq!(&g.points()[..3], &[Vec3::new(0.0, -0.5, 0.0), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.5, 0.0)]);
        // On the second leg (along +y), left is −x.
        assert_eq!(g.points()[14], Vec3::new(1.5, 2.0, 0.0));
        assert_eq!(Grid::route(&route, 1.0, 0.0, 1.0, 1).unwrap().points().len(), 5);
    }

    #[test]
    fn bad_dimensions_and_huge_grids_are_refused() {
        assert_eq!(Grid::area(&square(4.0), 0.0, 0.0, 0.0), Err(GridError::BadDimension("spacing")));
        assert_eq!(Grid::area(&square(4.0), 0.0, f64::NAN, 0.0), Err(GridError::BadDimension("spacing")));
        assert_eq!(Grid::area(&square(4.0), 0.0, 1.0, -1.0), Err(GridError::BadDimension("border")));
        assert_eq!(Grid::area(&square(4.0)[..2], 0.0, 1.0, 0.0), Err(GridError::BadOutline));
        assert_eq!(Grid::area(&square(1e6), 0.0, 0.01, 0.0), Err(GridError::TooManyPoints));
        assert_eq!(Grid::route(&[Vec2::default()], 1.0, 0.0, 1.0, 3), Err(GridError::BadOutline));
        assert_eq!(
            Grid::route(&[Vec2::default(), Vec2::new(1e9, 0.0)], 1.0, 0.0, 1e-3, 3),
            Err(GridError::TooManyPoints)
        );
        assert_eq!(
            Grid::route(&[Vec2::default(), Vec2::new(1.0, 0.0)], 1.0, 0.0, 1.0, 0),
            Err(GridError::BadDimension("number of points across"))
        );
        assert_eq!(Grid::new(vec![Vec3::default()], Vec3::default()), Err(GridError::BadDimension("point or normal")));
    }
}
