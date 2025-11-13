# AREF Expansion

Array references (AREF) in GDSII allow efficient representation of repeated cell instances. LayKit provides utilities to expand these arrays into individual structure references.

## Overview

An AREF represents multiple cell instances in a regular grid pattern:

```
AREF "SUBCELL" 3x2 array:
[0,0] [100,0] [200,0]
[0,100] [100,100] [200,100]
```

Expansion converts this single AREF into 6 individual SREF (structure reference) elements.

## Basic Usage

```rust
use laykit::{expand_array_ref, ArrayRef};

let aref = ArrayRef {
    sname: "SUBCELL".to_string(),
    columns: 3,
    rows: 2,
    xy: vec![
        (0, 0),      // Origin
        (300, 0),    // Column reference point
        (0, 200),    // Row reference point
    ],
    strans: None,
    properties: Vec::new(),
};

// Expand into individual references
let expanded = expand_array_ref(&aref);

// Result: 6 StructRef elements at positions:
// (0,0), (100,0), (200,0), (0,100), (100,100), (200,100)
println!("Created {} instances", expanded.len());
```

## How AREF Works

### Reference Points

An AREF has three reference points:
1. **Origin** - First instance position
2. **Column point** - Defines column spacing
3. **Row point** - Defines row spacing

```rust
let xy = vec![
    (0, 0),      // Origin: first instance
    (300, 0),    // Column: 3 columns spanning 300 units
    (0, 200),    // Row: 2 rows spanning 200 units
];

// Spacing calculated as:
// col_spacing = (300 - 0) / 3 = 100 per column
// row_spacing = (200 - 0) / 2 = 100 per row
```

### Instance Positions

Each instance position is calculated:

```
position = origin + (col * col_spacing) + (row * row_spacing)
```

## Expanding All Arrays

Process all array references in a structure:

```rust
use laykit::expand_all_array_refs;

let mut structure = /* load structure */;

// Expand all ARefs to SRefs
structure.elements = expand_all_array_refs(&structure.elements);

// Save modified structure
gds.write_to_file("expanded.gds")?;
```

## Counting Instances

Count total instances before expansion:

```rust
use laykit::count_expanded_instances;

let count = count_expanded_instances(&structure.elements);
println!("Total instances after expansion: {}", count);
```

## Preserving Transformations

AREF transformations are preserved in each expanded instance:

```rust
let aref = ArrayRef {
    sname: "CELL".to_string(),
    columns: 2,
    rows: 2,
    xy: vec![(0, 0), (200, 0), (0, 200)],
    strans: Some(STrans {
        reflection_x: true,
        absolute_magnification: false,
        absolute_angle: false,
        magnification: Some(2.0),
        angle: Some(90.0),
    }),
    properties: vec![
        GDSProperty {
            attribute: 1,
            value: "ARRAYINSTANCE".to_string(),
        }
    ],
};

let expanded = expand_array_ref(&aref);

// Each StructRef has the same strans and properties
for elem in &expanded {
    if let GDSElement::StructRef(sref) = elem {
        assert!(sref.strans.is_some());
        assert_eq!(sref.properties.len(), 1);
    }
}
```

## Use Cases

### Flattening Arrays

```rust
fn flatten_arrays(gds: &mut GDSIIFile) {
    for structure in &mut gds.structures {
        structure.elements = expand_all_array_refs(&structure.elements);
    }
}
```

### Selective Expansion

```rust
fn expand_large_arrays(elements: &[GDSElement], threshold: usize) -> Vec<GDSElement> {
    elements.iter()
        .flat_map(|elem| {
            match elem {
                GDSElement::ArrayRef(aref) => {
                    let count = (aref.rows as usize) * (aref.columns as usize);
                    if count > threshold {
                        expand_array_ref(aref)
                    } else {
                        vec![elem.clone()]
                    }
                }
                _ => vec![elem.clone()]
            }
        })
        .collect()
}
```

