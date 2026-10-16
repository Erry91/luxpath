//! Reads IFC (BIM) models and reports what they contain for emergency-lighting checks.
//!
//! M0 has the **inspector**: one pass over an IFC file (IFC2x3 or IFC4, STEP encoding) that answers
//! "could this model be checked against EN 1838, and what is missing?":
//! - **luminaires:** `IfcLightFixture` (IFC4) and anything typed by an `IfcLightFixtureType` (IFC2x3
//!   exporters write lights as `IfcFlowTerminal`); which are marked as emergency lighting
//!   (`SECURITYLIGHTING`, `Pset_LightFixtureTypeSecurityLighting`); which carry a product code
//!   (`Pset_ManufacturerTypeInformation`) or photometry (`IfcLightSourceGoniometric`, an
//!   `IfcLightIntensityDistribution`, or a property pointing at an IES/LDT file);
//! - **escape-relevant building data:** spaces and doors flagged `FireExit`, stairs, fire-fighting
//!   equipment, manual call points.
//!
//! Parsing is done by `ifc-lite-core` (MPL-2.0, unmodified dependency). This crate does no I/O:
//! it takes the file's bytes.
//!
//! Attribute positions below follow the IFC schema (IFC2x3 TC1 and IFC4 ADD2 TC1 agree on every
//! position used here); each one is named where it is used.
//!
//! Layer L3: depends on nothing in luxpath yet (later: `luxpath-scene`, to build scenes from models).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;

use ifc_lite_core::{AttributeValue, EntityDecoder, EntityScanner, build_entity_index, decode_ifc_string};
use serde::Serialize;

/// What the inspector found. Counts are of entity occurrences unless the name says otherwise.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Inspection {
    /// `FILE_SCHEMA`, e.g. `IFC4`.
    pub schema: Option<String>,
    /// `FILE_NAME` originating system (the exporting application).
    pub originating_system: Option<String>,
    /// `FILE_NAME` preprocessor version (often the exporter's toolkit).
    pub preprocessor: Option<String>,
    /// Entity records in the DATA section.
    pub entities: usize,
    pub storeys: usize,
    pub luminaires: LuminaireSummary,
    pub spaces: FlaggedCount,
    pub doors: FlaggedCount,
    pub stairs: usize,
    /// `IfcFireSuppressionTerminal` (extinguishers, hose reels, sprinklers…).
    pub fire_suppression_terminals: usize,
    /// `IfcAlarm`, and how many are manual call points (`MANUALPULLBOX`).
    pub alarms: usize,
    pub manual_call_points: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct LuminaireSummary {
    pub total: usize,
    /// Written as `IfcLightFixture` (IFC4).
    pub light_fixture_entities: usize,
    /// Written as another entity (usually `IfcFlowTerminal`) typed by an `IfcLightFixtureType`.
    pub typed_other_entities: usize,
    /// `IfcLightFixtureType` objects in the file.
    pub type_objects: usize,
    /// PredefinedType of each luminaire (occurrence first, then its type); `unset` when neither says.
    pub predefined_types: BTreeMap<String, usize>,
    /// Marked as emergency lighting: `SECURITYLIGHTING`, or `Pset_LightFixtureTypeSecurityLighting`.
    pub emergency: usize,
    /// With a manufacturer name in `Pset_ManufacturerTypeInformation`.
    pub with_manufacturer: usize,
    /// With a product code (`ArticleNumber`, `ModelReference` or `ModelLabel`).
    pub with_product_code: usize,
    /// Distinct product codes: roughly how many photometric files a mapping would need.
    pub distinct_product_codes: usize,
    /// With a type object that has a name: the fallback key for linking photometry when there
    /// is no product code (e.g. a Revit type name like `600x600 - 277`).
    pub with_type_name: usize,
    pub distinct_type_names: usize,
    /// With a property that names a photometric file (`.ies`/`.ldt`) or is called "photometric…".
    pub with_photometric_file_reference: usize,
    /// Photometric entities anywhere in the file.
    pub goniometric_sources: usize,
    pub intensity_distributions: usize,
    /// Property set names found on luminaires (occurrence and type), with how many luminaires have each.
    pub property_sets: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct FlaggedCount {
    pub total: usize,
    /// `FireExit = TRUE` (`Pset_SpaceFireSafetyRequirements` for spaces, `Pset_DoorCommon` for doors).
    pub fire_exit: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IfcError {
    /// The bytes don't start like an ISO 10303-21 (STEP) file.
    NotStep,
}

impl std::fmt::Display for IfcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStep => write!(f, "not an IFC file: it does not start with ISO-10303-21"),
        }
    }
}

