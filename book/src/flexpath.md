# FlexPath & Curves

## FlexPath

`FlexPath` generates a filled polygon from a centerline path, like a stroke with configurable width, join style, and end caps.

```rust
use laykit::{FlexPath, EndCap, Join};
use std::f64::consts::PI;

let mut path = FlexPath::new((0.0, 0.0), 2.0, 1, 0);
// new(start, width, layer, datatype)

// Add segments
path.segment((10.0, 0.0), None, None, false);     // absolute
path.segment((5.0, 0.0), Some(3.0), None, true);  // relative, taper to width 3

// Add an arc
path.arc(5.0, 0.0, PI / 2.0, None); // radius, initial_angle, final_angle

// Add a cubic Bezier
path.bezier((2.0, 4.0), (8.0, 4.0), (10.0, 0.0), None, 32);

// Styling
path.end_caps = (EndCap::Round, EndCap::Flush);
path.join = Join::Miter;

// Convert to polygon
let poly = path.to_polygon().unwrap();
let bb   = path.bounding_box().unwrap();
let len  = path.length();
```

### EndCap variants

| Variant | Description |
|---------|-------------|
| `Flush` | Square cap flush with end point |
| `HalfWidth` | Square cap extended by half the path width |
| `Extended(f64)` | Square cap extended by a fixed distance |
| `Round` | Semicircular cap |

### Join variants

| Variant | Description |
|---------|-------------|
| `Natural` | No extra join geometry |
| `Miter` | Sharp corner (default) |
| `Bevel` | Clipped corner |
| `Round` | Circular join |

## RobustPath

`RobustPath` wraps `FlexPath` and adds self-intersection checking (for complex paths that loop back on themselves):

```rust
use laykit::RobustPath;

let mut rp = RobustPath::new((0.0, 0.0), 1.0, 1, 0);
rp.segment((10.0, 0.0), None, false);
let poly = rp.to_polygon().unwrap();
```

---

## Curve

`Curve` builds a polyline from a sequence of geometric primitives, which can then be used as a path centerline or polygon outline.

```rust
use laykit::Curve;
use std::f64::consts::PI;

let mut c = Curve::new((0.0, 0.0));

c.line((10.0, 0.0), false);                      // line to absolute point
c.arc(5.0, 0.0, PI / 2.0);                       // arc by angles
c.bezier2((5.0, 5.0), (10.0, 0.0));              // quadratic Bezier
c.bezier3((2.0,4.0), (8.0,4.0), (10.0,0.0));    // cubic Bezier
c.smooth_bezier((8.0, 4.0), (10.0, 0.0));        // smooth cubic (auto first ctrl)
c.ellipse_arc(5.0, 3.0, 0.0, false, true, (15.0, 0.0)); // SVG-style elliptical arc
c.interpolate(&points, 0.0);                      // Catmull-Rom spline
c.close();

let pts = c.get_points();
let len  = c.length();
```

## Standalone Shape Functions

```rust
use laykit::{ellipse, regular_polygon, rounded_rectangle, star, spiral};

// Ellipse
let pts = ellipse((0.0,0.0), 10.0, 5.0, 0.0, 0.01);
// center, rx, ry, initial_angle, tolerance

// Regular polygon (e.g. hexagon)
let hex = regular_polygon((0.0,0.0), 5.0, 6, 0.0);
// center, circumradius, sides, initial_angle

// Rounded rectangle
let rr = rounded_rectangle((0.0,0.0), 20.0, 10.0, 2.0, 0.01);
// corner, width, height, corner_radius, tolerance

// Star
let s = star((0.0,0.0), 2.0, 5.0, 5, 0.0);
// center, inner_radius, outer_radius, points, initial_angle

// Spiral
let sp = spiral((0.0,0.0), 1.0, 10.0, 3.0, 0.01);
// center, r_start, r_end, turns, tolerance
```
