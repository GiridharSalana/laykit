use laykit::*;

#[test]
fn test_rectangle_detection() {
    let rect_points = vec![(0, 0), (100, 0), (100, 50), (0, 50), (0, 0)];
    assert!(converter::is_rectangle(&rect_points));

    let not_rect = vec![(0, 0), (100, 0), (100, 50)];
    assert!(!converter::is_rectangle(&not_rect));
}

#[test]
fn test_gdsii_to_oasis_conversion() {
    let mut gds = GDSIIFile::new("TEST".to_string());
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

    let oasis = converter::gdsii_to_oasis(&gds).unwrap();
    assert_eq!(oasis.cells.len(), 1);
    assert_eq!(oasis.cells[0].name, "TOP");
}
