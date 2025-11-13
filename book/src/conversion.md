# Format Conversion

LayKit provides bidirectional conversion between GDSII and OASIS formats, preserving geometry and hierarchy.

## Why Convert?

### GDSII to OASIS
- **Smaller file size** - OASIS uses compression and variable-length encoding
- **Modern format** - Better support for modern IC design features
- **Primitive shapes** - Rectangles, trapezoids, and circles for efficiency
- **Faster I/O** - Compact format means less data to read/write

### OASIS to GDSII
- **Universal compatibility** - GDSII is supported by virtually all EDA tools
- **Legacy tool support** - Older tools may not support OASIS
- **Industry standard** - Still the most widely used format

## Conversion Features

Both conversion directions provide:

✅ **Complete geometry preservation** - All shapes are accurately converted  
✅ **Hierarchy maintenance** - Cell references and structure are preserved  
✅ **Layer mapping** - Layers and datatypes are maintained  
✅ **Property transfer** - Element metadata is converted  
✅ **Smart optimization** - Intelligent shape detection (e.g., rectangles from polygons)

## Basic Usage

### GDSII → OASIS

```rust
use laykit::{GDSIIFile, converter};

let gds = GDSIIFile::read_from_file("input.gds")?;
let oasis = converter::gdsii_to_oasis(&gds)?;
oasis.write_to_file("output.oas")?;
```

### OASIS → GDSII

```rust
use laykit::{OASISFile, converter};

let oasis = OASISFile::read_from_file("input.oas")?;
let gds = converter::oasis_to_gdsii(&oasis)?;
gds.write_to_file("output.gds")?;
```

## Element Mapping

### GDSII to OASIS Element Conversion

| GDSII Element | OASIS Element | Notes |
|---------------|---------------|-------|
| Boundary (rectangle) | Rectangle | Detected automatically |
| Boundary (polygon) | Polygon | General polygons |
| Path | Path | Width and extensions preserved |
| Text | Text | Text strings maintained |
| StructRef | Placement | Single instance |
| ArrayRef | Placement with Repetition | Array converted to repetition |
| Node | Polygon | Converted to boundary |
| Box | Rectangle | Converted to rectangle |

### OASIS to GDSII Element Conversion

| OASIS Element | GDSII Element | Notes |
|---------------|---------------|-------|
| Rectangle | Boundary | Converted to 5-vertex polygon |
| Polygon | Boundary | Direct mapping |
| Path | Path | Width preserved |
| Trapezoid | Boundary | Converted to polygon |
| CTrapezoid | Boundary | Converted to polygon |
| Circle | Boundary | Approximated with polygon |
| Text | Text | Direct mapping |
| Placement | StructRef | Single instance |
| Placement with Repetition | ArrayRef | Repetition converted to array |

## Complete Conversion Example

```rust
use laykit::{GDSIIFile, OASISFile, converter};

fn convert_both_ways() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Format Conversion Demo ===\n");
    
    // Read original GDSII
    println!("1. Reading GDSII file...");
    let gds_original = GDSIIFile::read_from_file("design.gds")?;
    println!("   Library: {}", gds_original.library_name);
    println!("   Structures: {}", gds_original.structures.len());
    
    // Convert to OASIS
    println!("\n2. Converting GDSII → OASIS...");
    let oasis = converter::gdsii_to_oasis(&gds_original)?;
    println!("   Cells: {}", oasis.cells.len());
    oasis.write_to_file("converted.oas")?;
    println!("   ✅ Written to converted.oas");
    
    // Convert back to GDSII
    println!("\n3. Converting OASIS → GDSII...");
    let gds_converted = converter::oasis_to_gdsii(&oasis)?;
    println!("   Structures: {}", gds_converted.structures.len());
    gds_converted.write_to_file("roundtrip.gds")?;
    println!("   ✅ Written to roundtrip.gds");
    
    // Verify
    println!("\n4. Verification:");
    println!("   Original structures: {}", gds_original.structures.len());
    println!("   Roundtrip structures: {}", gds_converted.structures.len());
    println!("   Match: {}", 
        gds_original.structures.len() == gds_converted.structures.len());
    
    Ok(())
}
```

## Conversion Options

### Units Handling

**GDSII → OASIS:**
- GDSII user units and database units are converted to OASIS database unit
- Default: Uses GDSII database unit (typically 1nm)

**OASIS → GDSII:**
- OASIS unit becomes GDSII database unit
- User unit set to 1000× database unit (e.g., 1µm user, 1nm database)

### Rectangle Detection

When converting GDSII to OASIS, the converter automatically detects rectangles:

```rust
// A 5-vertex GDSII boundary that forms a rectangle
let boundary = Boundary {
    xy: vec![(0,0), (1000,0), (1000,500), (0,500), (0,0)],
    ...
};

// Is automatically converted to an OASIS Rectangle
let rectangle = Rectangle {
    x: 0, y: 0,
    width: 1000, height: 500,
    ...
};
```

This results in much smaller OASIS files!

## Error Handling

```rust
match converter::gdsii_to_oasis(&gds) {
    Ok(oasis) => {
        println!("Conversion successful!");
        oasis.write_to_file("output.oas")?;
    }
    Err(e) => {
        eprintln!("Conversion failed: {}", e);
        eprintln!("Check input file for unsupported features");
    }
}
```

## Performance Tips

1. **Batch conversions** - Process multiple files in parallel
2. **Stream large files** - For very large files, consider chunked processing
3. **Verify output** - Always check converted files in your EDA tool
4. **Keep originals** - Maintain backups before conversion

## Limitations

Current implementation:
- ✅ All standard elements supported
- ✅ Hierarchical designs fully supported
- ✅ Properties and metadata preserved
- ⚠️ Custom extensions may need manual handling
- ⚠️ Extremely large files (>1GB) may need memory optimization

## Next Steps

- [GDSII to OASIS](./gdsii-to-oasis.md) - Detailed GDSII→OASIS conversion
- [OASIS to GDSII](./oasis-to-gdsii.md) - Detailed OASIS→GDSII conversion
