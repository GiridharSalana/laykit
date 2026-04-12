# Boolean Operations

The `boolean_ops` module provides polygon clipping, offsetting, slicing, and convex hull.

## Boolean Operations

```rust
use laykit::{boolean, BooleanOp};

let a = vec![vec![(0.0,0.0),(10.0,0.0),(10.0,10.0),(0.0,10.0)]];
let b = vec![vec![(5.0,0.0),(15.0,0.0),(15.0,10.0),(5.0,10.0)]];

let union  = boolean(&a, &b, BooleanOp::Or);   // A ∪ B
let inter  = boolean(&a, &b, BooleanOp::And);  // A ∩ B
let diff   = boolean(&a, &b, BooleanOp::Not);  // A − B
let xor    = boolean(&a, &b, BooleanOp::Xor);  // A △ B
```

Each function takes `&[Vec<(f64, f64)>]` (a slice of polygons) and returns `Vec<Vec<(f64, f64)>>`.

## Offset (Expand / Shrink)

```rust
use laykit::offset;

let polys = vec![vec![(0.0,0.0),(10.0,0.0),(10.0,10.0),(0.0,10.0)]];

// Expand outward by 1 unit
let expanded = offset(&polys, 1.0, 0.01);

// Shrink inward by 1 unit
let shrunk = offset(&polys, -1.0, 0.01);
```

The third argument (`tolerance`) controls arc discretisation for curved corners — smaller values give smoother results.

## Slice

```rust
use laykit::{slice, Axis};

let polys = vec![vec![(0.0,0.0),(10.0,0.0),(10.0,10.0),(0.0,10.0)]];

// Cut vertically at x = 5
let (left, right) = slice(&polys, 5.0, Axis::X);

// Cut horizontally at y = 4
let (below, above) = slice(&polys, 4.0, Axis::Y);
```

## Convex Hull

```rust
use laykit::convex_hull;

let pts = vec![(0.0,0.0),(5.0,0.0),(5.0,5.0),(0.0,5.0),(2.5,2.5)];
let hull = convex_hull(&pts);
// Returns the 4 outer corners; the interior point is excluded
```