impl std::error::Error for IfcError {}

/// Inspects an IFC file given as bytes. Malformed records are skipped, never a panic.
pub fn inspect(content: &[u8]) -> Result<Inspection, IfcError> {
    let start = content.iter().position(|b| !b.is_ascii_whitespace()).unwrap_or(0);
    if !content.get(start..).is_some_and(|rest| rest.starts_with(b"ISO-10303-21")) {
        return Err(IfcError::NotStep);
    }
    let mut out = Inspection::default();
    if let Some(fields) = header_fields(content, "FILE_SCHEMA") {
        out.schema =
            fields.first().map(|s| s.trim_matches(|c| c == '(' || c == ')').to_string()).filter(|s| !s.is_empty());
    }
    if let Some(fields) = header_fields(content, "FILE_NAME") {
        out.preprocessor = fields.get(4).cloned().filter(|s| !s.is_empty());
        out.originating_system = fields.get(5).cloned().filter(|s| !s.is_empty());
    }

    // Pass 1: find the entities that matter, by keyword. Spans are decoded later, on demand.
    let mut scan = Scan::default();
    let mut scanner = EntityScanner::new(content);
    while let Some((id, keyword, start, end)) = scanner.next_entity() {
        out.entities = out.entities.saturating_add(1);
        let span = (id, start, end);
        match keyword.to_ascii_uppercase().as_str() {
            "IFCLIGHTFIXTURE" => scan.light_fixtures.push(span),
            "IFCLIGHTFIXTURETYPE" => scan.light_types.push(span),
            "IFCSPACE" => scan.spaces.push(id),
            "IFCDOOR" | "IFCDOORSTANDARDCASE" => scan.doors.push(id),
            "IFCSTAIR" => out.stairs += 1,
            "IFCFIRESUPPRESSIONTERMINAL" => out.fire_suppression_terminals += 1,
            "IFCALARM" => scan.alarms.push(span),
            "IFCBUILDINGSTOREY" => out.storeys += 1,
            "IFCLIGHTSOURCEGONIOMETRIC" => out.luminaires.goniometric_sources += 1,
            "IFCLIGHTINTENSITYDISTRIBUTION" => out.luminaires.intensity_distributions += 1,
            "IFCRELDEFINESBYTYPE" => scan.rel_types.push(span),
            "IFCRELDEFINESBYPROPERTIES" => scan.rel_props.push(span),
            _ => {}
        }
    }

    let mut model = Model::new(content);
    let lights = model.collect_lights(&scan);
    let psets_of = model.psets_by_object(&scan);
    let type_psets = model.type_psets(&scan);

    // Luminaires.
    let lum = &mut out.luminaires;
    lum.type_objects = scan.light_types.len();
    let mut codes = BTreeSet::new();
    let mut type_names = BTreeSet::new();
    for (id, light) in &lights {
        lum.total += 1;
        if light.is_fixture_entity {
            lum.light_fixture_entities += 1;
        } else {
            lum.typed_other_entities += 1;
        }
        let type_info = light.type_id.and_then(|t| model.light_type_info.get(&t)).cloned();
        let predefined = light
            .predefined
            .clone()
            .filter(|p| !p.eq_ignore_ascii_case("NOTDEFINED"))
            .or_else(|| type_info.as_ref().and_then(|t| t.predefined.clone()))
            .unwrap_or_else(|| "unset".to_string());
        *lum.predefined_types.entry(predefined.clone()).or_default() += 1;
        if let Some(name) = type_info.as_ref().and_then(|t| t.name.clone()) {
            lum.with_type_name += 1;
            type_names.insert(name);
        }

        let mut pset_ids: Vec<u32> = psets_of.get(id).cloned().unwrap_or_default();
        if let Some(t) = light.type_id {
            pset_ids.extend(type_psets.get(&t).into_iter().flatten().copied());
            pset_ids.extend(psets_of.get(&t).into_iter().flatten().copied());
        }
        let psets: Vec<Rc<Pset>> = pset_ids.iter().filter_map(|p| model.pset(*p)).collect();
        let names: BTreeSet<&str> = psets.iter().map(|p| p.name.as_str()).collect();
        for name in &names {
            *lum.property_sets.entry((*name).to_string()).or_default() += 1;
        }
        if predefined.eq_ignore_ascii_case("SECURITYLIGHTING")
            || names.contains("Pset_LightFixtureTypeSecurityLighting")
        {
            lum.emergency += 1;
        }
        let manufacturer = psets.iter().filter(|p| p.name == "Pset_ManufacturerTypeInformation");
        if manufacturer.clone().any(|p| p.text("Manufacturer").is_some()) {
            lum.with_manufacturer += 1;
        }
        let code = manufacturer.clone().find_map(|p| {
            ["ArticleNumber", "ModelReference", "ModelLabel"].iter().find_map(|k| p.text(k).map(str::to_string))
        });
        if let Some(code) = code {
            lum.with_product_code += 1;
            codes.insert(code);
        }
        if psets.iter().any(|p| p.names_photometric_file()) {
            lum.with_photometric_file_reference += 1;
        }
    }
    lum.distinct_product_codes = codes.len();
    lum.distinct_type_names = type_names.len();

    // Spaces and doors flagged as fire exits.
    out.spaces.total = scan.spaces.len();
    out.spaces.fire_exit = count_flagged(&mut model, &scan.spaces, &psets_of, "Pset_SpaceFireSafetyRequirements");
    out.doors.total = scan.doors.len();
    out.doors.fire_exit = count_flagged(&mut model, &scan.doors, &psets_of, "Pset_DoorCommon");

    // Alarms: IfcAlarm.PredefinedType is attribute 8.
    out.alarms = scan.alarms.len();
    out.manual_call_points = scan
        .alarms
        .iter()
        .filter(|(_, s, e)| model.enum_at(*s, *e, 8).is_some_and(|v| v.eq_ignore_ascii_case("MANUALPULLBOX")))
        .count();
    Ok(out)
}

