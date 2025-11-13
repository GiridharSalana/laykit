use laykit::*;

#[test]
fn test_gdsii_create_and_write() {
    let mut gds = GDSIIFile::new("TEST_LIB".to_string());
    gds.units = (1e-6, 1e-9);

    let mut structure = GDSStructure {
        name: "TOP".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };

    structure.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
        properties: Vec::new(),
    }));

    gds.structures.push(structure);

    assert!(gds.write_to_file("tests/test_gdsii_create.gds").is_ok());
    std::fs::remove_file("tests/test_gdsii_create.gds").ok();
}

#[test]
fn test_gdsii_round_trip() {
    let mut gds = GDSIIFile::new("ROUNDTRIP".to_string());
    gds.units = (1e-6, 1e-9);

    let mut structure = GDSStructure {
        name: "CELL1".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };

    structure.elements.push(GDSElement::Boundary(Boundary {
        layer: 5,
        datatype: 2,
        xy: vec![(10, 20), (110, 20), (110, 120), (10, 120), (10, 20)],
        properties: Vec::new(),
    }));

    structure.elements.push(GDSElement::Path(GPath {
        layer: 3,
        datatype: 1,
        pathtype: 0,
        width: Some(50),
        xy: vec![(0, 0), (100, 100), (200, 100)],
        properties: Vec::new(),
    }));

    gds.structures.push(structure);
    gds.write_to_file("tests/test_roundtrip.gds").unwrap();

    let gds_read = GDSIIFile::read_from_file("tests/test_roundtrip.gds").unwrap();

    assert_eq!(gds_read.library_name, "ROUNDTRIP");
    assert_eq!(gds_read.structures.len(), 1);

    std::fs::remove_file("tests/test_roundtrip.gds").ok();
}

#[test]
fn test_gdsii_text_element() {
    let mut gds = GDSIIFile::new("TEXT_TEST".to_string());

    let mut structure = GDSStructure {
        name: "TEXT_CELL".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };

    structure.elements.push(GDSElement::Text(GText {
        layer: 10,
        texttype: 0,
        string: "Test Label".to_string(),
        xy: (500, 600),
        presentation: None,
        strans: None,
        width: None,
        properties: Vec::new(),
    }));

    gds.structures.push(structure);
    gds.write_to_file("tests/test_text.gds").unwrap();

    let gds_read = GDSIIFile::read_from_file("tests/test_text.gds").unwrap();

    if let GDSElement::Text(t) = &gds_read.structures[0].elements[0] {
        assert_eq!(t.string, "Test Label");
        assert_eq!(t.layer, 10);
        assert_eq!(t.xy, (500, 600));
    } else {
        panic!("Expected Text element");
    }

    std::fs::remove_file("tests/test_text.gds").ok();
}

#[test]
fn test_gdsii_struct_ref() {
    let mut gds = GDSIIFile::new("HIERARCHY_TEST".to_string());

    let subcell = GDSStructure {
        name: "SUBCELL".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: vec![GDSElement::Boundary(Boundary {
            layer: 1,
            datatype: 0,
            xy: vec![(0, 0), (50, 0), (50, 50), (0, 50), (0, 0)],
            properties: Vec::new(),
        })],
    };

    let mut topcell = GDSStructure {
        name: "TOP".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };

    topcell.elements.push(GDSElement::StructRef(StructRef {
        sname: "SUBCELL".to_string(),
        xy: (100, 200),
        strans: None,
        properties: Vec::new(),
    }));

    gds.structures.push(subcell);
    gds.structures.push(topcell);

    gds.write_to_file("tests/test_hierarchy.gds").unwrap();
    let gds_read = GDSIIFile::read_from_file("tests/test_hierarchy.gds").unwrap();

    assert_eq!(gds_read.structures.len(), 2);

    std::fs::remove_file("tests/test_hierarchy.gds").ok();
}

#[test]
fn test_gdsii_empty_structure() {
    let mut gds = GDSIIFile::new("EMPTY_TEST".to_string());

    let empty_structure = GDSStructure {
        name: "EMPTY".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };

    gds.structures.push(empty_structure);
    gds.write_to_file("tests/test_empty.gds").unwrap();

    let gds_read = GDSIIFile::read_from_file("tests/test_empty.gds").unwrap();
    assert_eq!(gds_read.structures[0].elements.len(), 0);

    std::fs::remove_file("tests/test_empty.gds").ok();
}

#[test]
fn test_gdsii_multiple_layers() {
    let mut gds = GDSIIFile::new("MULTILAYER".to_string());

    let mut structure = GDSStructure {
        name: "MULTILAYER_CELL".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };

    for layer in 1..=5 {
        let base = (layer as i32) * 100;
        structure.elements.push(GDSElement::Boundary(Boundary {
            layer,
            datatype: 0,
            xy: vec![
                (base, base),
                (base + 50, base),
                (base + 50, base + 50),
                (base, base + 50),
                (base, base),
            ],
            properties: Vec::new(),
        }));
    }

    gds.structures.push(structure);
    gds.write_to_file("tests/test_multilayer.gds").unwrap();

    let gds_read = GDSIIFile::read_from_file("tests/test_multilayer.gds").unwrap();
    assert_eq!(gds_read.structures[0].elements.len(), 5);

    std::fs::remove_file("tests/test_multilayer.gds").ok();
}

#[test]
fn test_gdsii_complex_polygon() {
    let mut gds = GDSIIFile::new("COMPLEX_POLY".to_string());

    let mut structure = GDSStructure {
        name: "COMPLEX".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };

    structure.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![
            (100, 0),
            (200, 0),
            (250, 50),
            (250, 150),
            (200, 200),
            (100, 200),
            (50, 150),
            (50, 50),
            (100, 0),
        ],
        properties: Vec::new(),
    }));

    gds.structures.push(structure);
    gds.write_to_file("tests/test_complex.gds").unwrap();

    let gds_read = GDSIIFile::read_from_file("tests/test_complex.gds").unwrap();

    if let GDSElement::Boundary(b) = &gds_read.structures[0].elements[0] {
        assert_eq!(b.xy.len(), 9);
        assert_eq!(b.xy[0], b.xy[8]);
    } else {
        panic!("Expected Boundary");
    }

    std::fs::remove_file("tests/test_complex.gds").ok();
}
