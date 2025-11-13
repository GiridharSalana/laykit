# OASIS to GDSII Conversion

Detailed guide for converting OASIS files to GDSII format.

## Basic Conversion

```rust
use laykit::{OASISFile, converter};

let oasis = OASISFile::read_from_file("input.oas")?;
let gds = converter::oasis_to_gdsii(&oasis)?;
gds.write_to_file("output.gds")?;
```

## What Gets Converted

### File-Level

- **OASIS version** → Stored as GDSII version (typically 3 or 5)
- **Unit** → OASIS unit becomes GDSII database unit
- **Cells** → Converted to GDSII structures
- **Name tables** → Expanded to actual strings

### Cell-Level

- **Cell name** → Structure name
- **Timestamps** → Set to current time (OASIS doesn't always store timestamps)
- **Elements** → Converted with type mapping

### Element-Level Mapping

#### Rectangle → Boundary

OASIS rectangles become 5-vertex closed polygons:

```rust
// OASIS Rectangle
Rectangle {
    layer: 1,
    datatype: 0,
    x: 0,
    y: 0,
    width: 100,
    height: 50,
}

// Converts to GDSII Boundary
Boundary {
    layer: 1,
    datatype: 0,
    xy: vec![
        (0, 0),
        (100, 0),
        (100, 50),
        (0, 50),
        (0, 0),  // Closed polygon
    ],
}
```

#### Polygon → Boundary

Direct conversion:

```rust
// OASIS Polygon
Polygon {
    layer: 2,
    datatype: 0,
    x: 0,
    y: 0,
    points: vec![(0,0), (100,0), (50,100), (0,0)],
}

// Converts to GDSII Boundary
Boundary {
    layer: 2,
    datatype: 0,
    xy: vec![(0,0), (100,0), (50,100), (0,0)],
}
```

**Note:** OASIS polygon points are relative to (x, y), so they're adjusted to absolute coordinates.

#### Path → Path

Paths are directly converted:

```rust
// OASIS Path
OPath {
    layer: 3,
    datatype: 0,
    width: 10,
    points: vec![(0,0), (100,100)],
    start_extension: 5,
    end_extension: 5,
}

// Converts to GDSII Path
GPath {
    layer: 3,
    datatype: 0,
    pathtype: 0,
    width: Some(10),
    xy: vec![(0,0), (100,100)],
    // Extensions handled by pathtype
}
```

#### Trapezoid → Boundary

Trapezoids are converted to 5-vertex polygons:

```rust
// OASIS Trapezoid
Trapezoid {
    x: 0,
    y: 0,
    width: 100,
    height: 50,
    delta_a: 10,  // Top offset
    delta_b: -10, // Bottom offset
}

// Converts to GDSII Boundary
Boundary {
    xy: vec![
        (0, 0),
        (90, 0),      // width + delta_b
        (110, 50),    // width + delta_a at top
        (10, 50),     // delta_a
        (0, 0),
    ],
}
```

#### CTrapezoid → Boundary

Constrained trapezoids are also converted to polygons:

```rust
// OASIS CTrapezoid
CTrapezoid {
    layer: 4,
    ctrapezoid_type: 0,
    width: 100,
    height: 50,
    ...
}

// Converts to GDSII Boundary with appropriate vertices
```

#### Circle → Boundary (Approximated)

Circles are approximated with polygons:

```rust
// OASIS Circle
Circle {
    layer: 5,
    x: 500,
    y: 500,
    radius: 100,
}

// Converts to GDSII Boundary with 32 vertices (approximation)
Boundary {
    layer: 5,
    xy: vec![
        // 32 points forming a circle
        // calculated as: (x + r*cos(θ), y + r*sin(θ))
    ],
}
```

**Note:** The number of segments can be adjusted for accuracy vs. file size.

#### Text → Text

Direct text conversion:

```rust
// OASIS Text
OText {
    layer: 6,
    texttype: 0,
    string: "LABEL",
    x: 1000,
    y: 1000,
}

// Converts to GDSII Text
GText {
    layer: 6,
    texttype: 0,
    string: "LABEL",
    xy: (1000, 1000),
}
```

#### Placement → StructRef or ArrayRef

Single placements:

```rust
// OASIS Placement (no repetition)
Placement {
    cell_name: "SUBCELL",
    x: 1000,
    y: 2000,
    magnification: Some(1.5),
    angle: Some(90.0),
    mirror_x: true,
    repetition: None,
}

// Converts to GDSII StructRef
StructRef {
    sname: "SUBCELL",
    xy: (1000, 2000),
    strans: Some(STrans {
        reflect_x: true,
        magnification: Some(1.5),
        angle: Some(90.0),
        ...
    }),
}
```

Placements with repetition:

```rust
// OASIS Placement with Repetition
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

// Converts to GDSII ArrayRef
ArrayRef {
    sname: "SUBCELL",
    columns: 10,
    rows: 5,
    xy: [
        (0, 0),           // Reference point
        (10000, 0),       // Column spacing (10 * 1000)
        (0, 10000),       // Row spacing (5 * 2000)
    ],
}
```

## Units Conversion

```rust
// OASIS unit
oasis.unit = 1e-9;  // 1nm

// GDSII units conversion
gds.units = (1e-6, 1e-9);  // 1µm user unit, 1nm database unit
//              ^      ^
//          user   database (from OASIS)
```

The user unit is set to 1000× the database unit by default.

## Timestamp Handling

Since OASIS may not store modification times:

```rust
// All structures get current timestamp
structure.creation_time = GDSTime::now();
structure.modification_time = GDSTime::now();
```

## Name Table Expansion

OASIS name tables are expanded:

```rust
// OASIS has compact name tables
oasis.names.cell_names: {0: "CELL1", 1: "CELL2"}
oasis.names.text_strings: {0: "LABEL"}

// Expanded in GDSII as actual strings
gds.structures[0].name = "CELL1";
gds.structures[1].name = "CELL2";
// Text elements contain "LABEL" directly
```

## Coordinate Conversion

OASIS uses 64-bit coordinates, GDSII uses 32-bit:

```rust
// OASIS coordinates (i64)
let oasis_x: i64 = 1_000_000_000;

// Converted to GDSII (i32)
let gds_x: i32 = oasis_x as i32;  // May truncate if too large!
```

**Warning:** Very large OASIS coordinates may overflow GDSII's 32-bit limit.

## Complete Example with Validation

```rust
use laykit::{OASISFile, converter, GDSElement};

fn convert_and_validate(input_path: &str, output_path: &str) 
    -> Result<(), Box<dyn std::error::Error>> 
{
    // Read OASIS
    let oasis = OASISFile::read_from_file(input_path)?;
    
    println!("OASIS Analysis:");
    println!("  Version: {}", oasis.version);
    println!("  Unit: {} meters", oasis.unit);
    println!("  Cells: {}", oasis.cells.len());
    
    let mut total_elements = 0;
    let mut rectangles = 0;
    let mut circles = 0;
    let mut trapezoids = 0;
    
    for cell in &oasis.cells {
        total_elements += cell.elements.len();
        for element in &cell.elements {
            match element {
                OASISElement::Rectangle(_) => rectangles += 1,
                OASISElement::Circle(_) => circles += 1,
                OASISElement::Trapezoid(_) => trapezoids += 1,
                OASISElement::CTrapezoid(_) => trapezoids += 1,
                _ => {}
            }
        }
    }
    
    println!("  Total elements: {}", total_elements);
    println!("  Rectangles: {}", rectangles);
    println!("  Circles: {} (will be approximated)", circles);
    println!("  Trapezoids: {}", trapezoids);
    
    // Convert
    let gds = converter::oasis_to_gdsii(&oasis)?;
    
    // Analyze GDSII output
    println!("\nGDSII Output:");
    println!("  Library: {}", gds.library_name);
    println!("  Structures: {}", gds.structures.len());
    println!("  Units: {}µm user, {}nm database", 
        gds.units.0 * 1e6, gds.units.1 * 1e9);
    
    let mut gds_elements = 0;
    let mut boundaries = 0;
    
    for structure in &gds.structures {
        gds_elements += structure.elements.len();
        for element in &structure.elements {
            if let GDSElement::Boundary(_) = element {
                boundaries += 1;
            }
        }
    }
    
    println!("  Total elements: {}", gds_elements);
    println!("  Boundaries: {} (includes converted shapes)", boundaries);
    
    // Write output
    gds.write_to_file(output_path)?;
    println!("\n✅ Conversion complete: {}", output_path);
    
    Ok(())
}
```

## Handling Edge Cases

### Large Coordinates

```rust
// Check for coordinate overflow
if oasis_coord > i32::MAX as i64 {
    eprintln!("Warning: Coordinate {} exceeds GDSII limit", oasis_coord);
    // May need to scale design or split into multiple files
}
```

### Complex Repetitions

```rust
// Large arrays might create many instances
if repetition.x_dimension * repetition.y_dimension > 10000 {
    println!("Warning: Large array ({} instances)", 
        repetition.x_dimension * repetition.y_dimension);
}
```

### Circle Approximation Quality

```rust
// For critical circles, verify approximation:
// - Check vertex count (typically 32-64)
// - Ensure radius error is acceptable
// - Consider increasing segments for large circles
```

## Tips for Best Results

1. **Check coordinate ranges** - Ensure they fit in 32-bit integers
2. **Verify circle approximations** - May need visual inspection
3. **Test in target tool** - Load converted GDSII in your EDA software
4. **Compare file sizes** - GDSII will be larger than OASIS
5. **Preserve metadata** - Add comments about conversion if needed

## File Size Comparison

GDSII files are typically larger than OASIS:

```rust
// Example file sizes:
// design.oas:  2.3 MB
// design.gds: 10.5 MB  (4.5× larger)
```

This is expected due to GDSII's less efficient encoding.