impl Inspection {
    /// What stands between this model and an EN 1838 check, in plain language.
    pub fn findings(&self) -> Vec<String> {
        let l = &self.luminaires;
        let mut f = Vec::new();
        if l.total == 0 {
            f.push("No luminaires found.".to_string());
        } else {
            if l.emergency == 0 {
                f.push(
                    "No luminaire is marked as emergency lighting (SECURITYLIGHTING or \
                     Pset_LightFixtureTypeSecurityLighting): emergency luminaires must be identified another way."
                        .to_string(),
                );
            }
            if l.goniometric_sources == 0 && l.intensity_distributions == 0 && l.with_photometric_file_reference == 0 {
                f.push(
                    "No photometric data and no photometric file references: photometry must be linked externally."
                        .to_string(),
                );
            }
            if l.with_product_code < l.total {
                let fallback = if l.distinct_type_names > 0 {
                    format!(" ({} distinct type names could serve as the key instead)", l.distinct_type_names)
                } else {
                    String::new()
                };
                f.push(format!(
                    "{} of {} luminaires have no product code to link photometry by{fallback}.",
                    l.total - l.with_product_code,
                    l.total
                ));
            }
        }
        if self.spaces.total == 0 {
            f.push("No spaces: escape routes and open areas cannot be derived from the model.".to_string());
        } else if self.spaces.fire_exit == 0 {
            f.push("No space is flagged FireExit: escape routes must be identified another way.".to_string());
        }
        if self.doors.total > 0 && self.doors.fire_exit == 0 {
            f.push(
                "No door is flagged FireExit: exits (points of emphasis) must be identified another way.".to_string(),
            );
        }
        f
    }

