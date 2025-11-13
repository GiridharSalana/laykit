# Testing

LayKit includes a comprehensive test suite to ensure reliability and correctness.

## Running Tests

### All Tests

Run the complete test suite:

```bash
cargo test
```

Expected output:
```
running 21 tests
test test_gdsii_create_and_write ... ok
test test_gdsii_round_trip ... ok
test test_oasis_round_trip_rectangles ... ok
...
test result: ok. 21 passed; 0 failed; 0 ignored
```

### With Output

See test output:

```bash
cargo test -- --nocapture
```

### Specific Tests

Run a single test:

```bash
cargo test test_gdsii_round_trip
```

Run tests matching a pattern:

```bash
cargo test gdsii
cargo test oasis
cargo test conversion
```

### Doc Tests

Run documentation examples:

```bash
cargo test --doc
```

## Test Categories

### GDSII Tests (7 tests)

**1. test_gdsii_create_and_write**
- Creates a GDSII file from scratch
- Writes to disk
- Verifies file creation

**2. test_gdsii_round_trip**
- Creates file, writes it, reads it back
- Verifies data integrity
- Checks all fields match

**3. test_gdsii_text_element**
- Tests text label creation
- Verifies text properties
- Checks positioning

**4. test_gdsii_struct_ref**
- Tests structure references
- Verifies hierarchical designs
- Checks cell instantiation

**5. test_gdsii_empty_structure**
- Tests empty structures
- Verifies edge case handling

**6. test_gdsii_multiple_layers**
- Tests multi-layer designs
- Verifies layer/datatype handling

**7. test_gdsii_complex_polygon**
- Tests complex geometries
- Verifies octagon creation
- Checks coordinate handling

### OASIS Tests (11 tests)

**1. test_oasis_create_simple**
- Creates basic OASIS file
- Verifies structure

**2. test_oasis_round_trip_rectangles**
- Tests rectangle primitive
- Verifies read/write cycle

**3. test_oasis_polygon_round_trip**
- Tests polygon elements
- Verifies coordinate conversion

**4. test_oasis_path_round_trip**
- Tests path elements
- Verifies width handling

**5. test_oasis_mixed_elements**
- Tests multiple element types
- Verifies heterogeneous designs

**6. test_oasis_empty_cell**
- Tests empty cells
- Verifies edge cases

**7. test_oasis_large_coordinates**
- Tests large coordinate values
- Verifies i64 handling

**8. test_oasis_negative_coordinates**
- Tests negative coordinates
- Verifies signed integer handling

**9. test_oasis_read_write**
- Basic I/O test
- Verifies file operations

**10. test_oasis_multiple_cells**
- Tests multiple cells
- Verifies cell management

**11. test_oasis_element_types**
- Tests all element types
- Comprehensive element verification

### Converter Tests (3 tests)

**1. test_gdsii_to_oasis_conversion**
- Converts GDSII to OASIS
- Verifies element mapping
- Checks cell conversion

**2. test_oasis_to_gdsii_conversion**
- Converts OASIS to GDSII
- Verifies reverse conversion
- Checks boundary creation

**3. test_rectangle_detection**
- Tests automatic rectangle detection
- Verifies smart conversion
- Checks optimization

## Writing Tests

### Basic Test Structure

```rust
#[test]
fn test_my_feature() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    let mut gds = GDSIIFile::new("TEST".to_string());
    
    // Execute
    gds.structures.push(create_test_structure());
    
    // Verify
    assert_eq!(gds.structures.len(), 1);
    assert_eq!(gds.structures[0].name, "TEST_CELL");
    
    Ok(())
}
```

### Testing File I/O

```rust
#[test]
fn test_file_write_read() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = "test_output.gds";
    
    // Create and write
    let gds = create_test_file();
    gds.write_to_file(test_file)?;
    
    // Read back
    let gds_read = GDSIIFile::read_from_file(test_file)?;
    
    // Verify
    assert_eq!(gds.library_name, gds_read.library_name);
    assert_eq!(gds.structures.len(), gds_read.structures.len());
    
    // Cleanup
    std::fs::remove_file(test_file)?;
    
    Ok(())
}
```

### Testing Error Cases

```rust
#[test]
fn test_file_not_found() {
    let result = GDSIIFile::read_from_file("nonexistent.gds");
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "Invalid format")]
fn test_invalid_format() {
    // Test code that should panic
}
```

