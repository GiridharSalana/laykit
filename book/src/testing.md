# Testing

## Running Tests

```bash
# Full suite
cargo test

# With printed output
cargo test -- --nocapture

# Single test
cargo test test_gdsii_round_trip

# By module
cargo test geometry
cargo test boolean_ops
cargo test flexpath
cargo test topology

# Doc tests only
cargo test --doc
```

## Test Counts

| Suite | Tests |
|-------|-------|
| Unit tests (all modules) | 115 |
| Integration tests | 41 |
| Doc tests | 8 |
| **Total** | *run `cargo test` locally* |

Run `cargo test` and `cargo test --doc` from the repository root for current counts. The release workflow runs the same checks when you push a version tag. One dependency (`miniz_oxide` for OASIS CBLOCK).

## Coverage by Module

| Module | Tests |
|--------|-------|
| `gdsii` | 7 |
| `oasis` | 11 |
| `converter` | 3 |
| `geometry` | 22 |
| `boolean_ops` | 12 |
| `flexpath` | 10 |
| `curve` | 9 |
| `topology` | 13 |
| `streaming` | 8 |
| `aref_expansion` | 6 |
| `properties` | 4 |
| `format_detection` | 6 |
| Integration (cross-module) | 41 |
| Doc tests | 8 |

## Writing Tests

```rust
#[test]
fn test_my_feature() {
    let mut gds = GDSIIFile::new("TEST".to_string());
    gds.structures.push(GDSStructure {
        name: "CELL".to_string(),
        creation_time: GDSTime::now(),
        modification_time: GDSTime::now(),
        strclass: None,
        elements: Vec::new(),
    });
    assert_eq!(gds.structures.len(), 1);
}
```

For file I/O tests, clean up after yourself:

```rust
#[test]
fn test_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let path = "test_rt.gds";
    let gds = make_test_file();
    gds.write_to_file(path)?;
    let read = GDSIIFile::read_from_file(path)?;
    std::fs::remove_file(path)?;
    assert_eq!(gds.library_name, read.library_name);
    Ok(())
}
```
