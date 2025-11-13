# GDSII to OASIS Conversion

Detailed guide for converting GDSII files to OASIS format.

## Basic Conversion

```rust
use laykit::{GDSIIFile, converter};

let gds = GDSIIFile::read_from_file("input.gds")?;
let oasis = converter::gdsii_to_oasis(&gds)?;
oasis.write_to_file("output.oas")?;
```

## What Gets Converted

### File-Level

- **Library name** → Stored in OASIS structure
- **Units** → GDSII database unit becomes OASIS unit
- **Version** → OASIS version set to "1.0"
- **Structures** → Converted to OASIS cells

### Structure-Level

- **Structure name** → Cell name
- **Timestamps** → Not preserved (OASIS has different metadata)
- **Elements** → Converted with type mapping

### Element-Level Mapping

#### Boundary → Rectangle or Polygon

If the boundary has exactly 5 vertices forming a rectangle:
```rust
// GDSII Boundary (rectangle)
Boundary {
    layer: 1,
    datatype: 0,
    xy: vec![(0,0), (100,0), (100,50), (0,50), (0,0)],
}

// Converts to OASIS Rectangle
Rectangle {
    layer: 1,
    datatype: 0,
    x: 0,
    y: 0,
    width: 100,
    height: 50,
}
```

Otherwise, converts to polygon:
```rust
// GDSII Boundary (triangle)
Boundary {
    xy: vec![(0,0), (100,0), (50,100), (0,0)],
}

// Converts to OASIS Polygon
Polygon {
    x: 0,
    y: 0,
    points: vec![(0,0), (100,0), (50,100), (0,0)],
}
```

#### Path → Path

Direct conversion with width preserved:
```rust
// GDSII Path
GPath {
    layer: 2,
    datatype: 0,
    width: Some(10),
    xy: vec![(0,0), (100,100)],
}

// Converts to OASIS Path
OPath {
    layer: 2,
    datatype: 0,
    width: 10,
    points: vec![(0,0), (100,100)],
}
```

#### Text → Text

Direct text conversion:
```rust
// GDSII Text
GText {
    string: "LABEL",
    xy: (100, 100),
}

// Converts to OASIS Text
OText {
    string: "LABEL",
    x: 100,
    y: 100,
}
```

#### StructRef → Placement

Single cell instances:
```rust
// GDSII StructRef
StructRef {
    sname: "SUBCELL",
    xy: (1000, 2000),
    strans: Some(...),
}

// Converts to OASIS Placement
Placement {
    cell_name: "SUBCELL",
    x: 1000,
    y: 2000,
    // Transformation preserved
}
```

#### ArrayRef → Placement with Repetition

Arrays are converted to repetitions:
```rust
// GDSII ArrayRef
ArrayRef {
    sname: "SUBCELL",
    columns: 10,
    rows: 5,
    xy: [
        (0, 0),      // Reference point
        (1000, 0),   // Column spacing
        (0, 2000),   // Row spacing
    ],
}

// Converts to OASIS Placement with Repetition
Placement {
    cell_name: "SUBCELL",
    x: 0,
    y: 0,
    repetition: Some(Repetition {
        x_dimension: 10,
        y_dimension: 5,
        x_space: 1000,
        y_space: 2000,
    }),
}
```

#### Node → Polygon

Nodes are converted to polygons:
```rust
// GDSII Node
Node {
    layer: 1,
    xy: vec![(0,0), (100,0), (100,100)],
}

// Converts to OASIS Polygon
Polygon {
    layer: 1,
    points: vec![(0,0), (100,0), (100,100)],
}
```

#### Box → Rectangle

Boxes become rectangles:
```rust
// GDSII Box (5 points)
GDSBox {
    xy: [(0,0), (100,0), (100,100), (0,100), (0,0)],
}

// Converts to OASIS Rectangle
Rectangle {
    x: 0,
    y: 0,
    width: 100,
    height: 100,
}
```

## Units Conversion

```rust
// GDSII units
gds.units = (1e-6, 1e-9);  // 1µm user, 1nm database

// OASIS conversion
oasis.unit = 1e-9;  // Uses database unit
```

## Name Table Population

OASIS requires pre-registered names:

```rust
// Cell names are automatically registered
oasis.names.cell_names.insert(0, "CELL1".to_string());
oasis.names.cell_names.insert(1, "CELL2".to_string());

// Text strings are registered
oasis.names.text_strings.insert(0, "LABEL".to_string());
```

## Transformation Handling

STrans (transformation) data is converted:

```rust
// GDSII STrans
STrans {
    reflect_x: true,
    magnification: Some(1.5),
    angle: Some(90.0),
}

// OASIS Placement transformation
Placement {
    mirror_x: true,
    magnification: Some(1.5),
    angle: Some(90.0),
    ...
}
```

## Properties

Element properties are preserved where possible:

```rust
// GDSII Property
GDSProperty {
    attribute: 1,
    value: "property_value",
}

// OASIS Property
OASISProperty {
    name: "attr_1",
    values: vec![PropertyValue::String("property_value")],
}
```

## Complete Example with Analysis

```rust
use laykit::{GDSIIFile, converter};

fn analyze_and_convert(input_path: &str, output_path: &str) 
    -> Result<(), Box<dyn std::error::Error>> 
{
    // Read GDSII
    let gds = GDSIIFile::read_from_file(input_path)?;
    
    // Analyze before conversion
    println!("GDSII Analysis:");
    println!("  Library: {}", gds.library_name);
    println!("  Structures: {}", gds.structures.len());
    
    let mut total_elements = 0;
    let mut boundaries = 0;
    let mut rectangles_detected = 0;
    
    for structure in &gds.structures {
        total_elements += structure.elements.len();
        for element in &structure.elements {
            if let GDSElement::Boundary(b) = element {
                boundaries += 1;
                if converter::is_rectangle(&b.xy).is_some() {
                    rectangles_detected += 1;
                }
            }
        }
    }
    
    println!("  Total elements: {}", total_elements);
    println!("  Boundaries: {}", boundaries);
    println!("  Rectangles detected: {}", rectangles_detected);
    
    // Convert
    let oasis = converter::gdsii_to_oasis(&gds)?;
    
    // Analyze after conversion
    println!("\nOASIS Analysis:");
    println!("  Cells: {}", oasis.cells.len());
    
    let mut oasis_elements = 0;
    let mut oasis_rectangles = 0;
    
    for cell in &oasis.cells {
        oasis_elements += cell.elements.len();
        for element in &cell.elements {
            if let OASISElement::Rectangle(_) = element {
                oasis_rectangles += 1;
            }
        }
    }
    
    println!("  Total elements: {}", oasis_elements);
    println!("  Rectangles: {}", oasis_rectangles);
    
    // Write output
    oasis.write_to_file(output_path)?;
    println!("\n✅ Conversion complete: {}", output_path);
    
    Ok(())
}
```

## Tips for Best Results

1. **Clean input files** - Remove unused structures before conversion
2. **Use rectangles** - Rectangular boundaries convert to compact rectangles
3. **Simplify hierarchies** - Flatten unnecessary hierarchy levels if needed
4. **Check layer maps** - Verify layer numbers are within OASIS limits
5. **Validate output** - Always check converted files in your EDA tool

## File Size Comparison

Typical OASIS files are 2-5× smaller than equivalent GDSII:

```rust
// Example file sizes:
// design.gds:  10.5 MB
// design.oas:   2.3 MB  (78% reduction)
```

This is due to:
- Variable-length integer encoding
- Rectangle primitives
- Name table compression
- Delta encoding for coordinates