### Testing Conversions

```rust
#[test]
fn test_conversion_preserves_data() -> Result<(), Box<dyn std::error::Error>> {
    // Create GDSII with known data
    let gds = create_test_gdsii();
    let original_count = count_elements(&gds);
    
    // Convert to OASIS
    let oasis = converter::gdsii_to_oasis(&gds)?;
    let oasis_count = count_oasis_elements(&oasis);
    
    // Verify element count preserved
    assert_eq!(original_count, oasis_count);
    
    // Convert back
    let gds2 = converter::oasis_to_gdsii(&oasis)?;
    let final_count = count_elements(&gds2);
    
    // Verify round-trip
    assert_eq!(original_count, final_count);
    
    Ok(())
}
```

## Test Coverage

Current test coverage by module:

| Module | Lines | Coverage |
|--------|-------|----------|
| gdsii.rs | ~1000 | ~85% |
| oasis.rs | ~950 | ~80% |
| converter.rs | ~300 | ~90% |
| **Total** | **~2250** | **~85%** |

### Untested Areas

Areas that could use more testing:
- Large file handling (>100MB)
- Corrupt file recovery
- Edge cases with extreme coordinates
- Performance regression tests
- Concurrent access patterns

## Continuous Integration

The project uses GitHub Actions for CI:

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --verbose
      - run: cargo test --doc --verbose
```

Tests run on:
- ✅ Every push to main
- ✅ Every pull request
- ✅ Before releases

## Test Data

### Generating Test Files

```rust
fn create_test_gdsii() -> GDSIIFile {
    let mut gds = GDSIIFile::new("TEST_LIB".to_string());
    gds.units = (1e-6, 1e-9);
    
    let mut structure = GDSStructure {
        name: "TEST_CELL".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        elements: Vec::new(),
    };
    
    // Add test elements
    structure.elements.push(GDSElement::Boundary(Boundary {
        layer: 1,
        datatype: 0,
        xy: vec![(0,0), (100,0), (100,100), (0,100), (0,0)],
        properties: Vec::new(),
    }));
    
    gds.structures.push(structure);
    gds
}
```

### Test File Cleanup

```rust
struct TestFile {
    path: String,
}

impl TestFile {
    fn new(name: &str) -> Self {
        TestFile { path: name.to_string() }
    }
}

impl Drop for TestFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[test]
fn test_with_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let _test_file = TestFile::new("test.gds");
    
    // Test code...
    // File automatically deleted when _test_file goes out of scope
    
    Ok(())
}
```

## Benchmarking

### Using Criterion

Add to `Cargo.toml`:

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "laykit_bench"
harness = false
```

Create `benches/laykit_bench.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use laykit::GDSIIFile;

fn benchmark_read(c: &mut Criterion) {
    c.bench_function("gdsii_read", |b| {
        b.iter(|| {
            let gds = GDSIIFile::read_from_file(black_box("test.gds"));
            gds.unwrap()
        })
    });
}

criterion_group!(benches, benchmark_read);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench
```

## Testing Best Practices

1. **Test one thing at a time**
   ```rust
   #[test]
   fn test_boundary_layer() { /* Test only layer property */ }
   
   #[test]
   fn test_boundary_coordinates() { /* Test only coordinates */ }
   ```

2. **Use descriptive names**
   ```rust
   // Good
   #[test]
   fn test_gdsii_round_trip_preserves_layer_numbers() { }
   
   // Bad
   #[test]
   fn test1() { }
   ```

3. **Clean up resources**
   ```rust
   #[test]
   fn test_cleanup() -> Result<(), Box<dyn std::error::Error>> {
       let file = "test.gds";
       // test code...
       std::fs::remove_file(file)?; // Cleanup
       Ok(())
   }
   ```

4. **Test error paths**
   ```rust
   #[test]
   fn test_invalid_input_returns_error() {
       let result = process_invalid_data();
       assert!(result.is_err());
   }
   ```

5. **Use assertions effectively**
   ```rust
   assert_eq!(actual, expected, "Values should match");
   assert!(condition, "Condition should be true");
   assert_ne!(a, b, "Values should differ");
   ```

## Contributing Tests

When contributing, ensure:
- ✅ New features have tests
- ✅ Bug fixes have regression tests
- ✅ Tests pass locally
- ✅ Tests pass in CI
- ✅ Code coverage doesn't decrease
