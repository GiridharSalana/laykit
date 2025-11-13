# Hierarchical Designs

Learn how to create and work with hierarchical designs using cell references.

## Basic Cell Reference

Create a design with one cell referencing another:

```rust
use laykit::{GDSIIFile, GDSStructure, GDSTime, GDSElement, 
             Boundary, StructRef};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut gds = GDSIIFile::new("HIERARCHICAL".to_string());
    gds.units = (1e-6, 1e-9);
    
    // Create subcell
    let mut subcell = GDSStructure {
        name: "SUBCELL".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    // Add a rectangle to subcell
    subcell.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![(0, 0), (1000, 0), (1000, 1000), (0, 1000), (0, 0)],
        properties: Vec::new(),
    }));
    
    // Create top cell
    let mut topcell = GDSStructure {
        name: "TOPCELL".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    // Reference subcell
    topcell.elements.push(GDSElement::StructRef(StructRef {
        sname: "SUBCELL".to_string(),
        xy: (2000, 2000),
        strans: None,
        properties: Vec::new(),
    }));
    
    gds.structures.push(subcell);
    gds.structures.push(topcell);
    
    gds.write_to_file("hierarchical.gds")?;
    println!("✅ Created hierarchical design");
    
    Ok(())
}
```

## Multiple Instances

Place multiple instances of the same cell:

```rust
use laykit::{GDSIIFile, GDSStructure, GDSTime, GDSElement, 
             Boundary, StructRef};

fn create_repeated_instances() -> Result<(), Box<dyn std::error::Error>> {
    let mut gds = GDSIIFile::new("MULTI_INSTANCE".to_string());
    gds.units = (1e-6, 1e-9);
    
    // Create unit cell
    let mut unit = GDSStructure {
        name: "UNIT".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    unit.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![(0, 0), (500, 0), (500, 500), (0, 500), (0, 0)],
        properties: Vec::new(),
    }));
    
    // Create top cell with multiple instances
    let mut top = GDSStructure {
        name: "TOP".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    // Create a 3x3 grid manually
    for row in 0..3 {
        for col in 0..3 {
            top.elements.push(GDSElement::StructRef(StructRef {
                sname: "UNIT".to_string(),
                xy: (col * 1000, row * 1000),
                strans: None,
                properties: Vec::new(),
            }));
        }
    }
    
    gds.structures.push(unit);
    gds.structures.push(top);
    
    gds.write_to_file("multi_instance.gds")?;
    println!("✅ Created 3×3 array of instances");
    
    Ok(())
}
```

## Array References

Use ArrayRef for efficient array representation:

```rust
use laykit::{GDSIIFile, GDSStructure, GDSTime, GDSElement, 
             Boundary, ArrayRef};

fn create_array() -> Result<(), Box<dyn std::error::Error>> {
    let mut gds = GDSIIFile::new("ARRAY".to_string());
    gds.units = (1e-6, 1e-9);
    
    // Create unit cell
    let mut unit = GDSStructure {
        name: "UNIT".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    unit.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![(0, 0), (500, 0), (500, 500), (0, 500), (0, 0)],
        properties: Vec::new(),
    }));
    
    // Create top cell with array reference
    let mut top = GDSStructure {
        name: "TOP".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    // 10×5 array with 1000nm spacing
    top.elements.push(GDSElement::ArrayRef(ArrayRef {
        sname: "UNIT".to_string(),
        columns: 10,
        rows: 5,
        xy: [
            (0, 0),        // Reference point
            (10000, 0),    // Column extent (10 * 1000)
            (0, 5000),     // Row extent (5 * 1000)
        ],
        strans: None,
        properties: Vec::new(),
    }));
    
    gds.structures.push(unit);
    gds.structures.push(top);
    
    gds.write_to_file("array.gds")?;
    println!("✅ Created 10×5 array");
    
    Ok(())
}
```

## Transformations

Apply transformations to cell instances:

```rust
use laykit::{GDSIIFile, GDSStructure, GDSTime, GDSElement, 
             Boundary, StructRef, STrans};

fn create_transformed_instances() -> Result<(), Box<dyn std::error::Error>> {
    let mut gds = GDSIIFile::new("TRANSFORMED".to_string());
    gds.units = (1e-6, 1e-9);
    
    // Create base cell
    let mut base = GDSStructure {
        name: "BASE".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    base.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![(0, 0), (1000, 0), (1000, 500), (0, 500), (0, 0)],
        properties: Vec::new(),
    }));
    
    let mut top = GDSStructure {
        name: "TOP".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    // Original instance
    top.elements.push(GDSElement::StructRef(StructRef {
        sname: "BASE".to_string(),
        xy: (0, 0),
        strans: None,
        properties: Vec::new(),
    }));
    
    // Rotated 90 degrees
    top.elements.push(GDSElement::StructRef(StructRef {
        sname: "BASE".to_string(),
        xy: (3000, 0),
        strans: Some(STrans {
            reflect_x: false,
            absolute_mag: false,
            absolute_angle: false,
            magnification: None,
            angle: Some(90.0),
        }),
        properties: Vec::new(),
    }));
    
    // Mirrored
    top.elements.push(GDSElement::StructRef(StructRef {
        sname: "BASE".to_string(),
        xy: (6000, 0),
        strans: Some(STrans {
            reflect_x: true,
            absolute_mag: false,
            absolute_angle: false,
            magnification: None,
            angle: None,
        }),
        properties: Vec::new(),
    }));
    
    // Scaled 2x
    top.elements.push(GDSElement::StructRef(StructRef {
        sname: "BASE".to_string(),
        xy: (9000, 0),
        strans: Some(STrans {
            reflect_x: false,
            absolute_mag: false,
            absolute_angle: false,
            magnification: Some(2.0),
            angle: None,
        }),
        properties: Vec::new(),
    }));
    
    gds.structures.push(base);
    gds.structures.push(top);
    
    gds.write_to_file("transformed.gds")?;
    println!("✅ Created transformed instances");
    
    Ok(())
}
```

