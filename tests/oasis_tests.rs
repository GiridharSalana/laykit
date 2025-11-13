use laykit::*;

#[test]
fn test_oasis_create_simple() {
    let mut oasis = OASISFile::new();
    oasis.names.cell_names.insert(0, "SIMPLE".to_string());

    let mut cell = OASISCell {
        name: "SIMPLE".to_string(),
        elements: Vec::new(),
    };

    cell.elements.push(OASISElement::Rectangle(Rectangle {
        layer: 1,
        datatype: 0,
        x: 0,
        y: 0,
        width: 100,
        height: 50,
        repetition: None,
        properties: Vec::new(),
    }));

    oasis.cells.push(cell);

    assert!(oasis.write_to_file("tests/test_oasis_simple.oas").is_ok());
    std::fs::remove_file("tests/test_oasis_simple.oas").ok();
}

#[test]
fn test_oasis_round_trip_rectangles() {
    let mut oasis = OASISFile::new();
    oasis.names.cell_names.insert(0, "RECT_CELL".to_string());

    let mut cell = OASISCell {
        name: "RECT_CELL".to_string(),
        elements: Vec::new(),
    };

    // Add multiple rectangles
    for i in 0..3 {
        cell.elements.push(OASISElement::Rectangle(Rectangle {
            layer: i + 1,
            datatype: 0,
            x: (i as i64) * 200,
            y: (i as i64) * 150,
            width: 100 + (i as u64) * 10,
            height: 80 + (i as u64) * 10,
            repetition: None,
            properties: Vec::new(),
        }));
    }

    oasis.cells.push(cell);
    oasis.write_to_file("tests/test_oasis_rects.oas").unwrap();

    // Read back
    let oasis_read = OASISFile::read_from_file("tests/test_oasis_rects.oas").unwrap();

    assert_eq!(oasis_read.cells.len(), 1);
    assert_eq!(oasis_read.cells[0].name, "RECT_CELL");
    assert_eq!(oasis_read.cells[0].elements.len(), 3);

    // Verify each rectangle
    for (i, element) in oasis_read.cells[0].elements.iter().enumerate() {
        if let OASISElement::Rectangle(rect) = element {
            assert_eq!(rect.layer, (i + 1) as u32);
            assert_eq!(rect.x, (i as i64) * 200);
            assert_eq!(rect.width, 100 + (i as u64) * 10);
        } else {
            panic!("Expected Rectangle");
        }
    }

    std::fs::remove_file("tests/test_oasis_rects.oas").ok();
}

#[test]
fn test_oasis_polygon_round_trip() {
    let mut oasis = OASISFile::new();
    oasis.names.cell_names.insert(0, "POLY_CELL".to_string());

    let mut cell = OASISCell {
        name: "POLY_CELL".to_string(),
        elements: Vec::new(),
    };

    // Triangle
    cell.elements.push(OASISElement::Polygon(Polygon {
        layer: 2,
        datatype: 0,
        x: 1000,
        y: 2000,
        points: vec![(0, 0), (100, 0), (50, 100), (0, 0)],
        repetition: None,
        properties: Vec::new(),
    }));

    oasis.cells.push(cell);
    oasis.write_to_file("tests/test_oasis_poly.oas").unwrap();

    let oasis_read = OASISFile::read_from_file("tests/test_oasis_poly.oas").unwrap();

    if let OASISElement::Polygon(poly) = &oasis_read.cells[0].elements[0] {
        assert_eq!(poly.layer, 2);
        assert_eq!(poly.x, 1000);
        assert_eq!(poly.y, 2000);
        assert_eq!(poly.points.len(), 4);
        assert_eq!(poly.points[0], (0, 0));
        assert_eq!(poly.points[3], (0, 0)); // Closed
    } else {
        panic!("Expected Polygon");
    }

    std::fs::remove_file("tests/test_oasis_poly.oas").ok();
}

#[test]
fn test_oasis_path_round_trip() {
    let mut oasis = OASISFile::new();
    oasis.names.cell_names.insert(0, "PATH_CELL".to_string());

    let mut cell = OASISCell {
        name: "PATH_CELL".to_string(),
        elements: Vec::new(),
    };

    cell.elements.push(OASISElement::Path(OPath {
        layer: 5,
        datatype: 1,
        x: 500,
        y: 600,
        half_width: 25,
        extension_scheme: ExtensionScheme::HalfWidth,
        points: vec![(0, 0), (100, 0), (100, 100), (200, 100)],
        repetition: None,
        properties: Vec::new(),
    }));

    oasis.cells.push(cell);
    oasis.write_to_file("tests/test_oasis_path.oas").unwrap();

    let oasis_read = OASISFile::read_from_file("tests/test_oasis_path.oas").unwrap();

    if let OASISElement::Path(path) = &oasis_read.cells[0].elements[0] {
        assert_eq!(path.layer, 5);
        assert_eq!(path.datatype, 1);
        assert_eq!(path.x, 500);
        assert_eq!(path.y, 600);
        assert_eq!(path.half_width, 25);
        assert_eq!(path.points.len(), 4);
    } else {
        panic!("Expected Path");
    }

    std::fs::remove_file("tests/test_oasis_path.oas").ok();
}

