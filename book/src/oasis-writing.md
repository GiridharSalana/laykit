# Writing OASIS Files

Learn how to create and write OASIS files with LayKit.

## Creating a New File

```rust
use laykit::{OASISFile, OASISCell, OASISElement, Rectangle};

let mut oasis = OASISFile::new();
oasis.unit = 1e-9; // 1nm database unit
```

## Adding Cells

```rust
let mut cell = OASISCell {
    name: "TOP_CELL".to_string(),
    elements: Vec::new(),
};

// Add elements to cell...

oasis.cells.push(cell);
```

## Registering Names

Before referencing cells or strings, register them in name tables:

```rust
// Register cell name
oasis.names.cell_names.insert(0, "TOP_CELL".to_string());

// Register text strings
oasis.names.text_strings.insert(0, "LABEL_TEXT".to_string());

// Register property names
oasis.names.prop_names.insert(0, "PROPERTY_NAME".to_string());
```

## Adding Elements

### Rectangle

```rust
use laykit::{OASISElement, Rectangle};

cell.elements.push(OASISElement::Rectangle(Rectangle {
    layer: 1,
    datatype: 0,
    x: 0,
    y: 0,
    width: 10000,
    height: 5000,
    repetition: None,
    properties: Vec::new(),
}));
```

### Polygon

```rust
use laykit::{OASISElement, Polygon};

cell.elements.push(OASISElement::Polygon(Polygon {
    layer: 2,
    datatype: 0,
    x: 0,
    y: 0,
    points: vec![
        (0, 0),
        (10000, 0),
        (5000, 10000),
        (0, 0),
    ],
    repetition: None,
    properties: Vec::new(),
}));
```

### Path

```rust
use laykit::{OASISElement, OPath};

cell.elements.push(OASISElement::Path(OPath {
    layer: 3,
    datatype: 0,
    width: 100,
    x: 0,
    y: 0,
    points: vec![(0, 0), (1000, 1000), (2000, 0)],
    start_extension: 0,
    end_extension: 0,
    repetition: None,
    properties: Vec::new(),
}));
```

### Trapezoid

```rust
use laykit::{OASISElement, Trapezoid};

cell.elements.push(OASISElement::Trapezoid(Trapezoid {
    layer: 4,
    datatype: 0,
    x: 0,
    y: 0,
    width: 1000,
    height: 1000,
    delta_a: 100,
    delta_b: 200,
    repetition: None,
    properties: Vec::new(),
}));
```

### Circle

```rust
use laykit::{OASISElement, Circle};

cell.elements.push(OASISElement::Circle(Circle {
    layer: 5,
    datatype: 0,
    x: 500,
    y: 500,
    radius: 250,
    repetition: None,
    properties: Vec::new(),
}));
```

### Text

```rust
use laykit::{OASISElement, OText};

cell.elements.push(OASISElement::Text(OText {
    layer: 6,
    texttype: 0,
    string: "LABEL".to_string(),
    x: 1000,
    y: 1000,
    repetition: None,
    properties: Vec::new(),
}));
```

### Placement (Cell Instance)

```rust
use laykit::{OASISElement, Placement};

// Make sure to register the cell name first
oasis.names.cell_names.insert(1, "SUBCELL".to_string());

cell.elements.push(OASISElement::Placement(Placement {
    cell_name: "SUBCELL".to_string(),
    x: 2000,
    y: 2000,
    magnification: None,
    angle: None,
    mirror_x: false,
    repetition: None,
    properties: Vec::new(),
}));
```

## Writing to File

```rust
oasis.write_to_file("output.oas")?;
println!("OASIS file written successfully!");
```

## Complete Example

```rust
use laykit::{OASISFile, OASISCell, OASISElement, Rectangle, Polygon};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut oasis = OASISFile::new();
    oasis.unit = 1e-9; // 1nm
    
    // Register cell name
    oasis.names.cell_names.insert(0, "DEMO".to_string());
    
    let mut cell = OASISCell {
        name: "DEMO".to_string(),
        elements: Vec::new(),
    };
    
    // Add rectangle
    cell.elements.push(OASISElement::Rectangle(Rectangle {
        layer: 1,
        datatype: 0,
        x: 0,
        y: 0,
        width: 10000,
        height: 5000,
        repetition: None,
        properties: Vec::new(),
    }));
    
    // Add triangle
    cell.elements.push(OASISElement::Polygon(Polygon {
        layer: 2,
        datatype: 0,
        x: 15000,
        y: 0,
        points: vec![(0, 0), (5000, 0), (2500, 5000), (0, 0)],
        repetition: None,
        properties: Vec::new(),
    }));
    
    oasis.cells.push(cell);
    oasis.write_to_file("demo.oas")?;
    
    println!("✅ Created demo.oas");
    Ok(())
}
```

## Writing to Buffer

You can also write to any `Write` destination:

```rust
use std::fs::File;
use std::io::BufWriter;

let file = File::create("output.oas")?;
let mut writer = BufWriter::new(file);
oasis.write(&mut writer)?;
```