    /// A human-readable summary.
    pub fn to_text(&self) -> String {
        let l = &self.luminaires;
        let opt = |s: &Option<String>| s.clone().unwrap_or_else(|| "unknown".to_string());
        let mut t = String::new();
        t.push_str(&format!("schema:              {}\n", opt(&self.schema)));
        t.push_str(&format!("exported by:         {} ({})\n", opt(&self.originating_system), opt(&self.preprocessor)));
        t.push_str(&format!("entities:            {}   storeys: {}\n\n", self.entities, self.storeys));
        t.push_str(&format!(
            "luminaires:          {} ({} IfcLightFixture, {} typed others; {} type objects)\n",
            l.total, l.light_fixture_entities, l.typed_other_entities, l.type_objects
        ));
        let types: Vec<String> = l.predefined_types.iter().map(|(k, v)| format!("{k} {v}")).collect();
        t.push_str(&format!(
            "  predefined types:  {}\n",
            if types.is_empty() { "-".to_string() } else { types.join(", ") }
        ));
        t.push_str(&format!("  emergency:         {}\n", l.emergency));
        t.push_str(&format!(
            "  manufacturer:      {}   product code: {} ({} distinct)   type name: {} ({} distinct)\n",
            l.with_manufacturer, l.with_product_code, l.distinct_product_codes, l.with_type_name, l.distinct_type_names
        ));
        t.push_str(&format!(
            "  photometry:        {} goniometric sources, {} intensity tables, {} file references\n",
            l.goniometric_sources, l.intensity_distributions, l.with_photometric_file_reference
        ));
        for (name, n) in &l.property_sets {
            t.push_str(&format!("  pset {name}: {n}\n"));
        }
        t.push_str(&format!("\nspaces:              {} ({} FireExit)\n", self.spaces.total, self.spaces.fire_exit));
        t.push_str(&format!("doors:               {} ({} FireExit)\n", self.doors.total, self.doors.fire_exit));
        t.push_str(&format!("stairs:              {}\n", self.stairs));
        t.push_str(&format!("fire suppression:    {}\n", self.fire_suppression_terminals));
        t.push_str(&format!("alarms:              {} ({} manual call points)\n", self.alarms, self.manual_call_points));
        let findings = self.findings();
        if !findings.is_empty() {
            t.push_str("\nfor an EN 1838 check:\n");
            for line in findings {
                t.push_str(&format!("  - {line}\n"));
            }
        }
        t
    }
}

type Span = (u32, usize, usize);

#[derive(Default)]
struct Scan {
    light_fixtures: Vec<Span>,
    light_types: Vec<Span>,
    spaces: Vec<u32>,
    doors: Vec<u32>,
    alarms: Vec<Span>,
    rel_types: Vec<Span>,
    rel_props: Vec<Span>,
}

struct Light {
    is_fixture_entity: bool,
    predefined: Option<String>,
    type_id: Option<u32>,
}

#[derive(Clone)]
struct LightTypeInfo {
    name: Option<String>,
    predefined: Option<String>,
}

struct Pset {
    name: String,
    props: Vec<(String, AttributeValue)>,
}

