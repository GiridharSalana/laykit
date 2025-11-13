# Roadmap

Future plans and development priorities for LayKit.

## Version History

### v0.2.x (Current) ✅

**Status:** Production-ready with comprehensive documentation

**Features:**
- ✅ Complete GDSII read/write support
- ✅ Complete OASIS read/write support
- ✅ Bidirectional format conversion
- ✅ 21 comprehensive tests (100% passing)
- ✅ Zero dependencies
- ✅ Full mdBook documentation
- ✅ GitHub Actions CI/CD
- ✅ Automated releases

**Recent improvements:**
- mdBook documentation system
- Enhanced API documentation
- Performance benchmarks documented
- Contributing guidelines

### v0.1.x

**Initial release** with core functionality:
- Basic GDSII support
- Basic OASIS support
- Simple conversions
- Initial test suite

## Short-term Goals (v0.3.0)

**Target:** Q2 2025

### 1. Command-Line Interface

**Priority:** High  
**Complexity:** Medium

Add a CLI tool for common operations:

```bash
# Convert formats
laykit convert input.gds output.oas

# Display file information
laykit info design.gds

# Validate files
laykit validate layout.oas

# Extract statistics
laykit stats --layers --elements design.gds
```

**Implementation:**
- Use `clap` for argument parsing
- Support batch operations
- Progress indicators for large files
- JSON output option

### 2. Streaming Parser

**Priority:** High  
**Complexity:** High

For files >1GB that don't fit in memory:

```rust
// Streaming read API
let mut stream = GDSIIStream::open("huge.gds")?;
for structure in stream.structures() {
    // Process one structure at a time
    process_structure(structure?)?;
}
```

**Benefits:**
- Handle unlimited file sizes
- Constant memory usage
- Faster startup time

### 3. Enhanced Property Support

**Priority:** Medium  
**Complexity:** Low

Better property handling:

```rust
// Property builder
let prop = PropertyBuilder::new()
    .name("PROPERTY_1")
    .value(PropertyValue::String("value"))
    .build();

// Property queries
let has_prop = element.has_property("PROPERTY_1");
let value = element.get_property("PROPERTY_1")?;
```

### 4. AREF Expansion

**Priority:** Medium  
**Complexity:** Medium

Expand array references to individual instances:

```rust
let expanded = gds.expand_arrays()?;
// All ArrayRef elements become individual StructRef elements
```

## Medium-term Goals (v0.4.0)

**Target:** Q4 2025

### 1. Performance Optimizations

**SIMD Acceleration**
- Use SIMD for coordinate processing
- Vectorize transformation calculations
- Speed up bounding box calculations

**Parallel Processing**
- Multi-threaded file parsing
- Parallel structure processing
- Concurrent conversion

**Implementation:**
```rust
// Parallel structure processing
use rayon::prelude::*;

gds.structures.par_iter_mut()
    .for_each(|structure| {
        transform_structure(structure);
    });
```

### 2. Validation Tools

**Design Rule Checking (DRC)**

Basic geometric validations:
- Layer presence checking
- Minimum feature size
- Spacing rules
- Overlap detection

**Hierarchy Validation**
- Circular reference detection
- Missing cell detection
- Unused structure identification

**Implementation:**
```rust
let validator = Validator::new()
    .min_width(layer=1, width=100)
    .min_spacing(layer=1, spacing=50);

let violations = validator.check(&gds)?;
for violation in violations {
    println!("Violation: {}", violation);
}
```

### 3. Layer Map Support

**Priority:** Medium  
**Complexity:** Low

Layer mapping for conversions:

```rust
let layer_map = LayerMap::new()
    .map(1, 10)  // Layer 1 → 10
    .map(2, 20)  // Layer 2 → 20
    .default_offset(100); // Others → layer + 100

let gds = apply_layer_map(&gds, &layer_map)?;
```

### 4. Geometry Operations

**Priority:** Medium  
**Complexity:** High

Basic geometric operations:
- Boolean operations (AND, OR, NOT, XOR)
- Offset/buffer operations
- Polygon simplification
- Bounding box calculations

```rust
let result = gds_bool_operation(&gds1, &gds2, BoolOp::Union)?;
```

## Long-term Goals (v1.0.0+)

**Target:** 2026+

### 1. WebAssembly Support

**Priority:** Medium  
**Complexity:** High

Compile to WebAssembly for browser usage:

```javascript
import init, { GDSIIFile } from './laykit.js';

await init();
const gds = GDSIIFile.read_from_file('design.gds');
console.log(gds.library_name);
```

**Use cases:**
- Web-based IC layout viewers
- Online conversion tools
- Browser-based validation
- Cloud processing

