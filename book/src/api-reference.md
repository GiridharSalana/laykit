# API Reference

## Module Overview

```
laykit
├── gdsii           — GDSII format (read/write, all element types)
├── oasis           — OASIS format (read/write, all element types)
├── converter       — Bidirectional GDSII ↔ OASIS conversion
├── geometry        — Bounding box, transforms, area, point-in-polygon
├── boolean_ops     — Union, intersection, difference, XOR, slice, offset
├── flexpath        — FlexPath with joins/caps; RobustPath
├── curve           — Arc, Bezier, ellipse, spline, polygon, star, spiral
├── topology        — Cell hierarchy: flatten, order, merge, validate
├── streaming       — Streaming GDSII parser for large files
├── aref_expansion  — AREF → individual SREF expansion
├── properties      — Property builders and managers
└── format_detection — File format detection by magic bytes
```

---

## gdsii

### `GDSIIFile`

| Member | Type | Description |
|--------|------|-------------|
| `version` | `i16` | Format version |
| `library_name` | `String` | Library name |
| `units` | `(f64, f64)` | `(user_unit, database_unit)` in meters |
| `structures` | `Vec<GDSStructure>` | All cells |

| Method | Description |
|--------|-------------|
| `new(name)` | Create empty file |
| `read_from_file(path)` | Read `.gds` file |
| `write_to_file(path)` | Write `.gds` file |
| `read(reader)` | Read from any `Read` |
| `write(writer)` | Write to any `Write` |

### `GDSStructure`

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Cell name |
| `creation_time` | `GDSTime` | Creation timestamp |
| `modification_time` | `GDSTime` | Modification timestamp |
| `elements` | `Vec<GDSElement>` | Elements |

### `GDSElement`

```rust
pub enum GDSElement {
    Boundary(Boundary),   // polygon
    Path(GPath),          // wire/trace
    Text(GText),          // text label
    StructRef(StructRef), // cell instance (SREF)
    ArrayRef(ArrayRef),   // cell array (AREF)
    Node(Node),           // net topology
    Box(GDSBox),          // box element
}
```

### `GDSTime`

```rust
GDSTime::now() -> Self
// fields: year, month, day, hour, minute, second: i16
```

---

## oasis

### `OASISFile`

| Method | Description |
|--------|-------------|
| `new()` | Create empty file |
| `read_from_file(path)` | Read `.oas` file |
| `write_to_file(path)` | Write `.oas` file |

### `OASISElement`

```rust
pub enum OASISElement {
    Rectangle(Rectangle),
    Polygon(Polygon),
    Path(OPath),
    Trapezoid(Trapezoid),
    CTrapezoid(CTrapezoid),
    Circle(Circle),
    Text(OText),
    Placement(Placement),
}
```

---

## converter

```rust
converter::gdsii_to_oasis(gds: &GDSIIFile) -> Result<OASISFile, _>
converter::oasis_to_gdsii(oasis: &OASISFile) -> Result<GDSIIFile, _>
```

---

## geometry

```rust
// Bounding box
bounding_box(points: &[(f64, f64)]) -> Option<BoundingBox>
bounding_box_i32(points: &[(i32, i32)]) -> Option<BoundingBox>
gds_element_bounding_box(element: &GDSElement) -> Option<BoundingBox>
structure_bounding_box(structure: &GDSStructure) -> Option<BoundingBox>
library_bounding_box(library: &GDSIIFile) -> Option<BoundingBox>
oasis_element_bounding_box(element: &OASISElement) -> Option<BoundingBox>

// BoundingBox methods
bb.width() / bb.height() / bb.area() / bb.center()
bb.union(other) / bb.intersect(other)
bb.contains_point(x, y) / bb.expand(margin)

// Polygon metrics
polygon_area(points) -> f64
polygon_signed_area(points) -> f64
polygon_perimeter(points) -> f64
polygon_centroid(points) -> (f64, f64)

// Point queries
point_in_polygon(pt, polygon) -> bool
point_in_any_polygon(pt, polygons) -> bool
inside(points, polygons) -> Vec<bool>

// Transforms
translate(points, dx, dy) -> Vec<(f64, f64)>
rotate(points, angle, cx, cy) -> Vec<(f64, f64)>
scale(points, sx, sy, cx, cy) -> Vec<(f64, f64)>
mirror_x(points, axis_y) -> Vec<(f64, f64)>
mirror_y(points, axis_x) -> Vec<(f64, f64)>
affine_transform(points, matrix) -> Vec<(f64, f64)>

// Utilities
is_counter_clockwise(points) -> bool
ensure_counter_clockwise(points) -> Vec<(f64, f64)>
ensure_clockwise(points) -> Vec<(f64, f64)>
close_polygon(points) -> Vec<(f64, f64)>
remove_duplicates(points) -> Vec<(f64, f64)>
distance(a, b) -> f64

// Advanced
fillet(points, radius, points_per_arc) -> Vec<(f64, f64)>
fracture_to_rectangles(points) -> Vec<Vec<(f64, f64)>>
```

