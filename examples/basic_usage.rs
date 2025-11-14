// Example demonstrating basic usage of the laykit library

use laykit::converter::{gdsii_to_oasis, oasis_to_gdsii};
use laykit::gdsii::{Boundary, GDSElement, GDSIIFile, GDSStructure, GDSTime, GPath};
use laykit::oasis::{OASISCell, OASISElement, OASISFile, Polygon, Rectangle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== laykit Library Example ===\n");

    // Create a GDSII file
    println!("1. Creating GDSII file...");
    let gds = create_sample_gdsii();
    gds.write_to_file("example.gds")?;
    println!("   ✓ Written to example.gds");

    // Create an OASIS file
    println!("\n2. Creating OASIS file...");
    let oasis = create_sample_oasis();
    oasis.write_to_file("example.oas")?;
    println!("   ✓ Written to example.oas");

    // Convert GDSII to OASIS
    println!("\n3. Converting GDSII to OASIS...");
    let gds = GDSIIFile::read_from_file("example.gds")?;
    println!("   Read GDSII: {} structures", gds.structures.len());
    let converted_oasis = gdsii_to_oasis(&gds)?;
    converted_oasis.write_to_file("converted_from_gds.oas")?;
    println!("   ✓ Converted to converted_from_gds.oas");

    // Convert OASIS to GDSII
    println!("\n4. Converting OASIS to GDSII...");
    let oasis = OASISFile::read_from_file("example.oas")?;
    println!("   Read OASIS: {} cells", oasis.cells.len());
    let converted_gds = oasis_to_gdsii(&oasis)?;
    converted_gds.write_to_file("converted_from_oas.gds")?;
    println!("   ✓ Converted to converted_from_oas.gds");

    // Read and display info
    println!("\n5. Reading back files...");
    let gds = GDSIIFile::read_from_file("example.gds")?;
    println!("   GDSII Library: {}", gds.library_name);
    println!("   Units: {} user, {} database", gds.units.0, gds.units.1);
    for structure in &gds.structures {
        println!(
            "   Structure '{}': {} elements",
            structure.name,
            structure.elements.len()
        );
    }

    let oasis = OASISFile::read_from_file("example.oas")?;
    println!("\n   OASIS Version: {}", oasis.version);
    println!("   Unit: {} meters", oasis.unit);
    for cell in &oasis.cells {
        println!("   Cell '{}': {} elements", cell.name, cell.elements.len());
    }

    println!("\n✅ All operations completed successfully!");
    Ok(())
}

fn create_sample_gdsii() -> GDSIIFile {
    let mut gds = GDSIIFile::new("SAMPLE_LIB".to_string());
    gds.units = (1e-6, 1e-9); // 1 micron user unit, 1nm database unit

    let mut structure = GDSStructure {
        name: "TOP".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        strclass: None,
        elements: Vec::new(),
    };

    // Add a square boundary
    structure.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![(0, 0), (1000, 0), (1000, 1000), (0, 1000), (0, 0)],
        elflags: None,
        plex: None,
        properties: Vec::new(),
    }));

    // Add a triangular boundary
    structure.elements.push(GDSElement::Boundary(Boundary {
        layer: 2,
        datatype: 0,
        xy: vec![(2000, 0), (3000, 0), (2500, 1000), (2000, 0)],
        elflags: None,
        plex: None,
        properties: Vec::new(),
    }));

    // Add a path
    structure.elements.push(GDSElement::Path(GPath {
        layer: 3,
        datatype: 0,
        pathtype: 0,
        width: Some(100),
        bgnextn: None,
        endextn: None,
        xy: vec![(0, 1500), (500, 2000), (1000, 1500), (1500, 2000)],
        elflags: None,
        plex: None,
        properties: Vec::new(),
    }));

    gds.structures.push(structure);
    gds
}

fn create_sample_oasis() -> OASISFile {
    let mut oasis = OASISFile::new();
    oasis.unit = 1e-9; // 1nm database unit

    oasis.names.cell_names.insert(0, "TOP".to_string());

    let mut cell = OASISCell {
        name: "TOP".to_string(),
        elements: Vec::new(),
    };

    // Add rectangles
    cell.elements.push(OASISElement::Rectangle(Rectangle {
        layer: 1,
        datatype: 0,
        x: 0,
        y: 0,
        width: 1000,
        height: 1000,
        repetition: None,
        properties: Vec::new(),
    }));

    cell.elements.push(OASISElement::Rectangle(Rectangle {
        layer: 1,
        datatype: 0,
        x: 1500,
        y: 0,
        width: 500,
        height: 2000,
        repetition: None,
        properties: Vec::new(),
    }));

    // Add a polygon
    cell.elements.push(OASISElement::Polygon(Polygon {
        layer: 2,
        datatype: 0,
        x: 0,
        y: 2000,
        points: vec![(0, 0), (1000, 0), (500, 500), (0, 0)],
        repetition: None,
        properties: Vec::new(),
    }));

    oasis.cells.push(cell);
    oasis
}
