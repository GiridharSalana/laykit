// Example demonstrating OASIS functionality (fully working)

use laykit::oasis::{
    ExtensionScheme, OASISCell, OASISElement, OASISFile, OPath, Polygon, Rectangle,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OASIS Library Example ===\n");

    // Create an OASIS file
    println!("1. Creating OASIS file with multiple element types...");
    let oasis = create_sample_oasis();
    oasis.write_to_file("example_full.oas")?;
    println!("   ✓ Written to example_full.oas");

    // Read it back
    println!("\n2. Reading OASIS file...");
    let oasis_read = OASISFile::read_from_file("example_full.oas")?;
    println!("   OASIS Version: {}", oasis_read.version);
    println!("   Number of cells: {}", oasis_read.cells.len());
    println!(
        "   Cell names defined: {}",
        oasis_read.names.cell_names.len()
    );

    for cell in &oasis_read.cells {
        println!("\n   Cell: '{}'", cell.name);
        println!("   Elements: {}", cell.elements.len());

        for (idx, element) in cell.elements.iter().enumerate() {
            match element {
                OASISElement::Rectangle(r) => {
                    println!(
                        "     [{}] Rectangle: layer={}, datatype={}, {}×{} at ({},{})",
                        idx, r.layer, r.datatype, r.width, r.height, r.x, r.y
                    );
                }
                OASISElement::Polygon(p) => {
                    println!(
                        "     [{}] Polygon: layer={}, datatype={}, {} vertices at ({},{})",
                        idx,
                        p.layer,
                        p.datatype,
                        p.points.len(),
                        p.x,
                        p.y
                    );
                }
                OASISElement::Path(path) => {
                    println!(
                        "     [{}] Path: layer={}, datatype={}, half_width={}, {} points",
                        idx,
                        path.layer,
                        path.datatype,
                        path.half_width,
                        path.points.len()
                    );
                }
                OASISElement::Placement(p) => {
                    println!(
                        "     [{}] Placement: ref to '{}' at ({}, {}) [mirror: {}]",
                        idx, p.cell_name, p.x, p.y, p.mirror
                    );
                }
                _ => {
                    println!("     [{}] Other element type", idx);
                }
            }
        }
    }

    println!("\n✅ OASIS operations completed successfully!");
    Ok(())
}

fn create_sample_oasis() -> OASISFile {
    let mut oasis = OASISFile::new();

    // Register cell name
    oasis.names.cell_names.insert(0, "TOP".to_string());

    // Create TOP cell
    let mut top = OASISCell {
        name: "TOP".to_string(),
        name_ref: None,
        elements: Vec::new(),
    };

    // Add a large rectangle
    top.elements.push(OASISElement::Rectangle(Rectangle {
        layer: 1,
        datatype: 0,
        x: 0,
        y: 0,
        width: 2000,
        height: 2000,
        repetition: None,
        properties: Vec::new(),
    }));

    // Add a triangle (polygon)
    top.elements.push(OASISElement::Polygon(Polygon {
        layer: 2,
        datatype: 0,
        x: 3000,
        y: 0,
        points: vec![(0, 0), (1000, 0), (500, 1000), (0, 0)],
        repetition: None,
        properties: Vec::new(),
    }));

    // Add a pentagon (polygon)
    top.elements.push(OASISElement::Polygon(Polygon {
        layer: 3,
        datatype: 0,
        x: 5000,
        y: 500,
        points: vec![
            (0, 0),
            (400, -300),
            (700, 0),
            (500, 400),
            (100, 400),
            (0, 0),
        ],
        repetition: None,
        properties: Vec::new(),
    }));

    // Add a path
    top.elements.push(OASISElement::Path(OPath {
        layer: 4,
        datatype: 0,
        x: 0,
        y: 2500,
        half_width: 50, // Total width = 100
        extension_scheme: ExtensionScheme::HalfWidth,
        points: vec![(0, 0), (1000, 500), (2000, 0), (3000, 500), (4000, 0)],
        repetition: None,
        properties: Vec::new(),
    }));

    // Add another rectangle for variety
    top.elements.push(OASISElement::Rectangle(Rectangle {
        layer: 5,
        datatype: 0,
        x: 1000,
        y: 4000,
        width: 2000,
        height: 500,
        repetition: None,
        properties: Vec::new(),
    }));

    oasis.cells.push(top);
    oasis
}
