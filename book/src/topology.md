# Topology

The `topology` module manages cell hierarchies — dependency ordering, flattening, layer queries, and library merging.

## Cell Dependencies

```rust
use laykit::{GDSIIFile, top_level_cells, direct_references, cell_dependencies, dependency_order};

let gds = GDSIIFile::read_from_file("design.gds")?;

// Cells not referenced by any other cell
let tops = top_level_cells(&gds);
for cell in &tops {
    println!("top: {}", cell.name);
}

// Direct children of a cell
let children = direct_references(&gds.structures[0]);

// All transitive dependencies of a cell
let all_deps = cell_dependencies("TOP", &gds);

// Topological sort: leaf cells first, root last
let order = dependency_order(&gds);
for i in order {
    println!("{}", gds.structures[i].name);
}
```

## Hierarchy Validation

```rust
use laykit::{detect_cycles, validate_hierarchy};

// Find circular references
let cycles = detect_cycles(&gds);
if !cycles.is_empty() {
    println!("Cycles: {:?}", cycles);
}

// Full validation (missing refs, cycles)
match validate_hierarchy(&gds) {
    Ok(())   => println!("Hierarchy is valid"),
    Err(err) => println!("Errors: {:?}", err),
}
```

## Flattening

```rust
use laykit::flatten_structure;

// Expand all cell references into a flat list of elements
// (coordinates are transformed to the top-level frame)
let flat = flatten_structure("TOP", &gds, None);        // unlimited depth
let flat2 = flatten_structure("TOP", &gds, Some(2));    // max 2 levels deep
```

## Layer Queries

```rust
use laykit::{layers_in_structure, layers_in_library, filter_by_layer,
             element_layer, total_element_count};

// Which layers are used?
let layers = layers_in_library(&gds);

// Elements on a specific layer
let metal1 = filter_by_layer(&gds.structures[0], 1);

// Layer of a single element
if let Some(layer) = element_layer(&element) {
    println!("layer {}", layer);
}

// Total element count across all structures
println!("{} elements total", total_element_count(&gds));
```

## Library Merge

```rust
use laykit::{merge_library, merge_library_overwrite};

let mut target = GDSIIFile::read_from_file("base.gds")?;
let source = GDSIIFile::read_from_file("extra.gds")?;

// Add cells from source that don't already exist in target
let added = merge_library(&mut target, &source);

// Add all cells, overwriting duplicates
let replaced = merge_library_overwrite(&mut target, &source);

println!("{} cells added", added);
```
