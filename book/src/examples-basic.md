# Basic Usage Examples

Simple examples to get you started with LayKit.

## Reading a GDSII File

The most basic operation - reading and displaying information:

```rust
use laykit::GDSIIFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gds = GDSIIFile::read_from_file("layout.gds")?;
    
    println!("Library: {}", gds.library_name);
    println!("Structures: {}", gds.structures.len());
    
    for structure in &gds.structures {
        println!("  - {} ({} elements)", 
            structure.name, structure.elements.len());
    }
    
    Ok(())
}
```

## Reading an OASIS File

Similar to GDSII but with different structure:

```rust
use laykit::OASISFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oasis = OASISFile::read_from_file("layout.oas")?;
    
    println!("Version: {}", oasis.version);
    println!("Cells: {}", oasis.cells.len());
    
    for cell in &oasis.cells {
        println!("  - {} ({} elements)", 
            cell.name, cell.elements.len());
    }
    
    Ok(())
}
```

## Creating a Simple GDSII File

Create a file with a single rectangle:

```rust
use laykit::{GDSIIFile, GDSStructure, GDSTime, GDSElement, Boundary};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create file
    let mut gds = GDSIIFile::new("SIMPLE".to_string());
    gds.units = (1e-6, 1e-9);
    
    // Create structure
    let mut structure = GDSStructure {
        name: "TOP".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    // Add rectangle
    structure.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![
            (0, 0),
            (1000, 0),
            (1000, 500),
            (0, 500),
            (0, 0),
        ],
        properties: Vec::new(),
    }));
    
    gds.structures.push(structure);
    gds.write_to_file("simple.gds")?;
    
    println!("✅ Created simple.gds");
    Ok(())
}
```

## Creating a Simple OASIS File

Create an OASIS file with a rectangle:

```rust
use laykit::{OASISFile, OASISCell, OASISElement, Rectangle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut oasis = OASISFile::new();
    oasis.unit = 1e-9;
    
    // Register cell name
    oasis.names.cell_names.insert(0, "TOP".to_string());
    
    let mut cell = OASISCell {
        name: "TOP".to_string(),
        elements: Vec::new(),
    };
    
    // Add rectangle
    cell.elements.push(OASISElement::Rectangle(Rectangle {
        layer: 1,
        datatype: 0,
        x: 0,
        y: 0,
        width: 1000,
        height: 500,
        repetition: None,
        properties: Vec::new(),
    }));
    
    oasis.cells.push(cell);
    oasis.write_to_file("simple.oas")?;
    
    println!("✅ Created simple.oas");
    Ok(())
}
```

## Quick Format Conversion

Convert GDSII to OASIS in just a few lines:

```rust
use laykit::{GDSIIFile, converter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gds = GDSIIFile::read_from_file("input.gds")?;
    let oasis = converter::gdsii_to_oasis(&gds)?;
    oasis.write_to_file("output.oas")?;
    println!("✅ Converted to OASIS");
    Ok(())
}
```

Convert OASIS to GDSII:

```rust
use laykit::{OASISFile, converter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oasis = OASISFile::read_from_file("input.oas")?;
    let gds = converter::oasis_to_gdsii(&oasis)?;
    gds.write_to_file("output.gds")?;
    println!("✅ Converted to GDSII");
    Ok(())
}
```

## Copying a File

Read and write back (useful for validation):

```rust
use laykit::GDSIIFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gds = GDSIIFile::read_from_file("original.gds")?;
    gds.write_to_file("copy.gds")?;
    println!("✅ File copied");
    Ok(())
}
```

## Error Handling

Proper error handling with user-friendly messages:

```rust
use laykit::GDSIIFile;

fn main() {
    match GDSIIFile::read_from_file("layout.gds") {
        Ok(gds) => {
            println!("✅ Successfully read {} structures", gds.structures.len());
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            eprintln!("Make sure the file exists and is a valid GDSII file");
            std::process::exit(1);
        }
    }
}
```

## Next Steps

- [Working with Elements](./examples-elements.md) - Process and modify elements
- [Hierarchical Designs](./examples-hierarchical.md) - Work with cell references
- [Complete Examples](./examples-complete.md) - Full working programs