#[test]
fn test_oasis_mixed_elements() {
    let mut oasis = OASISFile::new();
    oasis.names.cell_names.insert(0, "MIXED".to_string());

    let mut cell = OASISCell {
        name: "MIXED".to_string(),
        elements: Vec::new(),
    };

    // Add one of each type
    cell.elements.push(OASISElement::Rectangle(Rectangle {
        layer: 1,
        datatype: 0,
        x: 0,
        y: 0,
        width: 100,
        height: 100,
        repetition: None,
        properties: Vec::new(),
    }));

    cell.elements.push(OASISElement::Polygon(Polygon {
        layer: 2,
        datatype: 0,
        x: 200,
        y: 200,
        points: vec![(0, 0), (50, 0), (50, 50), (0, 50)],
        repetition: None,
        properties: Vec::new(),
    }));

    cell.elements.push(OASISElement::Path(OPath {
        layer: 3,
        datatype: 0,
        x: 400,
        y: 400,
        half_width: 10,
        extension_scheme: ExtensionScheme::Flush,
        points: vec![(0, 0), (100, 100)],
        repetition: None,
        properties: Vec::new(),
    }));

    oasis.cells.push(cell);
    oasis.write_to_file("tests/test_oasis_mixed.oas").unwrap();

    let oasis_read = OASISFile::read_from_file("tests/test_oasis_mixed.oas").unwrap();

    assert_eq!(oasis_read.cells[0].elements.len(), 3);

    // Verify types in order
    assert!(matches!(
        oasis_read.cells[0].elements[0],
        OASISElement::Rectangle(_)
    ));
    assert!(matches!(
        oasis_read.cells[0].elements[1],
        OASISElement::Polygon(_)
    ));
    assert!(matches!(
        oasis_read.cells[0].elements[2],
        OASISElement::Path(_)
    ));

    std::fs::remove_file("tests/test_oasis_mixed.oas").ok();
}

#[test]
fn test_oasis_empty_cell() {
    let mut oasis = OASISFile::new();
    oasis.names.cell_names.insert(0, "EMPTY".to_string());

    let cell = OASISCell {
        name: "EMPTY".to_string(),
        elements: Vec::new(),
    };

    oasis.cells.push(cell);
    oasis.write_to_file("tests/test_oasis_empty.oas").unwrap();

    let oasis_read = OASISFile::read_from_file("tests/test_oasis_empty.oas").unwrap();
    assert_eq!(oasis_read.cells.len(), 1);
    assert_eq!(oasis_read.cells[0].elements.len(), 0);

    std::fs::remove_file("tests/test_oasis_empty.oas").ok();
}

#[test]
fn test_oasis_large_coordinates() {
    let mut oasis = OASISFile::new();
    oasis.names.cell_names.insert(0, "LARGE".to_string());

    let mut cell = OASISCell {
        name: "LARGE".to_string(),
        elements: Vec::new(),
    };

    // Test with large coordinate values
    cell.elements.push(OASISElement::Rectangle(Rectangle {
        layer: 1,
        datatype: 0,
        x: 1_000_000,
        y: 2_000_000,
        width: 500_000,
        height: 300_000,
        repetition: None,
        properties: Vec::new(),
    }));

    oasis.cells.push(cell);
    oasis.write_to_file("tests/test_oasis_large.oas").unwrap();

    let oasis_read = OASISFile::read_from_file("tests/test_oasis_large.oas").unwrap();

    if let OASISElement::Rectangle(rect) = &oasis_read.cells[0].elements[0] {
        assert_eq!(rect.x, 1_000_000);
        assert_eq!(rect.y, 2_000_000);
        assert_eq!(rect.width, 500_000);
        assert_eq!(rect.height, 300_000);
    } else {
        panic!("Expected Rectangle");
    }

    std::fs::remove_file("tests/test_oasis_large.oas").ok();
}

#[test]
fn test_oasis_negative_coordinates() {
    let mut oasis = OASISFile::new();
    oasis.names.cell_names.insert(0, "NEGATIVE".to_string());

    let mut cell = OASISCell {
        name: "NEGATIVE".to_string(),
        elements: Vec::new(),
    };

    // Test with negative coordinates
    cell.elements.push(OASISElement::Rectangle(Rectangle {
        layer: 1,
        datatype: 0,
        x: -100,
        y: -200,
        width: 50,
        height: 80,
        repetition: None,
        properties: Vec::new(),
    }));

    oasis.cells.push(cell);
    oasis.write_to_file("tests/test_oasis_neg.oas").unwrap();

    let oasis_read = OASISFile::read_from_file("tests/test_oasis_neg.oas").unwrap();

    if let OASISElement::Rectangle(rect) = &oasis_read.cells[0].elements[0] {
        assert_eq!(rect.x, -100);
        assert_eq!(rect.y, -200);
    } else {
        panic!("Expected Rectangle");
    }

    std::fs::remove_file("tests/test_oasis_neg.oas").ok();
}
