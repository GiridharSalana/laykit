# Working with Elements

Learn how to work with different element types in LayKit.

## Counting Elements

Count different element types in a GDSII file:

```rust
use laykit::{GDSIIFile, GDSElement};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gds = GDSIIFile::read_from_file("design.gds")?;
    
    let mut counts = ElementCounts::default();
    
    for structure in &gds.structures {
        for element in &structure.elements {
            match element {
                GDSElement::Boundary(_) => counts.boundaries += 1,
                GDSElement::Path(_) => counts.paths += 1,
                GDSElement::Text(_) => counts.texts += 1,
                GDSElement::StructRef(_) => counts.refs += 1,
                GDSElement::ArrayRef(_) => counts.arrays += 1,
                GDSElement::Node(_) => counts.nodes += 1,
                GDSElement::Box(_) => counts.boxes += 1,
            }
        }
    }
    
    println!("Element counts:");
    println!("  Boundaries: {}", counts.boundaries);
    println!("  Paths: {}", counts.paths);
    println!("  Texts: {}", counts.texts);
    println!("  References: {}", counts.refs);
    println!("  Arrays: {}", counts.arrays);
    println!("  Nodes: {}", counts.nodes);
    println!("  Boxes: {}", counts.boxes);
    
    Ok(())
}

#[derive(Default)]
struct ElementCounts {
    boundaries: usize,
    paths: usize,
    texts: usize,
    refs: usize,
    arrays: usize,
    nodes: usize,
    boxes: usize,
}
```

## Filtering by Layer

Extract all elements on a specific layer:

```rust
use laykit::{GDSIIFile, GDSElement};

fn filter_by_layer(gds: &GDSIIFile, target_layer: i16) {
    for structure in &gds.structures {
        let mut count = 0;
        
        for element in &structure.elements {
            let layer = match element {
                GDSElement::Boundary(b) => Some(b.layer),
                GDSElement::Path(p) => Some(p.layer),
                GDSElement::Text(t) => Some(t.layer),
                GDSElement::Node(n) => Some(n.layer),
                GDSElement::Box(b) => Some(b.layer),
                _ => None,
            };
            
            if layer == Some(target_layer) {
                count += 1;
            }
        }
        
        if count > 0 {
            println!("{}: {} elements on layer {}", 
                structure.name, count, target_layer);
        }
    }
}
```

## Calculating Bounding Box

Find the bounding box of all boundaries:

```rust
use laykit::{GDSIIFile, GDSElement};

fn calculate_bounds(gds: &GDSIIFile) -> Option<(i32, i32, i32, i32)> {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    let mut found = false;
    
    for structure in &gds.structures {
        for element in &structure.elements {
            if let GDSElement::Boundary(b) = element {
                for (x, y) in &b.xy {
                    min_x = min_x.min(*x);
                    min_y = min_y.min(*y);
                    max_x = max_x.max(*x);
                    max_y = max_y.max(*y);
                    found = true;
                }
            }
        }
    }
    
    if found {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gds = GDSIIFile::read_from_file("design.gds")?;
    
    if let Some((min_x, min_y, max_x, max_y)) = calculate_bounds(&gds) {
        println!("Bounding box:");
        println!("  Min: ({}, {})", min_x, min_y);
        println!("  Max: ({}, {})", max_x, max_y);
        println!("  Size: {} × {}", max_x - min_x, max_y - min_y);
    } else {
        println!("No boundaries found");
    }
    
    Ok(())
}
```

## Modifying Elements

Change all elements on layer 1 to layer 2:

```rust
use laykit::{GDSIIFile, GDSElement};

fn remap_layer(gds: &mut GDSIIFile, from_layer: i16, to_layer: i16) {
    for structure in &mut gds.structures {
        for element in &mut structure.elements {
            match element {
                GDSElement::Boundary(b) if b.layer == from_layer => {
                    b.layer = to_layer;
                }
                GDSElement::Path(p) if p.layer == from_layer => {
                    p.layer = to_layer;
                }
                GDSElement::Text(t) if t.layer == from_layer => {
                    t.layer = to_layer;
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut gds = GDSIIFile::read_from_file("input.gds")?;
    
    remap_layer(&mut gds, 1, 2);
    
    gds.write_to_file("remapped.gds")?;
    println!("✅ Remapped layer 1 to layer 2");
    
    Ok(())
}
```