impl Pset {
    fn value(&self, name: &str) -> Option<&AttributeValue> {
        self.props.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    fn text(&self, name: &str) -> Option<&str> {
        self.value(name).and_then(|v| unwrap_typed(v).as_string()).map(str::trim).filter(|s| !s.is_empty())
    }

    fn flag(&self, name: &str) -> Option<bool> {
        let v = unwrap_typed(self.value(name)?);
        match v.as_enum().or_else(|| v.as_string())?.trim_matches('.').to_ascii_uppercase().as_str() {
            "T" | "TRUE" => Some(true),
            "F" | "FALSE" => Some(false),
            _ => None,
        }
    }

    fn names_photometric_file(&self) -> bool {
        self.props.iter().any(|(name, value)| {
            let text = unwrap_typed(value).as_string().unwrap_or("").to_ascii_lowercase();
            let names_file = text.ends_with(".ies") || text.ends_with(".ldt");
            names_file || (name.to_ascii_lowercase().contains("photometric") && !text.trim().is_empty())
        })
    }
}

/// Typed values (`IFCLABEL('x')`, `IFCBOOLEAN(.T.)`) decode as `List([String(type), value])`, and
/// enumerated property values as a list of those. Returns the innermost plain value.
fn unwrap_typed(v: &AttributeValue) -> &AttributeValue {
    let mut v = v;
    // Bounded: at most a few levels of nesting occur in valid files.
    for _ in 0..4 {
        match v {
            AttributeValue::List(items)
                if matches!(items.first(), Some(AttributeValue::String(_))) && items.len() == 2 =>
            {
                v = items.get(1).unwrap_or(v);
            }
            AttributeValue::List(items) if matches!(items.first(), Some(AttributeValue::List(_))) => {
                v = items.first().unwrap_or(v);
            }
            _ => break,
        }
    }
    v
}

struct Model<'a> {
    decoder: EntityDecoder<'a>,
    light_type_info: HashMap<u32, LightTypeInfo>,
    pset_cache: HashMap<u32, Option<Rc<Pset>>>,
}

impl<'a> Model<'a> {
    fn new(content: &'a [u8]) -> Self {
        let index = build_entity_index(content);
        Self {
            decoder: EntityDecoder::with_index(content, index),
            light_type_info: HashMap::new(),
            pset_cache: HashMap::new(),
        }
    }

    fn enum_at(&mut self, start: usize, end: usize, attr: usize) -> Option<String> {
        let e = self.decoder.decode_at(start, end).ok()?;
        e.get(attr).and_then(AttributeValue::as_enum).map(str::to_string)
    }

    fn collect_lights(&mut self, scan: &Scan) -> BTreeMap<u32, Light> {
        // IfcLightFixtureType: Name is attribute 2; PredefinedType is attribute 9 (IfcElementType
        // has 9 attributes before it).
        for (id, s, e) in &scan.light_types {
            let Ok(t) = self.decoder.decode_at(*s, *e) else { continue };
            let name = t.get_string(2).map(str::trim).filter(|n| !n.is_empty()).map(str::to_string);
            let predefined = t
                .get(9)
                .and_then(AttributeValue::as_enum)
                .filter(|p| !p.eq_ignore_ascii_case("NOTDEFINED"))
                .map(str::to_string);
            self.light_type_info.insert(*id, LightTypeInfo { name, predefined });
        }
        let mut lights = BTreeMap::new();
        // IfcLightFixture.PredefinedType is attribute 8 (IFC4; the entity doesn't exist in IFC2x3).
        for (id, s, e) in &scan.light_fixtures {
            let predefined = self.enum_at(*s, *e, 8);
            lights.insert(*id, Light { is_fixture_entity: true, predefined, type_id: None });
        }
        // IfcRelDefinesByType: RelatedObjects is attribute 4, RelatingType attribute 5.
        for (_, s, e) in &scan.rel_types {
            let Ok(rel) = self.decoder.decode_at(*s, *e) else { continue };
            let Some(type_id) = rel.get_ref(5) else { continue };
            if !self.light_type_info.contains_key(&type_id) {
                continue;
            }
            for object in rel.get_refs(4).unwrap_or_default() {
                lights
                    .entry(object)
                    .or_insert(Light { is_fixture_entity: false, predefined: None, type_id: None })
                    .type_id = Some(type_id);
            }
        }
        lights
    }

    /// Object id → property set ids, from IfcRelDefinesByProperties (RelatedObjects attribute 4,
    /// RelatingPropertyDefinition attribute 5).
    fn psets_by_object(&mut self, scan: &Scan) -> HashMap<u32, Vec<u32>> {
        let mut map: HashMap<u32, Vec<u32>> = HashMap::new();
        for (_, s, e) in &scan.rel_props {
            let Ok(rel) = self.decoder.decode_at(*s, *e) else { continue };
            let Some(pset) = rel.get_ref(5) else { continue };
            for object in rel.get_refs(4).unwrap_or_default() {
                map.entry(object).or_default().push(pset);
            }
        }
        map
    }

