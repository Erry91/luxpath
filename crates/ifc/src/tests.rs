use super::*;

/// A hand-written IFC4 model. What it contains, so the expected counts can be checked by reading:
/// - #10 exit sign, PredefinedType SECURITYLIGHTING, no type object → emergency;
/// - #11 downlight, NOTDEFINED, typed by #22 (POINTSOURCE); a property names an `.IES` file;
/// - #12 PredefinedType unset, typed by #20 (SECURITYLIGHTING) whose own property set #30 gives
///   manufacturer Acme and article number EM-100 → emergency, with a product code;
/// - spaces #50 (FireExit true) and #51 (FireExit false); door #60 (FireExit true);
/// - one stair, one fire suppression terminal, two alarms of which one is a manual call point.
const IFC4: &str = r"ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('ViewDefinition [ReferenceView]'),'2;1');
FILE_NAME('test.ifc','2026-10-16T00:00:00',('a'),('b'),'luxpath test \X2\00E8\X0\','Synthetic Exporter 1.0','');
FILE_SCHEMA(('IFC4'));
ENDSEC;
DATA;
#1=IFCBUILDINGSTOREY('0s',$,'Ground',$,$,$,$,$,.ELEMENT.,0.);
#10=IFCLIGHTFIXTURE('1a',$,'Exit sign',$,$,$,$,$,.SECURITYLIGHTING.);
#11=IFCLIGHTFIXTURE('1b',$,'Downlight',$,$,$,$,$,.NOTDEFINED.);
#12=IFCLIGHTFIXTURE('1c',$,'Emergency downlight',$,$,$,$,$,$);
#20=IFCLIGHTFIXTURETYPE('2a',$,'EM type',$,$,(#30),$,$,$,.SECURITYLIGHTING.);
#22=IFCLIGHTFIXTURETYPE('2b',$,'Downlight type',$,$,$,$,$,$,.POINTSOURCE.);
#23=IFCRELDEFINESBYTYPE('3a',$,$,$,(#12),#20);
#24=IFCRELDEFINESBYTYPE('3b',$,$,$,(#11),#22);
#30=IFCPROPERTYSET('4a',$,'Pset_ManufacturerTypeInformation',$,(#31,#32));
#31=IFCPROPERTYSINGLEVALUE('Manufacturer',$,IFCLABEL('Acme'),$);
#32=IFCPROPERTYSINGLEVALUE('ArticleNumber',$,IFCIDENTIFIER('EM-100'),$);
#40=IFCPROPERTYSET('4b',$,'Other',$,(#41));
#41=IFCPROPERTYSINGLEVALUE('Photometric Web File',$,IFCTEXT('downlight.IES'),$);
#42=IFCRELDEFINESBYPROPERTIES('5a',$,$,$,(#11),#40);
#50=IFCSPACE('6a',$,'Corridor',$,$,$,$,$,.ELEMENT.,.INTERNAL.,$);
#51=IFCSPACE('6b',$,'Office',$,$,$,$,$,.ELEMENT.,.INTERNAL.,$);
#52=IFCPROPERTYSET('7a',$,'Pset_SpaceFireSafetyRequirements',$,(#53));
#53=IFCPROPERTYSINGLEVALUE('FireExit',$,IFCBOOLEAN(.T.),$);
#54=IFCRELDEFINESBYPROPERTIES('7b',$,$,$,(#50),#52);
#55=IFCPROPERTYSET('7c',$,'Pset_SpaceFireSafetyRequirements',$,(#56));
#56=IFCPROPERTYSINGLEVALUE('FireExit',$,IFCBOOLEAN(.F.),$);
#57=IFCRELDEFINESBYPROPERTIES('7d',$,$,$,(#51),#55);
#60=IFCDOOR('8a',$,'Exit door',$,$,$,$,$,2.1,0.9,.DOOR.,.SINGLE_SWING_LEFT.,$);
#61=IFCPROPERTYSINGLEVALUE('FireExit',$,IFCBOOLEAN(.T.),$);
#62=IFCPROPERTYSET('8b',$,'Pset_DoorCommon',$,(#61));
#63=IFCRELDEFINESBYPROPERTIES('8c',$,$,$,(#60),#62);
#70=IFCSTAIR('9a',$,'Stair',$,$,$,$,$,.STRAIGHT_RUN_STAIR.);
#80=IFCALARM('9b',$,'Call point',$,$,$,$,$,.MANUALPULLBOX.);
#81=IFCALARM('9c',$,'Bell',$,$,$,$,$,.BELL.);
#90=IFCFIRESUPPRESSIONTERMINAL('9d',$,'Extinguisher',$,$,$,$,$,.NOTDEFINED.);
ENDSEC;
END-ISO-10303-21;
";

/// IFC2x3 has no IfcLightFixture: lights are flow terminals typed by an IfcLightFixtureType.
const IFC2X3: &str = r"ISO-10303-21;
HEADER;
FILE_DESCRIPTION((''),'2;1');
FILE_NAME('old.ifc','2012-01-01T00:00:00',(''),(''),'pre','Old Exporter','');
FILE_SCHEMA(('IFC2X3'));
ENDSEC;
DATA;
#10=IFCFLOWTERMINAL('1a',$,'Lamp',$,$,$,$,$);
#11=IFCFLOWTERMINAL('1b',$,'Washbasin',$,$,$,$,$);
#20=IFCLIGHTFIXTURETYPE('2a',$,'Lamp type',$,$,$,$,$,$,.POINTSOURCE.);
#21=IFCRELDEFINESBYTYPE('3a',$,$,$,(#10),#20);
#30=IFCSANITARYTERMINALTYPE('2b',$,'Basin type',$,$,$,$,$,$,.WASHHANDBASIN.);
#31=IFCRELDEFINESBYTYPE('3b',$,$,$,(#11),#30);
ENDSEC;
END-ISO-10303-21;
";

#[test]
fn the_header_names_the_schema_and_the_exporter() {
    let i = inspect(IFC4.as_bytes()).unwrap();
    assert_eq!(i.schema.as_deref(), Some("IFC4"));
    assert_eq!(i.originating_system.as_deref(), Some("Synthetic Exporter 1.0"));
    // STEP escapes are decoded: \X2\00E8\X0\ is "è".
    assert_eq!(i.preprocessor.as_deref(), Some("luxpath test è"));
}

#[test]
fn ifc4_luminaires_are_classified() {
    let l = inspect(IFC4.as_bytes()).unwrap().luminaires;
    assert_eq!(l.total, 3);
    assert_eq!((l.light_fixture_entities, l.typed_other_entities, l.type_objects), (3, 0, 2));
    // #10 by its own predefined type, #12 through its type object; #11 is a POINTSOURCE.
    assert_eq!(l.predefined_types.get("SECURITYLIGHTING"), Some(&2));
    assert_eq!(l.predefined_types.get("POINTSOURCE"), Some(&1));
    assert_eq!(l.emergency, 2);
    // Only #12 inherits the manufacturer and article number from its type's property set.
    assert_eq!((l.with_manufacturer, l.with_product_code, l.distinct_product_codes), (1, 1, 1));
    // #11 and #12 have named types; #10 has no type object.
    assert_eq!((l.with_type_name, l.distinct_type_names), (2, 2));
    assert_eq!(l.with_photometric_file_reference, 1);
    assert_eq!(l.property_sets.get("Pset_ManufacturerTypeInformation"), Some(&1));
}

#[test]
fn fire_exits_stairs_and_safety_equipment_are_counted() {
    let i = inspect(IFC4.as_bytes()).unwrap();
    assert_eq!(i.spaces, FlaggedCount { total: 2, fire_exit: 1 });
    assert_eq!(i.doors, FlaggedCount { total: 1, fire_exit: 1 });
    assert_eq!((i.stairs, i.fire_suppression_terminals, i.alarms, i.manual_call_points), (1, 1, 2, 1));
    assert_eq!(i.storeys, 1);
}

#[test]
fn ifc2x3_flow_terminals_typed_as_lights_count_as_luminaires() {
    let i = inspect(IFC2X3.as_bytes()).unwrap();
    assert_eq!(i.schema.as_deref(), Some("IFC2X3"));
    let l = &i.luminaires;
    // The washbasin (#11) is typed too, but by a sanitary terminal type: it must not count.
    assert_eq!((l.total, l.light_fixture_entities, l.typed_other_entities), (1, 0, 1));
    assert_eq!(l.predefined_types.get("POINTSOURCE"), Some(&1));
    assert_eq!(l.emergency, 0);
    let findings = i.findings().join("\n");
    assert!(findings.contains("No luminaire is marked as emergency lighting"));
    assert!(findings.contains("No photometric data"));
    assert!(findings.contains("No spaces"));
}

#[test]
fn a_complete_model_has_no_photometry_finding_but_reports_missing_codes() {
    let findings = inspect(IFC4.as_bytes()).unwrap().findings().join("\n");
    assert!(!findings.contains("No photometric data"));
    assert!(!findings.contains("FireExit"));
    assert!(findings.contains("2 of 3 luminaires have no product code"));
    assert!(findings.contains("2 distinct type names could serve as the key instead"));
}

#[test]
fn files_that_are_not_step_are_rejected() {
    assert_eq!(inspect(b""), Err(IfcError::NotStep));
    assert_eq!(inspect(b"PK\x03\x04 a zip, perhaps an .ifczip"), Err(IfcError::NotStep));
    assert!(inspect(b"  \nISO-10303-21;\nHEADER;ENDSEC;DATA;ENDSEC;").is_ok());
}

#[test]
fn truncated_files_never_panic() {
    let bytes = IFC4.as_bytes();
    for end in 0..=bytes.len() {
        let _ = inspect(bytes.get(..end).unwrap());
    }
}

#[test]
fn garbage_in_the_data_section_never_panics() {
    let mut text = IFC4.to_string();
    for junk in [
        "#99=IFCPROPERTYSET(;",
        "#100=IFCRELDEFINESBYPROPERTIES('x',$,$,$,#999,(#1));",
        "#101=IFCLIGHTFIXTURE(",
        "''''",
    ] {
        text = text.replace("ENDSEC;\nEND", &format!("{junk}\nENDSEC;\nEND"));
        let _ = inspect(text.as_bytes());
    }
}