### 2. GUI Viewer

**Priority:** Low  
**Complexity:** Very High

Simple layout visualization:

**Features:**
- Pan and zoom
- Layer visibility control
- Cell hierarchy browser
- Measurement tools
- Export to images

**Technology options:**
- egui (immediate mode GUI)
- iced (native GUI)
- Web-based (with WASM)

### 3. Python Bindings

**Priority:** Medium  
**Complexity:** Medium

Python interface via PyO3:

```python
import laykit

gds = laykit.GDSIIFile.read_from_file("design.gds")
print(f"Library: {gds.library_name}")

oasis = laykit.convert.gdsii_to_oasis(gds)
oasis.write_to_file("output.oas")
```

**Benefits:**
- Integration with Python EDA tools
- Jupyter notebook support
- NumPy integration for coordinates
- Matplotlib for visualization

### 4. Advanced Features

**Incremental File Updates**
- Modify files without full rewrite
- Append structures
- Update metadata

**Partial File Reading**
- Read specific structures only
- Region-of-interest extraction
- Metadata-only reading

**Compression Support**
- Gzip compressed files
- Streaming decompression
- Transparent handling

## Community Requests

Track feature requests from users:

### High Demand
- ⭐⭐⭐ CLI tool
- ⭐⭐⭐ Streaming parser
- ⭐⭐ Python bindings

### Medium Demand
- ⭐⭐ GUI viewer
- ⭐⭐ Layer mapping
- ⭐ Boolean operations

### Under Consideration
- Format migration tools
- DRC capabilities
- Custom element types
- Plugin system

## Performance Targets

### Current Performance
- Read: ~20-25 MB/s
- Write: ~25-30 MB/s
- Memory: ~30× file size

### Target Performance (v0.4.0)
- Read: ~50 MB/s (2× faster)
- Write: ~60 MB/s (2× faster)
- Memory: ~15× file size (50% reduction)

## Breaking Changes Policy

### Semantic Versioning

LayKit follows [Semantic Versioning](https://semver.org/):

- **Major (1.0.0)** - Breaking API changes
- **Minor (0.x.0)** - New features, backward compatible
- **Patch (0.0.x)** - Bug fixes

### Stability Guarantees

- **Pre-1.0:** May have breaking changes in minor versions
- **Post-1.0:** Breaking changes only in major versions

### Deprecation Policy

When removing features:
1. Mark as deprecated
2. Maintain for at least 2 minor versions
3. Provide migration guide
4. Remove in next major version

## Contributing to Roadmap

### Suggesting Features

Open a GitHub Issue with:
- Clear description
- Use cases
- Expected behavior
- Willingness to contribute

### Voting on Features

React to issues with 👍 to show support.

### Implementing Features

See [Contributing Guide](./contributing.md) for how to contribute code.

## Release Schedule

### Regular Releases
- **Patch releases:** As needed (bug fixes)
- **Minor releases:** Quarterly (new features)
- **Major releases:** Yearly (breaking changes)

### Release Process
1. Feature complete
2. Testing and bug fixing
3. Documentation updated
4. Changelog updated
5. Release candidate (if major)
6. Final release

## Backwards Compatibility

### What We Guarantee
- ✅ File format compatibility
- ✅ Public API stability (post-1.0)
- ✅ Semantic versioning

### What May Change
- ⚠️ Internal implementation
- ⚠️ Private APIs
- ⚠️ Performance characteristics
- ⚠️ Error messages

## Platform Support

### Current Support
- ✅ Linux (x86_64, ARM64)
- ✅ macOS (x86_64, ARM64)
- ✅ Windows (x86_64)

### Future Support
- BSD systems
- WebAssembly
- Mobile (iOS, Android) - low priority

## Dependencies

### Current Policy
- ✅ Zero runtime dependencies

### Future Considerations

May add optional dependencies for:
- CLI tool (clap, indicatif)
- SIMD operations (packed_simd)
- Parallel processing (rayon)
- Python bindings (pyo3)

All features requiring dependencies will be:
- Optional (feature flags)
- Well-documented
- Justified by clear benefits

## Stay Updated

Follow development:
- ⭐ [Star on GitHub](https://github.com/giridharsalana/laykit)
- 👁️ Watch releases
- 💬 Join [discussions](https://github.com/giridharsalana/laykit/discussions)
- 📖 Read [CHANGELOG](https://github.com/giridharsalana/laykit/blob/main/CHANGELOG.md)

## Questions?

- Open an [issue](https://github.com/giridharsalana/laykit/issues)
- Start a [discussion](https://github.com/giridharsalana/laykit/discussions)
- Check the [documentation](https://giridharsalana.github.io/laykit/)
