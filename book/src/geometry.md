# Geometry

The `geometry` module provides geometric primitives and operations on polygons.

## Bounding Box

```rust
use laykit::{bounding_box, BoundingBox};

let pts = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 5.0), (0.0, 5.0)];
let bb = bounding_box(&pts).unwrap();

println!("width:  {}", bb.width());   // 10.0
println!("height: {}", bb.height());  // 5.0
println!("area:   {}", bb.area());    // 50.0
println!("center: {:?}", bb.center()); // (5.0, 2.5)

// Combine two bounding boxes
let bb2 = BoundingBox { x_min: 5.0, x_max: 20.0, y_min: -2.0, y_max: 3.0 };
let merged = bb.union(&bb2);

// Per-element helpers
use laykit::{gds_element_bounding_box, structure_bounding_box, library_bounding_box};
```

## Polygon Metrics

```rust
use laykit::{polygon_area, polygon_perimeter, polygon_centroid, polygon_signed_area};

let square = vec![(0.0,0.0),(10.0,0.0),(10.0,10.0),(0.0,10.0)];

println!("area:      {}", polygon_area(&square));       // 100.0
println!("perimeter: {}", polygon_perimeter(&square));  // 40.0
println!("centroid:  {:?}", polygon_centroid(&square)); // (5.0, 5.0)

// Positive = CCW, negative = CW
let signed = polygon_signed_area(&square);
```

## Point-in-Polygon

```rust
use laykit::{point_in_polygon, inside};

let poly = vec![(0.0,0.0),(10.0,0.0),(10.0,10.0),(0.0,10.0)];

assert!(point_in_polygon((5.0, 5.0), &poly));   // inside
assert!(!point_in_polygon((15.0, 5.0), &poly)); // outside

// Batch query across multiple polygons
let results = inside(&[(5.0,5.0),(20.0,20.0)], &[poly]);
// results = [true, false]
```

## Transforms

```rust
use laykit::{translate, rotate, scale, mirror_x, mirror_y, affine_transform};

let pts = vec![(1.0, 0.0), (2.0, 0.0)];

let moved    = translate(&pts, 5.0, 3.0);
let rotated  = rotate(&pts, std::f64::consts::PI / 2.0, 0.0, 0.0); // 90°
let scaled   = scale(&pts, 2.0, 2.0, 0.0, 0.0);
let flipped  = mirror_x(&pts, 0.0); // reflect over y = 0
let flipped2 = mirror_y(&pts, 0.0); // reflect over x = 0

// 2×3 affine matrix [a, b, c, d, tx, ty]
let mat = [1.0_f64, 0.0, 0.0, 1.0, 5.0, 3.0]; // pure translation
let transformed = affine_transform(&pts, &mat);
```

## Orientation & Utilities

```rust
use laykit::{is_counter_clockwise, ensure_counter_clockwise, fillet, fracture_to_rectangles};

let pts = vec![(0.0,0.0),(10.0,0.0),(10.0,10.0),(0.0,10.0)];

// Check/enforce winding order
let ccw = is_counter_clockwise(&pts);
let enforced = ensure_counter_clockwise(&pts);

// Round polygon corners (radius, points per arc quadrant)
let rounded = fillet(&pts, 1.0, 8);

// Decompose polygon into non-overlapping rectangles
let rects = fracture_to_rectangles(&pts);
```
