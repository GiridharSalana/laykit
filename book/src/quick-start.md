# Quick Start

Get up and running with LayKit in minutes!

## Minimal Example

Here's the simplest possible LayKit program:

```rust
use laykit::load_library;

fn main() -> Result<(), laykit::LaykitError> {
    // Read GDSII or OASIS (auto-detected)
    let lib = load_library("layout.gds")?;
    println!("Library: {}", lib.name());
    println!("Cells: {}", lib.cell_count());
    lib.save("output.oas")?;
    Ok(())
}
```

### Format-specific reading

```rust
use laykit::GDSIIFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gds = GDSIIFile::read_from_file("layout.gds")?;
    
    // Print information
    println!("Library: {}", gds.library_name);
    println!("Structures: {}", gds.structures.len());
    
    // Write it back
    gds.write_to_file("output.gds")?;
    
    Ok(())
}
```

## Common Operations

### Reading Files

```rust
use laykit::{GDSIIFile, OASISFile};

// Read GDSII
let gds = GDSIIFile::read_from_file("design.gds")?;

// Read OASIS
let oasis = OASISFile::read_from_file("design.oas")?;
```

### Creating Files

```rust
use laykit::{GDSIIFile, GDSStructure, GDSTime, GDSElement, Boundary};

let mut gds = GDSIIFile::new("MY_LIBRARY".to_string());
gds.units = (1e-6, 1e-9); // 1 micron, 1nm

let mut structure = GDSStructure {
    name: "TOP".to_string(),
    creation_time: GDSTime::now(),
    modification_time: GDSTime::now(),
    elements: Vec::new(),
};

// Add a rectangle
structure.elements.push(GDSElement::Boundary(Boundary {
    layer: 1,
    datatype: 0,
    xy: vec![(0, 0), (1000, 0), (1000, 1000), (0, 1000), (0, 0)],
    properties: Vec::new(),
}));

gds.structures.push(structure);
gds.write_to_file("output.gds")?;
```

### Converting Between Formats

```rust
use laykit::{GDSIIFile, converter};

// GDSII to OASIS
let gds = GDSIIFile::read_from_file("input.gds")?;
let oasis = converter::gdsii_to_oasis(&gds)?;
oasis.write_to_file("output.oas")?;

// OASIS to GDSII
let oasis = OASISFile::read_from_file("input.oas")?;
let gds = converter::oasis_to_gdsii(&oasis)?;
gds.write_to_file("output.gds")?;
```

## Running Examples

LayKit comes with complete working examples:

```bash
# Clone the repository
git clone https://github.com/giridharsalana/laykit.git
cd laykit

# Run the basic usage example
cargo run --example basic_usage

# Run GDSII-only example
cargo run --example gdsii_only

# Run OASIS-only example
cargo run --example oasis_only
```

## Next Steps

- Learn more about [GDSII Format](./gdsii.md)
- Learn more about [OASIS Format](./oasis.md)
- Explore [Complete Examples](./examples-complete.md)
- Read the [API Reference](./api-reference.md)