### Array Analysis

```rust
fn analyze_arrays(structure: &GDSStructure) {
    for element in &structure.elements {
        if let GDSElement::ArrayRef(aref) = element {
            let total = (aref.rows as usize) * (aref.columns as usize);
            println!("Array '{}': {}x{} = {} instances",
                aref.sname, aref.columns, aref.rows, total);
            
            // Calculate bounding box
            let col_spacing = (aref.xy[1].0 - aref.xy[0].0) / aref.columns as i32;
            let row_spacing = (aref.xy[2].1 - aref.xy[0].1) / aref.rows as i32;
            
            println!("  Spacing: col={}, row={}", col_spacing, row_spacing);
        }
    }
}
```

### Instance Position Mapping

```rust
fn get_instance_positions(aref: &ArrayRef) -> Vec<(i32, i32)> {
    let expanded = expand_array_ref(aref);
    
    expanded.iter()
        .filter_map(|elem| {
            if let GDSElement::StructRef(sref) = elem {
                Some(sref.xy)
            } else {
                None
            }
        })
        .collect()
}
```

## Performance Considerations

### Memory Usage

Expansion increases memory usage:

```rust
// Original: 1 AREF element
let aref = ArrayRef { /* 10x10 array */ };

// Expanded: 100 SREF elements
let expanded = expand_array_ref(&aref);

// Memory increase: ~100x for this case
```

### When to Expand

**Expand when:**
- Tool doesn't support AREF
- Need to modify individual instances
- Performing instance-level analysis
- Converting to formats without array support

**Keep AREF when:**
- File size matters
- Regular grid pattern is important
- Tool supports AREF natively
- No modification needed

### File Size Impact

| Array Size | AREF Size | Expanded Size | Ratio |
|------------|-----------|---------------|-------|
| 10x10 | ~100 bytes | ~10 KB | 100x |
| 100x100 | ~100 bytes | ~1 MB | 10,000x |
| 1000x1000 | ~100 bytes | ~100 MB | 1,000,000x |

## Advanced Features

### Non-orthogonal Arrays

Arrays don't have to be axis-aligned:

```rust
let diagonal_array = ArrayRef {
    sname: "CELL".to_string(),
    columns: 5,
    rows: 5,
    xy: vec![
        (0, 0),       // Origin
        (500, 500),   // Diagonal columns
        (-500, 500),  // Diagonal rows (perpendicular)
    ],
    strans: None,
    properties: Vec::new(),
};

let expanded = expand_array_ref(&diagonal_array);
// Creates 25 instances in a diamond pattern
```

### Validating Arrays

```rust
fn validate_aref(aref: &ArrayRef) -> Result<(), String> {
    if aref.xy.len() != 3 {
        return Err(format!("AREF must have 3 points, found {}", aref.xy.len()));
    }
    
    if aref.columns == 0 || aref.rows == 0 {
        return Err(format!("Invalid dimensions: {}x{}", aref.columns, aref.rows));
    }
    
    // Check for zero spacing
    let col_spacing_x = (aref.xy[1].0 - aref.xy[0].0) / aref.columns as i32;
    let col_spacing_y = (aref.xy[1].1 - aref.xy[0].1) / aref.columns as i32;
    
    if col_spacing_x == 0 && col_spacing_y == 0 {
        return Err("Column spacing is zero".to_string());
    }
    
    Ok(())
}
```

## Limitations

Current implementation:
- ✅ Regular grids
- ✅ Transformation preservation
- ✅ Property preservation
- ⚠️ Assumes valid 3-point AREF format
- ⚠️ No automatic overlap detection
- ⚠️ No optimization for partial expansion

## Future Enhancements

Planned for future versions:
- Partial array expansion (expand only a region)
- Array optimization (detect expandable SREF patterns)
- Overlap detection and warning
- Support for OASIS repetitions
- Array pattern recognition