## Extracting Text Labels

Get all text labels from a design:

```rust
use laykit::{GDSIIFile, GDSElement};

fn extract_text_labels(gds: &GDSIIFile) -> Vec<String> {
    let mut labels = Vec::new();
    
    for structure in &gds.structures {
        for element in &structure.elements {
            if let GDSElement::Text(t) = element {
                labels.push(t.string.clone());
            }
        }
    }
    
    labels
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gds = GDSIIFile::read_from_file("design.gds")?;
    let labels = extract_text_labels(&gds);
    
    println!("Found {} text labels:", labels.len());
    for (i, label) in labels.iter().enumerate() {
        println!("  {}. {}", i + 1, label);
    }
    
    Ok(())
}
```

## Scaling Coordinates

Scale all coordinates by a factor:

```rust
use laykit::{GDSIIFile, GDSElement};

fn scale_design(gds: &mut GDSIIFile, factor: f64) {
    for structure in &mut gds.structures {
        for element in &mut structure.elements {
            match element {
                GDSElement::Boundary(b) => {
                    for (x, y) in &mut b.xy {
                        *x = (*x as f64 * factor) as i32;
                        *y = (*y as f64 * factor) as i32;
                    }
                }
                GDSElement::Path(p) => {
                    for (x, y) in &mut p.xy {
                        *x = (*x as f64 * factor) as i32;
                        *y = (*y as f64 * factor) as i32;
                    }
                    if let Some(width) = &mut p.width {
                        *width = (*width as f64 * factor) as i32;
                    }
                }
                GDSElement::Text(t) => {
                    t.xy.0 = (t.xy.0 as f64 * factor) as i32;
                    t.xy.1 = (t.xy.1 as f64 * factor) as i32;
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut gds = GDSIIFile::read_from_file("input.gds")?;
    
    scale_design(&mut gds, 2.0); // 2x scaling
    
    gds.write_to_file("scaled.gds")?;
    println!("✅ Design scaled by 2x");
    
    Ok(())
}
```

## Filtering Structures

Keep only structures matching a pattern:

```rust
use laykit::GDSIIFile;

fn filter_structures(gds: &mut GDSIIFile, pattern: &str) {
    gds.structures.retain(|s| s.name.contains(pattern));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut gds = GDSIIFile::read_from_file("input.gds")?;
    
    println!("Before: {} structures", gds.structures.len());
    
    filter_structures(&mut gds, "TOP");
    
    println!("After: {} structures", gds.structures.len());
    
    gds.write_to_file("filtered.gds")?;
    println!("✅ Filtered structures");
    
    Ok(())
}
```

## Merging Files

Combine multiple GDSII files:

```rust
use laykit::GDSIIFile;

fn merge_files(files: Vec<&str>) -> Result<GDSIIFile, Box<dyn std::error::Error>> {
    let mut merged = GDSIIFile::new("MERGED".to_string());
    merged.units = (1e-6, 1e-9);
    
    for file_path in files {
        let gds = GDSIIFile::read_from_file(file_path)?;
        
        // Copy units from first file
        if merged.structures.is_empty() {
            merged.units = gds.units;
        }
        
        // Add all structures
        for structure in gds.structures {
            merged.structures.push(structure);
        }
    }
    
    Ok(merged)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let merged = merge_files(vec!["file1.gds", "file2.gds", "file3.gds"])?;
    
    println!("Merged {} structures", merged.structures.len());
    
    merged.write_to_file("merged.gds")?;
    println!("✅ Files merged");
    
    Ok(())
}
```
