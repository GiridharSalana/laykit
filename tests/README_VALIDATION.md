# LayKit Validation Tests

## Overview

This directory contains cross-validation tests that compare LayKit's output against **gdstk**, the industry-standard Python library for GDSII/OASIS manipulation.

## Requirements

- Python 3.7+
- gdstk library

## Installation

Install gdstk:

```bash
pip install gdstk
```

Or with pip3:

```bash
pip3 install gdstk
```

## Running Validation Tests

### Locally

```bash
# From project root:

# Build LayKit first
cargo build --release

# Run validation tests only
cd tests && python3 gdstk_validation.py

# Or run ALL tests (Rust + validation)
tests/run_all_tests.sh
```

### From tests directory

```bash
cd tests

# Run validation only
python3 gdstk_validation.py

# Run all tests
./run_all_tests.sh
```

## Test Coverage

The validation suite tests:

1. **Read Compatibility**: LayKit reading gdstk-created files
2. **Write Compatibility**: gdstk reading LayKit-created files
3. **Format Conversion**: GDS ↔ OASIS round-trip integrity
4. **Property Handling**: GDSII property preservation
5. **Array References**: AREF handling and expansion
6. **Large Files**: Performance with 1000+ elements

## CI/CD Integration

These tests run automatically in GitHub Actions after the Rust test suite passes.

See `.github/workflows/ci.yml` for details.

## Troubleshooting

### gdstk not found

```bash
pip install gdstk
```

### LayKit binary not found

```bash
cargo build --release
```

The binary should be at: `./target/release/laykit`

### Tests failing

Check that:
- LayKit is built with `cargo build --release`
- gdstk is installed: `python3 -c "import gdstk"`
- You have write permissions in `/tmp` or the test directory

## Adding New Tests

To add a new validation test:

1. Add a function `test_your_feature()` in `gdstk_validation.py`
2. Add it to the `tests` list in `main()`
3. Follow the existing pattern:
   - Create test files with gdstk
   - Process with LayKit
   - Validate results with gdstk
   - Print "PASS" or "FAIL" with details

Example:

```python
def test_your_feature():
    """Test description."""
    print("Test N: Your feature... ", end="")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create test data
        # Process with LayKit
        # Validate results
        
        if success:
            print("PASS")
            return True
        else:
            print(f"FAIL\n  Reason: {error}")
            return False
```

