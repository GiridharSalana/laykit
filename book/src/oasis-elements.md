# OASIS Elements

Detailed reference for all OASIS element types supported by LayKit.

## Rectangle

Axis-aligned rectangle primitive - the most common shape in OASIS.

```rust
pub struct Rectangle {
    pub layer: u32,
    pub datatype: u32,
    pub x: i64,
    pub y: i64,
    pub width: u64,
    pub height: u64,
    pub repetition: Option<Repetition>,
    pub properties: Vec<OASISProperty>,
}
```

**Usage:** Optimized for rectangular shapes, much more compact than polygons.

**Example:**
```rust
Rectangle {
    layer: 1,
    datatype: 0,
    x: 0,
    y: 0,
    width: 1000,
    height: 500,
    repetition: None,
    properties: Vec::new(),
}
```

## Polygon

General polygon with arbitrary vertices.

```rust
pub struct Polygon {
    pub layer: u32,
    pub datatype: u32,
    pub x: i64,
    pub y: i64,
    pub points: Vec<(i64, i64)>,
    pub repetition: Option<Repetition>,
    pub properties: Vec<OASISProperty>,
}
```

**Usage:** For complex shapes that aren't rectangles or other primitives.

**Note:** Points are relative to (x, y).

## Path

Wire or trace element.

```rust
pub struct OPath {
    pub layer: u32,
    pub datatype: u32,
    pub width: u64,
    pub x: i64,
    pub y: i64,
    pub points: Vec<(i64, i64)>,
    pub start_extension: i64,
    pub end_extension: i64,
    pub repetition: Option<Repetition>,
    pub properties: Vec<OASISProperty>,
}
```

**Usage:** Representing wires, interconnects, and traces.

**Extensions:** Control how the path extends beyond its endpoints.

## Trapezoid

Trapezoidal shape primitive.

```rust
pub struct Trapezoid {
    pub layer: u32,
    pub datatype: u32,
    pub x: i64,
    pub y: i64,
    pub width: u64,
    pub height: u64,
    pub delta_a: i64,
    pub delta_b: i64,
    pub repetition: Option<Repetition>,
    pub properties: Vec<OASISProperty>,
}
```

**Usage:** Efficiently representing trapezoidal shapes common in IC layouts.

**Parameters:**
- `delta_a`: Horizontal offset at top
- `delta_b`: Horizontal offset at bottom

## CTrapezoid (Constrained Trapezoid)

Constrained trapezoid with specific geometry.

```rust
pub struct CTrapezoid {
    pub layer: u32,
    pub datatype: u32,
    pub x: i64,
    pub y: i64,
    pub width: u64,
    pub height: u64,
    pub ctrapezoid_type: u8,
    pub repetition: Option<Repetition>,
    pub properties: Vec<OASISProperty>,
}
```

**Usage:** Specific trapezoid types with predefined constraints for even more compact storage.

## Circle

Circle primitive.

```rust
pub struct Circle {
    pub layer: u32,
    pub datatype: u32,
    pub x: i64,
    pub y: i64,
    pub radius: u64,
    pub repetition: Option<Repetition>,
    pub properties: Vec<OASISProperty>,
}
```

**Usage:** Representing circular shapes, vias, and contacts.

**Note:** Center is at (x, y).

## Text

Text label element.

```rust
pub struct OText {
    pub layer: u32,
    pub texttype: u32,
    pub string: String,
    pub x: i64,
    pub y: i64,
    pub repetition: Option<Repetition>,
    pub properties: Vec<OASISProperty>,
}
```

**Usage:** Adding text annotations and labels to the layout.

## Placement

Cell instance (reference to another cell).

```rust
pub struct Placement {
    pub cell_name: String,
    pub x: i64,
    pub y: i64,
    pub magnification: Option<f64>,
    pub angle: Option<f64>,
    pub mirror_x: bool,
    pub repetition: Option<Repetition>,
    pub properties: Vec<OASISProperty>,
}
```

**Usage:** Creating hierarchical designs by instantiating other cells.

**Transformations:**
- `x, y`: Position
- `magnification`: Scaling factor
- `angle`: Rotation in degrees
- `mirror_x`: Mirror across X-axis

## Repetition

OASIS supports repetition patterns for creating arrays:

```rust
pub struct Repetition {
    pub x_dimension: u32,
    pub y_dimension: u32,
    pub x_space: i64,
    pub y_space: i64,
}
```

**Usage:** Efficiently representing repeated patterns without duplicating elements.

**Example:**
```rust
repetition: Some(Repetition {
    x_dimension: 10,  // 10 columns
    y_dimension: 5,   // 5 rows
    x_space: 1000,    // 1000nm spacing in X
    y_space: 2000,    // 2000nm spacing in Y
})
```

## Properties

Additional metadata attached to elements:

```rust
pub struct OASISProperty {
    pub name: String,
    pub values: Vec<PropertyValue>,
}

pub enum PropertyValue {
    Integer(i64),
    Real(f64),
    String(String),
    Boolean(bool),
}
```

## Coordinate System

- All coordinates are 64-bit signed integers (`i64`)
- Units defined by `OASISFile.unit` (typically in meters)
- Typical database unit: 1nm (1e-9 meters)
- Relative coordinates used in polygons and paths for compactness
