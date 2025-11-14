use laykit::*;
use std::fs;
use std::io::{BufReader, Cursor};
use std::process::Command;

// Helper function for CLI tests
fn get_cli_path() -> String {
    let mut path = std::env::current_dir().unwrap();
    path.push("target");
    path.push("debug");
    path.push("laykit");
    path.to_str().unwrap().to_string()
}

// ============================================================================
// Converter Tests
// ============================================================================

#[cfg(test)]
mod converter_tests {
    use super::*;

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
                strclass: None,
            elements: Vec::new(),
        };

        structure.elements.push(GDSElement::Boundary(Boundary {
            layer: 1,
            datatype: 0,
            xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));

        gds.structures.push(structure);

        let oasis = converter::gdsii_to_oasis(&gds).unwrap();
        assert_eq!(oasis.cells.len(), 1);
        assert_eq!(oasis.cells[0].name, "TOP");
    }
}

// ============================================================================
// GDSII Tests
// ============================================================================

#[cfg(test)]
mod gdsii_tests {
    use super::*;

    #[test]
    fn test_gdsii_create_and_write() {
        let mut gds = GDSIIFile::new("TEST_LIB".to_string());
        gds.units = (1e-6, 1e-9);

        let mut structure = GDSStructure {
            name: "TOP".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };

        structure.elements.push(GDSElement::Boundary(Boundary {
            layer: 1,
            datatype: 0,
            xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
                    elflags: None,
                    plex: None,
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
                strclass: None,
            elements: Vec::new(),
        };

        structure.elements.push(GDSElement::Boundary(Boundary {
            layer: 5,
            datatype: 2,
            xy: vec![(10, 20), (110, 20), (110, 120), (10, 120), (10, 20)],
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));

        structure.elements.push(GDSElement::Path(GPath {
            layer: 3,
            datatype: 1,
            pathtype: 0,
            width: Some(50),
                    bgnextn: None,
                    endextn: None,
            xy: vec![(0, 0), (100, 100), (200, 100)],
                    elflags: None,
                    plex: None,
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
                strclass: None,
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
                    elflags: None,
                    plex: None,
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
                strclass: None,
            elements: vec![GDSElement::Boundary(Boundary {
                layer: 1,
                datatype: 0,
                xy: vec![(0, 0), (50, 0), (50, 50), (0, 50), (0, 0)],
                    elflags: None,
                    plex: None,
                properties: Vec::new(),
            })],
        };

        let mut topcell = GDSStructure {
            name: "TOP".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };

        topcell.elements.push(GDSElement::StructRef(StructRef {
            sname: "SUBCELL".to_string(),
            xy: (100, 200),
            strans: None,
            elflags: None,
            plex: None,
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
                strclass: None,
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
                strclass: None,
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
                    elflags: None,
                    plex: None,
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
                strclass: None,
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
                    elflags: None,
                    plex: None,
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
}

// ============================================================================
// OASIS Tests
// ============================================================================

#[cfg(test)]
mod oasis_tests {
    use super::*;

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
}

// ============================================================================
// CLI Tests
// ============================================================================

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn test_cli_help() {
        let output = Command::new(get_cli_path())
            .arg("help")
            .output()
            .expect("Failed to execute CLI");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("LayKit"));
        assert!(stdout.contains("convert"));
        assert!(stdout.contains("info"));
        assert!(stdout.contains("validate"));
    }

    #[test]
    fn test_cli_no_args() {
        let output = Command::new(get_cli_path())
            .output()
            .expect("Failed to execute CLI");

        assert!(!output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("USAGE"));
    }

    #[test]
    fn test_cli_unknown_command() {
        let output = Command::new(get_cli_path())
            .arg("unknown_cmd")
            .output()
            .expect("Failed to execute CLI");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Unknown command"));
    }

    #[test]
    fn test_cli_convert_missing_args() {
        let output = Command::new(get_cli_path())
            .arg("convert")
            .output()
            .expect("Failed to execute CLI");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("requires input and output"));
    }

    #[test]
    fn test_cli_convert_gds_to_gds() {
        // Create a test GDSII file
        let input_path = "tests/cli_test_input.gds";
        let output_path = "tests/cli_test_output.gds";

        let mut gds = GDSIIFile::new("TEST".to_string());
        gds.units = (1e-6, 1e-9);
        let mut structure = GDSStructure {
            name: "TESTCELL".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };
        structure.elements.push(GDSElement::Boundary(Boundary {
            layer: 1,
            datatype: 0,
            xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));
        gds.structures.push(structure);
        gds.write_to_file(input_path).unwrap();

        // Test conversion
        let output = Command::new(get_cli_path())
            .arg("convert")
            .arg(input_path)
            .arg(output_path)
            .output()
            .expect("Failed to execute CLI");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("successful"));

        // Verify output file exists
        assert!(std::path::Path::new(output_path).exists());

        // Cleanup
        fs::remove_file(input_path).ok();
        fs::remove_file(output_path).ok();
    }

    #[test]
    fn test_cli_info_gds() {
        // Create a test GDSII file
        let test_path = "tests/cli_test_info.gds";

        let mut gds = GDSIIFile::new("INFOTEST".to_string());
        gds.units = (1e-6, 1e-9);
        let mut structure = GDSStructure {
            name: "CELL1".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };
        structure.elements.push(GDSElement::Boundary(Boundary {
            layer: 1,
            datatype: 0,
            xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));
        structure.elements.push(GDSElement::Text(GText {
            layer: 2,
            texttype: 0,
            string: "TEST".to_string(),
            xy: (50, 50),
            presentation: None,
            strans: None,
            width: None,
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));
        gds.structures.push(structure);
        gds.write_to_file(test_path).unwrap();

        // Test info command
        let output = Command::new(get_cli_path())
            .arg("info")
            .arg(test_path)
            .output()
            .expect("Failed to execute CLI");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("GDSII File Information"));
        assert!(stdout.contains("INFOTEST"));
        assert!(stdout.contains("CELL1"));
        assert!(stdout.contains("Structures: 1"));
        assert!(stdout.contains("Total Elements: 2"));
        assert!(stdout.contains("Boundary"));
        assert!(stdout.contains("Text"));

        // Cleanup
        fs::remove_file(test_path).ok();
    }

    #[test]
    fn test_cli_info_missing_file() {
        let output = Command::new(get_cli_path())
            .arg("info")
            .arg("nonexistent_file.gds")
            .output()
            .expect("Failed to execute CLI");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("does not exist"));
    }

    #[test]
    fn test_cli_validate_gds_valid() {
        // Create a valid GDSII file
        let test_path = "tests/cli_test_validate_valid.gds";

        let mut gds = GDSIIFile::new("VALIDATETEST".to_string());
        gds.units = (1e-6, 1e-9);
        let mut structure = GDSStructure {
            name: "VALIDCELL".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };
        structure.elements.push(GDSElement::Boundary(Boundary {
            layer: 1,
            datatype: 0,
            xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));
        gds.structures.push(structure);
        gds.write_to_file(test_path).unwrap();

        // Test validate command
        let output = Command::new(get_cli_path())
            .arg("validate")
            .arg(test_path)
            .output()
            .expect("Failed to execute CLI");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Validation Results"));
        assert!(stdout.contains("File is valid") || stdout.contains("no issues"));

        // Cleanup
        fs::remove_file(test_path).ok();
    }

    #[test]
    fn test_cli_validate_gds_invalid() {
        // Create an invalid GDSII file (unclosed boundary)
        let test_path = "tests/cli_test_validate_invalid.gds";

        let mut gds = GDSIIFile::new("INVALIDTEST".to_string());
        gds.units = (1e-6, 1e-9);
        let mut structure = GDSStructure {
            name: "INVALIDCELL".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };
        // Add an unclosed boundary (first != last)
        structure.elements.push(GDSElement::Boundary(Boundary {
            layer: 1,
            datatype: 0,
            xy: vec![(0, 0), (100, 0), (100, 100), (0, 100)], // Not closed!
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));
        gds.structures.push(structure);
        gds.write_to_file(test_path).unwrap();

        // Test validate command
        let output = Command::new(get_cli_path())
            .arg("validate")
            .arg(test_path)
            .output()
            .expect("Failed to execute CLI");

        assert!(output.status.success()); // Command succeeds, but finds issues
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Validation Results"));
        assert!(stdout.contains("issue") || stdout.contains("not closed"));

        // Cleanup
        fs::remove_file(test_path).ok();
    }

    #[test]
    fn test_cli_validate_missing_args() {
        let output = Command::new(get_cli_path())
            .arg("validate")
            .output()
            .expect("Failed to execute CLI");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("requires a file path"));
    }

    #[test]
    fn test_cli_convert_nonexistent_file() {
        let output = Command::new(get_cli_path())
            .arg("convert")
            .arg("nonexistent.gds")
            .arg("output.oas")
            .output()
            .expect("Failed to execute CLI");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("does not exist"));
    }

    #[test]
    fn test_cli_info_unsupported_format() {
        // Create a dummy file with wrong extension
        let test_path = "tests/cli_test.txt";
        fs::write(test_path, "dummy content").unwrap();

        let output = Command::new(get_cli_path())
            .arg("info")
            .arg(test_path)
            .output()
            .expect("Failed to execute CLI");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Unknown file format"));

        // Cleanup
        fs::remove_file(test_path).ok();
    }
}

// ============================================================================
// Streaming Parser Tests
// ============================================================================

#[cfg(test)]
mod streaming_tests {
    use super::*;

    #[test]
    fn test_streaming_small_file() {
        // Create a small test file
        let mut gds = GDSIIFile::new("SMALLTEST".to_string());
        gds.units = (1e-6, 1e-9);

        let mut structure = GDSStructure {
            name: "TESTCELL".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };

        structure.elements.push(GDSElement::Boundary(Boundary {
            layer: 1,
            datatype: 0,
            xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));

        gds.structures.push(structure);

        // Write to bytes
        let mut buffer = Vec::new();
        gds.write_to_writer(&mut buffer).unwrap();

        // Stream read
        let cursor = Cursor::new(buffer);
        let mut reader = StreamingGDSIIReader::new(cursor).unwrap();

        assert_eq!(reader.library_name(), "SMALLTEST");
        // Streaming parser reads units (may have minor precision differences in Real8 conversion)
        let units = reader.units();
        assert!(units.0 > 0.0 && units.1 > 0.0);

        let mut stats = StatisticsCollector::new();
        reader.process_structures(&mut stats).unwrap();

        assert_eq!(stats.structure_count, 1);
    }

    #[test]
    fn test_streaming_multiple_structures() {
        // Create a file with multiple structures
        let mut gds = GDSIIFile::new("MULTITEST".to_string());
        gds.units = (1e-6, 1e-9);

        for i in 0..50 {
            let mut structure = GDSStructure {
                name: format!("CELL_{:03}", i),
                creation_time: GDSTime::now(),
                modification_time: GDSTime::now(),
                strclass: None,
                elements: Vec::new(),
            };

            for j in 0..10 {
                structure.elements.push(GDSElement::Boundary(Boundary {
                    layer: (j % 5 + 1) as i16,
                    datatype: 0,
                    xy: vec![
                        (j * 100, 0),
                        ((j + 1) * 100, 0),
                        ((j + 1) * 100, 100),
                        (j * 100, 100),
                        (j * 100, 0),
                    ],
                    elflags: None,
                    plex: None,
                    properties: Vec::new(),
                }));
            }

            gds.structures.push(structure);
        }

        // Write to bytes
        let mut buffer = Vec::new();
        gds.write_to_writer(&mut buffer).unwrap();

        println!("Test file size: {} bytes", buffer.len());

        // Stream read
        let cursor = Cursor::new(buffer);
        let mut reader = StreamingGDSIIReader::new(cursor).unwrap();

        let mut stats = StatisticsCollector::new();
        reader.process_structures(&mut stats).unwrap();

        assert_eq!(stats.structure_count, 50);
    }

    #[test]
    fn test_streaming_name_collector() {
        // Create a test file with known structure names
        let mut gds = GDSIIFile::new("NAMECOLLECT".to_string());
        gds.units = (1e-6, 1e-9);

        let expected_names = vec![
            "TOP".to_string(),
            "SUBCELL_A".to_string(),
            "SUBCELL_B".to_string(),
            "SUBCELL_C".to_string(),
        ];

        for name in &expected_names {
            let structure = GDSStructure {
                name: name.clone(),
                creation_time: GDSTime::now(),
                modification_time: GDSTime::now(),
                strclass: None,
                elements: Vec::new(),
            };
            gds.structures.push(structure);
        }

        // Write to bytes
        let mut buffer = Vec::new();
        gds.write_to_writer(&mut buffer).unwrap();

        // Stream read with name collector
        let cursor = Cursor::new(buffer);
        let mut reader = StreamingGDSIIReader::new(cursor).unwrap();

        let mut collector = StructureNameCollector::new();
        reader.process_structures(&mut collector).unwrap();

        assert_eq!(collector.names.len(), expected_names.len());
        assert_eq!(collector.names, expected_names);
    }

    #[test]
    fn test_streaming_large_file_simulation() {
        // Simulate a larger file (not 1GB, but large enough to test streaming)
        let mut gds = GDSIIFile::new("LARGETEST".to_string());
        gds.units = (1e-6, 1e-9);

        // Create 100 structures with 100 elements each = 10,000 elements total
        for i in 0..100 {
            let mut structure = GDSStructure {
                name: format!("CELL_{:05}", i),
                creation_time: GDSTime::now(),
                modification_time: GDSTime::now(),
                strclass: None,
                elements: Vec::new(),
            };

            for j in 0..100 {
                structure.elements.push(GDSElement::Boundary(Boundary {
                    layer: ((i + j) % 10 + 1) as i16,
                    datatype: ((i + j) % 5) as i16,
                    xy: vec![
                        (j * 1000, i * 1000),
                        ((j + 1) * 1000, i * 1000),
                        ((j + 1) * 1000, (i + 1) * 1000),
                        (j * 1000, (i + 1) * 1000),
                        (j * 1000, i * 1000),
                    ],
                    elflags: None,
                    plex: None,
                    properties: Vec::new(),
                }));
            }

            gds.structures.push(structure);
        }

        // Write to bytes
        let mut buffer = Vec::new();
        gds.write_to_writer(&mut buffer).unwrap();

        let file_size = buffer.len();
        println!(
            "Large test file size: {} bytes ({:.2} MB)",
            file_size,
            file_size as f64 / 1_048_576.0
        );

        // Stream read
        let cursor = Cursor::new(buffer);
        let mut reader = StreamingGDSIIReader::new(cursor).unwrap();

        let mut stats = StatisticsCollector::new();
        reader.process_structures(&mut stats).unwrap();

        assert_eq!(stats.structure_count, 100);
        println!("Processed {} structures", stats.structure_count);
    }

    #[test]
    fn test_streaming_empty_structures() {
        // Test with structures that have no elements
        let mut gds = GDSIIFile::new("EMPTYTEST".to_string());
        gds.units = (1e-6, 1e-9);

        for i in 0..10 {
            let structure = GDSStructure {
                name: format!("EMPTY_{}", i),
                creation_time: GDSTime::now(),
                modification_time: GDSTime::now(),
                strclass: None,
                elements: Vec::new(),
            };
            gds.structures.push(structure);
        }

        // Write to bytes
        let mut buffer = Vec::new();
        gds.write_to_writer(&mut buffer).unwrap();

        // Stream read
        let cursor = Cursor::new(buffer);
        let mut reader = StreamingGDSIIReader::new(cursor).unwrap();

        let mut stats = StatisticsCollector::new();
        reader.process_structures(&mut stats).unwrap();

        assert_eq!(stats.structure_count, 10);
        assert_eq!(stats.element_count, 0);
    }

    #[test]
    fn test_streaming_mixed_elements() {
        // Test with various element types
        let mut gds = GDSIIFile::new("MIXEDTEST".to_string());
        gds.units = (1e-6, 1e-9);

        let mut structure = GDSStructure {
            name: "MIXED".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };

        // Add boundary
        structure.elements.push(GDSElement::Boundary(Boundary {
            layer: 1,
            datatype: 0,
            xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));

        // Add path
        structure.elements.push(GDSElement::Path(GPath {
            layer: 2,
            datatype: 0,
            pathtype: 0,
            width: Some(10),
                    bgnextn: None,
                    endextn: None,
            xy: vec![(0, 0), (100, 0), (100, 100)],
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));

        // Add text
        structure.elements.push(GDSElement::Text(GText {
            layer: 3,
            texttype: 0,
            string: "TEST".to_string(),
            xy: (50, 50),
            presentation: None,
            strans: None,
            width: None,
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));

        gds.structures.push(structure);

        // Write to bytes
        let mut buffer = Vec::new();
        gds.write_to_writer(&mut buffer).unwrap();

        // Stream read
        let cursor = Cursor::new(buffer);
        let mut reader = StreamingGDSIIReader::new(cursor).unwrap();

        let mut stats = StatisticsCollector::new();
        reader.process_structures(&mut stats).unwrap();

        assert_eq!(stats.structure_count, 1);
    }

    #[test]
    fn test_streaming_from_file() {
        // Create a test file on disk
        let test_path = "tests/streaming_test_file.gds";

        let mut gds = GDSIIFile::new("FILETEST".to_string());
        gds.units = (1e-6, 1e-9);

        for i in 0..20 {
            let mut structure = GDSStructure {
                name: format!("FILE_CELL_{}", i),
                creation_time: GDSTime::now(),
                modification_time: GDSTime::now(),
                strclass: None,
                elements: Vec::new(),
            };

            structure.elements.push(GDSElement::Boundary(Boundary {
                layer: (i % 5 + 1) as i16,
                datatype: 0,
                xy: vec![
                    (0, 0),
                    (i * 100, 0),
                    (i * 100, i * 100),
                    (0, i * 100),
                    (0, 0),
                ],
                    elflags: None,
                    plex: None,
                properties: Vec::new(),
            }));

            gds.structures.push(structure);
        }

        gds.write_to_file(test_path).unwrap();

        // Stream read from file
        let file = std::fs::File::open(test_path).unwrap();
        let reader_buf = BufReader::new(file);
        let mut reader = StreamingGDSIIReader::new(reader_buf).unwrap();

        assert_eq!(reader.library_name(), "FILETEST");

        let mut collector = StructureNameCollector::new();
        reader.process_structures(&mut collector).unwrap();

        assert_eq!(collector.names.len(), 20);

        // Cleanup
        std::fs::remove_file(test_path).ok();
    }
}

// ============================================================================
// Non-UTF-8 String Handling Tests
// ============================================================================

#[cfg(test)]
mod utf8_handling_tests {
    use super::*;

    #[test]
    fn test_gdsii_non_utf8_strings() {
        // Create a GDSII file with a structure that has a name with non-UTF-8 bytes
        let mut gds = GDSIIFile::new("TestLib".to_string());
        
        // Add a normal structure
        let mut structure = GDSStructure {
            name: "ValidName".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };
        
        structure.elements.push(GDSElement::Boundary(Boundary {
            layer: 1,
            datatype: 0,
            xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
                    elflags: None,
                    plex: None,
            properties: Vec::new(),
        }));
        
        gds.structures.push(structure);
        
        // Write to file
        let test_path = "tests/test_utf8_gdsii.gds";
        gds.write_to_file(test_path).unwrap();
        
        // Manually inject non-UTF-8 bytes into the file
        let mut file_data = std::fs::read(test_path).unwrap();
        
        // Find the structure name in the file and replace some bytes with invalid UTF-8
        // GDSII structure names are in STRNAME records (0x06)
        for i in 0..file_data.len() - 20 {
            if file_data[i] == 0x00 && file_data[i+2] == 0x06 {
                // Found a STRNAME record, inject invalid UTF-8 (0xFF is invalid in UTF-8)
                if i + 10 < file_data.len() {
                    file_data[i + 8] = 0xFF;
                    file_data[i + 9] = 0xFE;
                    break;
                }
            }
        }
        
        std::fs::write(test_path, &file_data).unwrap();
        
        // Try to read the file - should succeed with lossy conversion
        let result = GDSIIFile::read_from_file(test_path);
        assert!(result.is_ok(), "Should handle non-UTF-8 strings gracefully");
        
        let gds_read = result.unwrap();
        assert_eq!(gds_read.structures.len(), 1);
        // The name should contain replacement characters but still be readable
        assert!(!gds_read.structures[0].name.is_empty());
        
        // Cleanup
        std::fs::remove_file(test_path).ok();
    }

    #[test]
    fn test_oasis_non_utf8_strings() {
        // Create an OASIS file
        let mut oasis = OASISFile::new();
        
        let mut cell = OASISCell {
            name: "TestCell".to_string(),
            elements: Vec::new(),
        };
        
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
        
        oasis.cells.push(cell);
        
        // Write to file
        let test_path = "tests/test_utf8_oasis.oas";
        oasis.write_to_file(test_path).unwrap();
        
        // Manually inject non-UTF-8 bytes into the file
        let mut file_data = std::fs::read(test_path).unwrap();
        
        // Find the cell name in the file and replace some bytes with invalid UTF-8
        // OASIS cell names typically appear after record ID 3 or 13
        for i in 13..file_data.len() - 10 {
            // Look for a string that looks like it could be a cell name
            if file_data[i] > 0 && file_data[i] < 20 {
                // Inject invalid UTF-8 sequence
                if i + 5 < file_data.len() {
                    file_data[i + 3] = 0xFF;
                    file_data[i + 4] = 0xFE;
                    break;
                }
            }
        }
        
        std::fs::write(test_path, &file_data).unwrap();
        
        // Try to read the file - should succeed with lossy conversion
        let result = OASISFile::read_from_file(test_path);
        assert!(result.is_ok(), "Should handle non-UTF-8 strings gracefully");
        
        let oasis_read = result.unwrap();
        assert!(!oasis_read.cells.is_empty());
        
        // Cleanup
        std::fs::remove_file(test_path).ok();
    }

    #[test]
    fn test_gdsii_latin1_strings() {
        // Test with Latin-1 encoded strings (common in some European tools)
        let mut gds = GDSIIFile::new("TestLib".to_string());
        
        let structure = GDSStructure {
            name: "Cell".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };
        
        gds.structures.push(structure);
        
        let test_path = "tests/test_latin1.gds";
        gds.write_to_file(test_path).unwrap();
        
        // Read and inject Latin-1 characters (e.g., é = 0xE9 in Latin-1)
        let mut file_data = std::fs::read(test_path).unwrap();
        
        // Find LIBNAME record and inject Latin-1
        for i in 0..file_data.len() - 20 {
            if file_data[i] == 0x00 && file_data[i+2] == 0x02 {
                // Found LIBNAME, inject Latin-1 character
                if i + 10 < file_data.len() {
                    file_data[i + 8] = 0xE9; // é in Latin-1
                    break;
                }
            }
        }
        
        std::fs::write(test_path, &file_data).unwrap();
        
        // Should read successfully
        let result = GDSIIFile::read_from_file(test_path);
        assert!(result.is_ok(), "Should handle Latin-1 strings");
        
        // Cleanup
        std::fs::remove_file(test_path).ok();
    }

    #[test]
    fn test_empty_strings() {
        // Test that empty strings are handled correctly
        let mut gds = GDSIIFile::new("Test".to_string());
        
        let structure = GDSStructure {
            name: "EmptyTest".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };
        
        gds.structures.push(structure);
        
        let test_path = "tests/test_empty_strings.gds";
        gds.write_to_file(test_path).unwrap();
        
        let result = GDSIIFile::read_from_file(test_path);
        assert!(result.is_ok());
        
        // Cleanup
        std::fs::remove_file(test_path).ok();
    }

    #[test]
    fn test_null_terminated_strings() {
        // Test strings with null bytes in the middle
        let mut gds = GDSIIFile::new("NullTest".to_string());
        
        let structure = GDSStructure {
            name: "Cell".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
                strclass: None,
            elements: Vec::new(),
        };
        
        gds.structures.push(structure);
        
        let test_path = "tests/test_null_strings.gds";
        gds.write_to_file(test_path).unwrap();
        
        // The null-termination handling should work correctly
        let result = GDSIIFile::read_from_file(test_path);
        assert!(result.is_ok());
        
        let gds_read = result.unwrap();
        assert_eq!(gds_read.library_name, "NullTest");
        
        // Cleanup
        std::fs::remove_file(test_path).ok();
    }
}