    /// Light fixture type id → its own property sets (IfcTypeObject.HasPropertySets, attribute 5).
    fn type_psets(&mut self, scan: &Scan) -> HashMap<u32, Vec<u32>> {
        let mut map = HashMap::new();
        for (id, s, e) in &scan.light_types {
            if let Ok(t) = self.decoder.decode_at(*s, *e) {
                map.insert(*id, t.get_refs(5).unwrap_or_default());
            }
        }
        map
    }

    /// Decodes a property set (Name attribute 2, HasProperties attribute 4) and its single and
    /// enumerated values (Name attribute 0, value attribute 2). Cached.
    fn pset(&mut self, id: u32) -> Option<Rc<Pset>> {
        if let Some(cached) = self.pset_cache.get(&id) {
            return cached.clone();
        }
        let decoded = self.decoder.decode_by_id(id).ok().and_then(|p| {
            let name = p.get_string(2)?.to_string();
            let mut props = Vec::new();
            for prop_id in p.get_refs(4).unwrap_or_default() {
                let Ok(prop) = self.decoder.decode_by_id(prop_id) else { continue };
                let (Some(prop_name), Some(value)) = (prop.get_string(0), prop.get(2)) else { continue };
                props.push((prop_name.to_string(), value.clone()));
            }
            Some(Rc::new(Pset { name, props }))
        });
        self.pset_cache.insert(id, decoded.clone());
        decoded
    }
}

fn count_flagged(model: &mut Model<'_>, objects: &[u32], psets_of: &HashMap<u32, Vec<u32>>, pset_name: &str) -> usize {
    let wanted: HashSet<u32> = objects.iter().copied().collect();
    wanted
        .iter()
        .filter(|object| {
            psets_of.get(object).into_iter().flatten().any(|p| {
                model.pset(*p).is_some_and(|pset| pset.name == pset_name && pset.flag("FireExit") == Some(true))
            })
        })
        .count()
}

/// The top-level fields of a HEADER record such as `FILE_NAME(...)`, with quotes removed and STEP
/// string escapes decoded. Nested lists are returned as their raw text.
fn header_fields(content: &[u8], keyword: &str) -> Option<Vec<String>> {
    // The HEADER section ends at DATA; never look further than that (or 1 MiB).
    let limit = content.len().min(1 << 20);
    let window = content.get(..limit)?;
    let header_end = find(window, b"DATA;").unwrap_or(limit);
    let header = window.get(..header_end)?;
    let after = find(header, keyword.as_bytes())?.checked_add(keyword.len())?;
    let mut bytes = header.get(after..)?.iter().copied().peekable();
    loop {
        match bytes.next()? {
            b'(' => break,
            b if b.is_ascii_whitespace() => continue,
            _ => return None,
        }
    }
    let mut fields = Vec::new();
    let mut current = Vec::new();
    let (mut depth, mut in_string) = (0usize, false);
    let finish = |buf: &mut Vec<u8>| -> String {
        let s = decode_ifc_string(&String::from_utf8_lossy(buf)).into_owned();
        buf.clear();
        s
    };
    while let Some(b) = bytes.next() {
        if in_string {
            if b == b'\'' {
                if bytes.peek() == Some(&b'\'') {
                    bytes.next();
                    current.push(b'\'');
                } else {
                    in_string = false;
                }
            } else {
                current.push(b);
            }
            continue;
        }
        match b {
            b'\'' => in_string = true,
            b'(' => {
                depth = depth.saturating_add(1);
                current.push(b);
            }
            b')' if depth == 0 => {
                fields.push(finish(&mut current));
                return Some(fields);
            }
            b')' => {
                depth -= 1;
                current.push(b);
            }
            b',' if depth == 0 => fields.push(finish(&mut current)),
            b if b.is_ascii_whitespace() => {}
            _ => current.push(b),
        }
    }
    None
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[cfg(test)]
mod tests;
