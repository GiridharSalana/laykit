# Property Utilities

LayKit provides utilities for working with GDSII and OASIS properties, making it easier to add, query, and manage metadata.

## Overview

Properties are metadata attached to layout elements. They're used for:
- Design annotations
- Manufacturing instructions
- Verification markers
- Tool-specific data
- Custom metadata

## Property Builder (GDSII)

The `PropertyBuilder` provides a fluent interface for creating properties:

```rust
use laykit::PropertyBuilder;

let properties = PropertyBuilder::new()
    .add(1, "DEVICE_TYPE=NMOS".to_string())
    .add(2, "WIDTH=10um".to_string())
    .add(3, "LENGTH=0.5um".to_string())
    .build();

// Use in an element
let boundary = Boundary {
    layer: 1,
    datatype: 0,
    xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
    properties,
};
```

## Property Manager

The `PropertyManager` helps query and manipulate existing properties:

```rust
use laykit::PropertyManager;

// Create from existing properties
let manager = PropertyManager::from_properties(&boundary.properties);

// Query properties
if let Some(value) = manager.get(1) {
    println!("Device type: {}", value);
}

// Check existence
if manager.has_property(2) {
    println!("Width property exists");
}

// Get all attributes
let attrs = manager.attributes();
println!("Property attributes: {:?}", attrs);

// Convert back to property list
let updated_props = manager.to_properties();
```

## OASIS Properties

OASIS properties support multiple value types:

```rust
use laykit::OASISPropertyBuilder;

let properties = OASISPropertyBuilder::new()
    .add_string("name".to_string(), "MyDevice".to_string())
    .add_integer("count".to_string(), 42)
    .add_real("temperature".to_string(), 25.5)
    .add_boolean("verified".to_string(), true)
    .build();

// Use in OASIS element
let rectangle = Rectangle {
    layer: 1,
    datatype: 0,
    x: 0,
    y: 0,
    width: 1000,
    height: 500,
    repetition: None,
    properties,
};
```

## Common Use Cases

### Device Annotations

```rust
fn annotate_device(boundary: &mut Boundary, device_type: &str, params: &HashMap<String, String>) {
    let mut builder = PropertyBuilder::new();
    
    // Add device type
    builder = builder.add(1, format!("TYPE={}", device_type));
    
    // Add parameters
    for (i, (key, value)) in params.iter().enumerate() {
        builder = builder.add((i + 2) as i16, format!("{}={}", key, value));
    }
    
    boundary.properties = builder.build();
}
```

### Manufacturing Instructions

```rust
fn add_manufacturing_notes(element: &mut GDSElement, notes: Vec<String>) {
    let mut builder = PropertyBuilder::new();
    
    for (i, note) in notes.iter().enumerate() {
        builder = builder.add((i + 100) as i16, note.clone());
    }
    
    match element {
        GDSElement::Boundary(b) => b.properties = builder.build(),
        GDSElement::Path(p) => p.properties = builder.build(),
        _ => {}
    }
}
```

### Property Filtering

```rust
fn filter_by_property(structure: &GDSStructure, attr: i16, value: &str) -> Vec<&GDSElement> {
    structure.elements.iter()
        .filter(|elem| {
            let props = match elem {
                GDSElement::Boundary(b) => &b.properties,
                GDSElement::Path(p) => &p.properties,
                _ => return false,
            };
            
            let manager = PropertyManager::from_properties(props);
            manager.get(attr) == Some(value)
        })
        .collect()
}
```

### Property Extraction

```rust
fn extract_all_properties(file: &GDSIIFile) -> HashMap<String, Vec<String>> {
    let mut property_map = HashMap::new();
    
    for structure in &file.structures {
        for element in &structure.elements {
            let props = match element {
                GDSElement::Boundary(b) => &b.properties,
                GDSElement::Path(p) => &p.properties,
                GDSElement::Text(t) => &t.properties,
                _ => continue,
            };
            
            let manager = PropertyManager::from_properties(props);
            for attr in manager.attributes() {
                if let Some(value) = manager.get(attr) {
                    property_map
                        .entry(format!("ATTR_{}", attr))
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
                }
            }
        }
    }
    
    property_map
}
```

## Property Best Practices

### Attribute Numbering

Use consistent attribute numbering:

```rust
// Define constants for property attributes
const PROP_DEVICE_TYPE: i16 = 1;
const PROP_WIDTH: i16 = 2;
const PROP_LENGTH: i16 = 3;
const PROP_MANUFACTURER: i16 = 100;
const PROP_LOT_NUMBER: i16 = 101;

let props = PropertyBuilder::new()
    .add(PROP_DEVICE_TYPE, "NMOS".to_string())
    .add(PROP_WIDTH, "10".to_string())
    .add(PROP_MANUFACTURER, "ACME Corp".to_string())
    .build();
```

### Value Formatting

Use consistent value formatting:

```rust
// Good: Structured format
builder.add(1, format!("DEVICE=NMOS WIDTH={}um LENGTH={}um", w, l));

// Good: Key-value pairs
builder.add(1, format!("device:NMOS"));
builder.add(2, format!("width:{}um", w));

// Avoid: Inconsistent formats
builder.add(1, format!("NMOS {} {}", w, l)); // Hard to parse
```

### Property Validation

Validate properties after loading:

```rust
fn validate_properties(element: &GDSElement) -> Result<(), String> {
    let props = match element {
        GDSElement::Boundary(b) => &b.properties,
        _ => return Ok(()),
    };
    
    let manager = PropertyManager::from_properties(props);
    
    // Check required properties
    if !manager.has_property(PROP_DEVICE_TYPE) {
        return Err("Missing DEVICE_TYPE property".to_string());
    }
    
    // Validate values
    if let Some(width) = manager.get(PROP_WIDTH) {
        if width.parse::<f64>().is_err() {
            return Err("Invalid WIDTH value".to_string());
        }
    }
    
    Ok(())
}
```

## Performance Notes

- Properties are stored as vectors (O(n) lookup)
- PropertyManager uses HashMap for fast access (O(1) average)
- Use PropertyManager for frequent queries
- Use PropertyBuilder for construction
- Properties add minimal overhead to file size

## GDSII vs OASIS Properties

| Feature | GDSII | OASIS |
|---------|-------|-------|
| **Attribute** | Integer (i16) | String name |
| **Value Type** | String only | Multiple types |
| **Size** | Fixed overhead | Variable |
| **Lookup** | By number | By name |

## Future Enhancements

Planned improvements:
- Property schema validation
- Type-safe property accessors
- Property template system
- Conversion utilities between GDSII/OASIS properties

