//! EN 1838:2024's requirements on routes, areas and points, as luxpath knows them so far: each with its clause
//! (`docs/standards-parity.md`) and the elements it applies to.
//!
//! No requirement has a threshold yet. Their values come from secondary sources until someone checks them against
//! the standard's text (gap 4), so every finding is "cannot verify" and never "pass"; M4 gives each requirement its
//! verified threshold. Requirements on the whole installation rather than on an element (disability glare,
//! duration, response time, colour rendering) are not listed here yet.

use luxpath_scene::AreaKind;

use crate::ElementKind;

/// The edition every clause below belongs to.
pub const EDITION: &str = "EN 1838:2024";

/// Which elements a requirement applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppliesTo {
    Routes,
    Areas(AreaKind),
    Points,
}

impl AppliesTo {
    pub fn includes(self, kind: ElementKind) -> bool {
        match (self, kind) {
            (Self::Routes, ElementKind::Route) | (Self::Points, ElementKind::Point(_)) => true,
            (Self::Areas(want), ElementKind::Area(have)) => want == have,
            _ => false,
        }
    }
}

/// One requirement: what it asks for, in our own words, never the standard's text (`AGENTS.md` §1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Requirement {
    /// Its entry in `parity/features.toml`.
    pub id: &'static str,
    /// The clause of [`EDITION`] it comes from.
    pub clause: &'static str,
    pub title: &'static str,
    pub applies_to: AppliesTo,
}

/// Every requirement on routes, areas and points, in clause order within each kind of element.
pub const REQUIREMENTS: &[Requirement] = &[
    Requirement {
        id: "en1838.escape-route-full-width",
        clause: "5.1",
        title: "minimum illuminance over the route's full width",
        applies_to: AppliesTo::Routes,
    },
    Requirement {
        id: "en1838.escape-route-uniformity",
        clause: "5.1",
        title: "uniformity along the route (max/min)",
        applies_to: AppliesTo::Routes,
    },
    Requirement {
        id: "en1838.open-area",
        clause: "5.2",
        title: "minimum illuminance less the border, and uniformity",
        applies_to: AppliesTo::Areas(AreaKind::Open),
    },
    Requirement {
        id: "en1838.high-risk",
        clause: "5.3",
        title: "illuminance relative to the normal lighting, and uniformity",
        applies_to: AppliesTo::Areas(AreaKind::HighRisk),
    },
    Requirement {
        id: "en1838.specific-areas",
        clause: "5.4.1",
        title: "toilets and changing rooms",
        applies_to: AppliesTo::Areas(AreaKind::Toilets),
    },
    Requirement {
        id: "en1838.specific-areas",
        clause: "5.4.2",
        title: "public indoor swimming pools",
        applies_to: AppliesTo::Areas(AreaKind::SwimmingPool),
    },
    Requirement {
        id: "en1838.specific-areas",
        clause: "5.4.3",
        title: "motor generator, control, switch and plant rooms",
        applies_to: AppliesTo::Areas(AreaKind::PlantRoom),
    },
    Requirement {
        id: "en1838.local-area",
        clause: "7.1",
        title: "local area lighting",
        applies_to: AppliesTo::Areas(AreaKind::LocalArea),
    },
    Requirement {
        id: "en1838.points-of-emphasis",
        clause: "4.2",
        title: "illuminance at the point of emphasis",
        applies_to: AppliesTo::Points,
    },
];

/// The requirements that apply to an element of `kind`.
pub fn requirements_for(kind: ElementKind) -> impl Iterator<Item = &'static Requirement> {
    REQUIREMENTS.iter().filter(move |r| r.applies_to.includes(kind))
}
