# Reading OASIS Files

Learn how to read and parse OASIS files with LayKit.

## Basic Reading

```rust
use laykit::OASISFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oasis = OASISFile::read_from_file("design.oas")?;
    println!("Loaded OASIS file with {} cells", oasis.cells.len());
    Ok(())
}
```

## Accessing File Information

```rust
let oasis = OASISFile::read_from_file("design.oas")?;

println!("Version: {}", oasis.version);
println!("Unit: {} meters", oasis.unit);
println!("Cells: {}", oasis.cells.len());
println!("Cell names in table: {}", oasis.names.cell_names.len());
```

## Iterating Through Cells

```rust
for cell in &oasis.cells {
    println!("\nCell: {}", cell.name);
    println!("  Elements: {}", cell.elements.len());
    
    // Count element types
    let mut rectangles = 0;
    let mut polygons = 0;
    let mut paths = 0;
    
    for element in &cell.elements {
        match element {
            OASISElement::Rectangle(_) => rectangles += 1,
            OASISElement::Polygon(_) => polygons += 1,
            OASISElement::Path(_) => paths += 1,
            _ => {}
        }
    }
    
    println!("  Rectangles: {}, Polygons: {}, Paths: {}", 
        rectangles, polygons, paths);
}
```

## Processing Elements

```rust
use laykit::OASISElement;

for element in &cell.elements {
    match element {
        OASISElement::Rectangle(r) => {
            println!("Rectangle: layer={}, {}×{} at ({},{})",
                r.layer, r.width, r.height, r.x, r.y);
        }
        OASISElement::Polygon(p) => {
            println!("Polygon: layer={}, {} points at ({},{})",
                p.layer, p.points.len(), p.x, p.y);
        }
        OASISElement::Path(p) => {
            println!("Path: layer={}, {} points",
                p.layer, p.points.len());
        }
        OASISElement::Trapezoid(t) => {
            println!("Trapezoid: layer={}", t.layer);
        }
        OASISElement::CTrapezoid(ct) => {
            println!("CTrapezoid: layer={}", ct.layer);
        }
        OASISElement::Circle(c) => {
            println!("Circle: layer={}, radius={}", c.layer, c.radius);
        }
        OASISElement::Text(t) => {
            println!("Text: \"{}\" at ({},{})", t.string, t.x, t.y);
        }
        OASISElement::Placement(p) => {
            println!("Placement: cell reference at ({},{})", p.x, p.y);
        }
    }
}
```

## Working with Name Tables

OASIS uses name tables for efficient string storage:

```rust
// Access cell names
for (id, name) in &oasis.names.cell_names {
    println!("Cell ID {}: {}", id, name);
}

// Access text strings
for (id, text) in &oasis.names.text_strings {
    println!("Text ID {}: {}", id, text);
}

// Access property names
for (id, prop) in &oasis.names.prop_names {
    println!("Property ID {}: {}", id, prop);
}
```

## Error Handling

```rust
match OASISFile::read_from_file("design.oas") {
    Ok(oasis) => {
        println!("Successfully read {} cells", oasis.cells.len());
    }
    Err(e) => {
        eprintln!("Error reading OASIS file: {}", e);
        eprintln!("Make sure the file exists and is a valid OASIS file");
    }
}
```

## Reading from Buffer

You can also read from any `Read` source:

```rust
use std::fs::File;
use std::io::BufReader;
use laykit::OASISFile;

let file = File::open("design.oas")?;
let mut reader = BufReader::new(file);
let oasis = OASISFile::read(&mut reader)?;
```