---

## boolean_ops

```rust
// High-level boolean on sets of polygons
boolean(a, b, op: BooleanOp) -> Vec<Vec<(f64, f64)>>
// BooleanOp: Or | And | Not | Xor

// Offset (expand/shrink)
offset(polygons, distance: f64, tolerance: f64) -> Vec<Vec<(f64, f64)>>

// Slice along axis
slice(polygons, position: f64, axis: Axis) -> (Vec<Vec<(f64, f64)>>, Vec<Vec<(f64, f64)>>)
// Axis: X | Y

// Convex hull
convex_hull(points: &[(f64, f64)]) -> Vec<(f64, f64)>
```

---

## flexpath

```rust
// FlexPath — stroke-style path converted to polygon
let mut path = FlexPath::new(start, width, layer, datatype);
path.segment(end, width, offset, relative);
path.arc(radius, initial_angle, final_angle, width);
path.bezier(ctrl1, ctrl2, end, width, steps);

path.end_caps = (EndCap::Flush, EndCap::Round);
path.join = Join::Miter;

path.to_polygon() -> Option<Vec<(f64, f64)>>
path.bounding_box() -> Option<BoundingBox>
path.length() -> f64

// EndCap: Flush | HalfWidth | Extended(f64) | Round
// Join:   Natural | Miter | Bevel | Round
```

---

## curve

```rust
// Curve builder
let mut c = Curve::new(start);
c.line(end, relative);
c.arc(radius, initial_angle, final_angle);
c.bezier2(ctrl, end);          // quadratic
c.bezier3(ctrl1, ctrl2, end);  // cubic
c.smooth_bezier(ctrl2, end);   // smooth cubic
c.ellipse_arc(rx, ry, angle, large_arc, sweep, end);
c.interpolate(points, tension);
c.close();
c.get_points() -> &[(f64, f64)]

// Standalone primitives
ellipse(center, rx, ry, angle, tolerance) -> Vec<(f64, f64)>
regular_polygon(center, radius, sides, angle) -> Vec<(f64, f64)>
rounded_rectangle(corner, width, height, radius, tolerance) -> Vec<(f64, f64)>
star(center, r_inner, r_outer, points, angle) -> Vec<(f64, f64)>
spiral(center, r_start, r_end, turns, tolerance) -> Vec<(f64, f64)>
```

---

## topology

```rust
top_level_cells(library) -> Vec<&GDSStructure>
direct_references(structure) -> Vec<String>
cell_dependencies(name, library) -> HashSet<String>
dependency_order(library) -> Vec<usize>
detect_cycles(library) -> Vec<Vec<String>>
validate_hierarchy(library) -> Result<(), Vec<String>>

flatten_structure(name, library, max_depth) -> Vec<GDSElement>
merge_library(target, source) -> usize           // skip duplicates
merge_library_overwrite(target, source) -> usize // overwrite duplicates

filter_by_layer(structure, layer) -> Vec<&GDSElement>
element_layer(element) -> Option<i16>
layers_in_structure(structure) -> Vec<i16>
layers_in_library(library) -> Vec<i16>
total_element_count(library) -> usize
```

---

## streaming

```rust
let mut reader = StreamingGDSIIReader::new(file);
reader.read_with_callback(&mut my_callback)?;

// Implement the callback trait:
trait GDSIIStreamCallback {
    fn on_structure_start(&mut self, name: &str);
    fn on_element(&mut self, element: &GDSElement);
    fn on_structure_end(&mut self);
}

// Built-in collectors:
StructureNameCollector   // collects all cell names
StatisticsCollector      // counts elements per structure
```

---

## Error Handling

All I/O operations return `Result<T, Box<dyn std::error::Error>>`:

```rust
match GDSIIFile::read_from_file("design.gds") {
    Ok(gds)  => println!("{} structures", gds.structures.len()),
    Err(e)   => eprintln!("Error: {}", e),
}
```
