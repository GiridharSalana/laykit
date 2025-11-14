// Example demonstrating GDSII functionality (fully working)

use laykit::gdsii::{
    Boundary, GDSElement, GDSIIFile, GDSStructure, GDSTime, GPath, GText, StructRef,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GDSII Library Example ===\n");

    // Create a GDSII file
    println!("1. Creating GDSII file with multiple element types...");
    let gds = create_sample_gdsii();
    gds.write_to_file("example_full.gds")?;
    println!("   ✓ Written to example_full.gds");

    // Read it back
    println!("\n2. Reading GDSII file...");
    let gds_read = GDSIIFile::read_from_file("example_full.gds")?;
    println!("   Library: {}", gds_read.library_name);
    println!("   Version: {}", gds_read.version);
    println!(
        "   Units: {} user, {} database (meters)",
        gds_read.units.0, gds_read.units.1
    );
    println!("   Number of structures: {}", gds_read.structures.len());

    for structure in &gds_read.structures {
        println!("\n   Structure: '{}'", structure.name);
        println!(
            "   Created: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            structure.creation_time.year,
            structure.creation_time.month,
            structure.creation_time.day,
            structure.creation_time.hour,
            structure.creation_time.minute,
            structure.creation_time.second
        );
        println!("   Elements: {}", structure.elements.len());

        for (idx, element) in structure.elements.iter().enumerate() {
            match element {
                GDSElement::Boundary(b) => {
                    println!(
                        "     [{}] Boundary: layer={}, datatype={}, {} points",
                        idx,
                        b.layer,
                        b.datatype,
                        b.xy.len()
                    );
                }
                GDSElement::Path(p) => {
                    println!(
                        "     [{}] Path: layer={}, datatype={}, width={:?}, {} points",
                        idx,
                        p.layer,
                        p.datatype,
                        p.width,
                        p.xy.len()
                    );
                }
                GDSElement::Text(t) => {
                    println!(
                        "     [{}] Text: layer={}, texttype={}, string=\"{}\"",
                        idx, t.layer, t.texttype, t.string
                    );
                }
                GDSElement::StructRef(s) => {
                    println!(
                        "     [{}] StructRef: ref to '{}' at ({}, {})",
                        idx, s.sname, s.xy.0, s.xy.1
                    );
                }
                _ => {
                    println!("     [{}] Other element type", idx);
                }
            }
        }
    }

    println!("\n✅ GDSII operations completed successfully!");
    Ok(())
}

fn create_sample_gdsii() -> GDSIIFile {
    let mut gds = GDSIIFile::new("DEMO_LIBRARY".to_string());
    gds.units = (1e-6, 1e-9); // 1 micron user unit, 1nm database unit

    // Create SUBCELL structure
    let mut subcell = GDSStructure {
        name: "SUBCELL".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
            strclass: None,
        elements: Vec::new(),
    };

    // Add a small rectangle in subcell
    subcell.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![(0, 0), (500, 0), (500, 500), (0, 500), (0, 0)],
            elflags: None,
            plex: None,
        properties: Vec::new(),
    }));

    gds.structures.push(subcell);

    // Create TOP structure
    let mut top = GDSStructure {
        name: "TOP".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
            strclass: None,
        elements: Vec::new(),
    };

    // Add a large rectangle
    top.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![(0, 0), (2000, 0), (2000, 2000), (0, 2000), (0, 0)],
            elflags: None,
            plex: None,
        properties: Vec::new(),
    }));

    // Add a triangle
    top.elements.push(GDSElement::Boundary(Boundary {
        layer: 2,
        datatype: 0,
        xy: vec![(3000, 0), (4000, 0), (3500, 1000), (3000, 0)],
            elflags: None,
            plex: None,
        properties: Vec::new(),
    }));

    // Add a pentagon
    top.elements.push(GDSElement::Boundary(Boundary {
        layer: 3,
        datatype: 0,
        xy: vec![
            (5000, 500),
            (5400, 200),
            (5700, 500),
            (5500, 900),
            (5100, 900),
            (5000, 500),
        ],
            elflags: None,
            plex: None,
        properties: Vec::new(),
    }));

    // Add a path
    top.elements.push(GDSElement::Path(GPath {
        layer: 4,
        datatype: 0,
        pathtype: 0,
        width: Some(100),
            bgnextn: None,
            endextn: None,
        xy: vec![
            (0, 2500),
            (1000, 3000),
            (2000, 2500),
            (3000, 3000),
            (4000, 2500),
        ],
            elflags: None,
            plex: None,
        properties: Vec::new(),
    }));

    // Add text
    top.elements.push(GDSElement::Text(GText {
        layer: 5,
        texttype: 0,
        string: "DEMO LAYOUT".to_string(),
        xy: (1000, 4000),
        presentation: None,
        strans: None,
        width: None,
            elflags: None,
            plex: None,
        properties: Vec::new(),
    }));

    // Add reference to subcell
    top.elements.push(GDSElement::StructRef(StructRef {
        sname: "SUBCELL".to_string(),
        xy: (6000, 0),
        strans: None,
            elflags: None,
            plex: None,
        properties: Vec::new(),
    }));

    gds.structures.push(top);
    gds
}