## Multi-Level Hierarchy

Create a 3-level hierarchy:

```rust
use laykit::{GDSIIFile, GDSStructure, GDSTime, GDSElement, 
             Boundary, StructRef};

fn create_multilevel() -> Result<(), Box<dyn std::error::Error>> {
    let mut gds = GDSIIFile::new("MULTILEVEL".to_string());
    gds.units = (1e-6, 1e-9);
    
    // Level 1: Basic shape
    let mut l1_cell = GDSStructure {
        name: "L1_BASIC".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    l1_cell.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
        properties: Vec::new(),
    }));
    
    // Level 2: Group of L1 cells
    let mut l2_cell = GDSStructure {
        name: "L2_GROUP".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    for i in 0..4 {
        l2_cell.elements.push(GDSElement::StructRef(StructRef {
            sname: "L1_BASIC".to_string(),
            xy: (i * 200, 0),
            strans: None,
            properties: Vec::new(),
        }));
    }
    
    // Level 3: Top level with L2 cells
    let mut l3_cell = GDSStructure {
        name: "L3_TOP".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    for i in 0..3 {
        l3_cell.elements.push(GDSElement::StructRef(StructRef {
            sname: "L2_GROUP".to_string(),
            xy: (0, i * 500),
            strans: None,
            properties: Vec::new(),
        }));
    }
    
    gds.structures.push(l1_cell);
    gds.structures.push(l2_cell);
    gds.structures.push(l3_cell);
    
    gds.write_to_file("multilevel.gds")?;
    println!("✅ Created 3-level hierarchy");
    
    Ok(())
}
```

## Finding Cell Dependencies

Analyze cell reference relationships:

```rust
use laykit::{GDSIIFile, GDSElement};
use std::collections::{HashMap, HashSet};

fn analyze_hierarchy(gds: &GDSIIFile) {
    let mut references: HashMap<String, HashSet<String>> = HashMap::new();
    
    // Build reference map
    for structure in &gds.structures {
        let mut refs = HashSet::new();
        
        for element in &structure.elements {
            match element {
                GDSElement::StructRef(sref) => {
                    refs.insert(sref.sname.clone());
                }
                GDSElement::ArrayRef(aref) => {
                    refs.insert(aref.sname.clone());
                }
                _ => {}
            }
        }
        
        references.insert(structure.name.clone(), refs);
    }
    
    // Print hierarchy
    println!("Cell hierarchy:");
    for (cell, refs) in &references {
        if !refs.is_empty() {
            println!("  {} references:", cell);
            for ref_name in refs {
                println!("    - {}", ref_name);
            }
        }
    }
    
    // Find top cells (not referenced by others)
    let all_refs: HashSet<_> = references.values()
        .flat_map(|refs| refs.iter().cloned())
        .collect();
    
    let top_cells: Vec<_> = references.keys()
        .filter(|name| !all_refs.contains(*name))
        .collect();
    
    println!("\nTop-level cells:");
    for cell in top_cells {
        println!("  - {}", cell);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gds = GDSIIFile::read_from_file("design.gds")?;
    analyze_hierarchy(&gds);
    Ok(())
}
```

## Flattening Hierarchy

Flatten a hierarchical design to a single level:

```rust
// Note: This is a simplified example
// Full flattening requires resolving all transformations

use laykit::{GDSIIFile, GDSStructure, GDSTime, GDSElement};

fn flatten_simple(gds: &GDSIIFile, top_cell_name: &str) 
    -> Result<GDSStructure, Box<dyn std::error::Error>> 
{
    let mut flattened = GDSStructure {
        name: format!("{}_FLAT", top_cell_name),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    // Find top cell
    let top_cell = gds.structures.iter()
        .find(|s| s.name == top_cell_name)
        .ok_or("Top cell not found")?;
    
    // Copy non-reference elements
    for element in &top_cell.elements {
        match element {
            GDSElement::Boundary(_) | 
            GDSElement::Path(_) | 
            GDSElement::Text(_) => {
                flattened.elements.push(element.clone());
            }
            _ => {
                // References would need transformation and recursion
                println!("Warning: Skipping reference (not implemented)");
            }
        }
    }
    
    Ok(flattened)
}
```
