# Project Structure

Overview of the LayKit codebase organization.

## Directory Structure

```
laykit/
├── .github/
│   └── workflows/
│       └── ci.yml              # GitHub Actions CI/CD
├── book/
│   ├── src/                    # mdBook documentation source
│   └── theme/                  # Custom CSS/JS for docs
├── examples/
│   ├── basic_usage.rs          # Basic usage example
│   ├── gdsii_only.rs           # GDSII-focused example
│   └── oasis_only.rs           # OASIS-focused example
├── src/
│   ├── lib.rs                  # Library entry point
│   ├── gdsii.rs                # GDSII implementation
│   ├── oasis.rs                # OASIS implementation
│   └── converter.rs            # Format conversion
├── tests/
│   └── tests.rs                # Integration tests
├── target/                     # Build artifacts (gitignored)
├── book.toml                   # mdBook configuration
├── Cargo.toml                  # Project manifest
├── Cargo.lock                  # Dependency lock file
├── CHANGELOG.md                # Version history
├── LICENSE                     # MIT License
├── README.md                   # Project README
└── .gitignore                  # Git ignore rules
```

## Source Files

### lib.rs (~90 lines)

Entry point and public API exports.

**Contents:**
- Module declarations
- Public re-exports
- Top-level documentation

**Example:**
```rust
//! LayKit - GDSII and OASIS library
pub mod gdsii;
pub mod oasis;
pub mod converter;

// Re-export main types
pub use gdsii::*;
pub use oasis::*;
```

### gdsii.rs (~1,000 lines)

Complete GDSII format implementation.

**Major components:**
- `GDSIIFile` - Main file structure
- `GDSStructure` - Cell/structure definition
- `GDSElement` - Element enum
- Element types: `Boundary`, `Path`, `Text`, etc.
- `STrans` - Transformation data
- `GDSTime` - Timestamp handling
- Binary I/O functions
- Real8 encoding/decoding

**Structure:**
```rust
// Data structures (~300 lines)
pub struct GDSIIFile { ... }
pub struct GDSStructure { ... }
pub enum GDSElement { ... }

// Implementation (~400 lines)
impl GDSIIFile {
    pub fn read_from_file(...) { ... }
    pub fn write_to_file(...) { ... }
    pub fn read<R: Read>(...) { ... }
    pub fn write<W: Write>(...) { ... }
}

// Helper functions (~300 lines)
fn read_record(...) { ... }
fn write_record(...) { ... }
fn decode_real8(...) { ... }
fn encode_real8(...) { ... }
```

### oasis.rs (~950 lines)

Complete OASIS format implementation.

**Major components:**
- `OASISFile` - Main file structure
- `OASISCell` - Cell definition
- `OASISElement` - Element enum
- Element types: `Rectangle`, `Polygon`, `Path`, etc.
- `NameTable` - String compression
- `Repetition` - Array patterns
- Variable-length integer encoding
- Zigzag encoding for signed integers

**Structure:**
```rust
// Data structures (~350 lines)
pub struct OASISFile { ... }
pub struct OASISCell { ... }
pub enum OASISElement { ... }

// Implementation (~350 lines)
impl OASISFile {
    pub fn read_from_file(...) { ... }
    pub fn write_to_file(...) { ... }
}

// Helper functions (~250 lines)
fn read_unsigned_integer(...) { ... }
fn read_signed_integer(...) { ... }
fn write_unsigned_integer(...) { ... }
fn zigzag_encode(...) { ... }
fn zigzag_decode(...) { ... }
```

### converter.rs (~300 lines)

Format conversion utilities.

**Functions:**
- `gdsii_to_oasis()` - GDSII to OASIS conversion
- `oasis_to_gdsii()` - OASIS to GDSII conversion
- `is_rectangle()` - Rectangle detection helper

**Structure:**
```rust
// Main conversion functions (~250 lines)
pub fn gdsii_to_oasis(gds: &GDSIIFile) 
    -> Result<OASISFile, Box<dyn Error>> { ... }

pub fn oasis_to_gdsii(oasis: &OASISFile) 
    -> Result<GDSIIFile, Box<dyn Error>> { ... }

// Helper functions (~50 lines)
pub fn is_rectangle(points: &[(i32, i32)]) 
    -> Option<(i32, i32, i32, i32)> { ... }
```

## Examples

### basic_usage.rs (~150 lines)

Demonstrates core functionality:
- Creating GDSII files
- Creating OASIS files
- Format conversion
- Reading and displaying information

### gdsii_only.rs (~200 lines)

Comprehensive GDSII example:
- All element types
- Hierarchical design
- Transformations
- Properties
- Arrays

### oasis_only.rs (~150 lines)

OASIS-specific features:
- Rectangle primitives
- Trapezoids
- Circles
- Name tables
- Repetitions

## Tests

### tests.rs (~600 lines)

Comprehensive test suite:

**Test categories:**
```rust
// GDSII tests (7 tests, ~200 lines)
#[test] fn test_gdsii_create_and_write() { ... }
#[test] fn test_gdsii_round_trip() { ... }
// ...

// OASIS tests (11 tests, ~300 lines)
#[test] fn test_oasis_create_simple() { ... }
#[test] fn test_oasis_round_trip_rectangles() { ... }
// ...

// Converter tests (3 tests, ~100 lines)
#[test] fn test_gdsii_to_oasis_conversion() { ... }
#[test] fn test_oasis_to_gdsii_conversion() { ... }
#[test] fn test_rectangle_detection() { ... }
```

## Documentation

### API Documentation

Generated from source code comments:
- Module-level documentation
- Struct/enum documentation
- Function documentation
- Example code

**Location:** `target/doc/laykit/`

### User Guide (mdBook)

Located in `book/src/`:
- Introduction and getting started
- Format-specific guides
- Conversion guide
- Examples and tutorials
- API reference
- Technical details

**Build output:** `book/build/`

## Build Artifacts

### target/ Directory

```
target/
├── debug/              # Debug builds
│   ├── deps/          # Dependencies
│   ├── examples/      # Example binaries
│   └── incremental/   # Incremental compilation data
├── release/           # Release builds (optimized)
│   ├── deps/
│   └── examples/
└── doc/              # Generated API documentation
    └── laykit/
```

## Configuration Files

### Cargo.toml

Project manifest:

```toml
[package]
name = "laykit"
version = "0.2.4"
edition = "2021"
authors = ["Giridhar Salana <giridharsalana@gmail.com>"]
description = "Production-ready Rust library for GDSII and OASIS"
repository = "https://github.com/giridharsalana/laykit"
license = "MIT"
keywords = ["gdsii", "oasis", "ic-layout", "vlsi", "eda"]
categories = ["parser-implementations", "encoding"]

[dependencies]
# Zero dependencies!

[dev-dependencies]
# Testing dependencies (if any)

[[example]]
name = "basic_usage"
path = "examples/basic_usage.rs"
```

### book.toml

mdBook configuration:

```toml
[book]
title = "LayKit Documentation"
authors = ["Giridhar Salana"]
description = "Production-ready Rust library for GDSII and OASIS"
language = "en"
src = "book/src"

[build]
build-dir = "book/build"

[output.html]
default-theme = "rust"
git-repository-url = "https://github.com/giridharsalana/laykit"
```

### .gitignore

Ignored files:

```
/target/              # Build artifacts
**/*.rs.bk           # Rust backups
*.gds                # Test GDSII files
*.oas                # Test OASIS files
/book/build/         # mdBook output
.vscode/             # Editor config
.DS_Store            # macOS files
```

## Module Organization

### Public API

What users interact with:

```rust
laykit
├── GDSIIFile          // Main GDSII type
├── GDSStructure       // GDSII structures
├── GDSElement         // GDSII elements
├── OASISFile          // Main OASIS type
├── OASISCell          // OASIS cells
├── OASISElement       // OASIS elements
└── converter          // Conversion functions
    ├── gdsii_to_oasis
    └── oasis_to_gdsii
```

### Internal Organization

How code is structured:

```
src/
├── lib.rs                    # Public API surface
├── gdsii.rs                  # Self-contained module
│   ├── Data structures
│   ├── I/O implementation
│   └── Helper functions
├── oasis.rs                  # Self-contained module
│   ├── Data structures
│   ├── I/O implementation
│   └── Helper functions
└── converter.rs              # Uses both modules
    └── Conversion logic
```

## Statistics

### Code Metrics

| Metric | Value |
|--------|-------|
| Total source lines | ~2,950 |
| GDSII module | ~1,000 |
| OASIS module | ~950 |
| Converter module | ~300 |
| Tests | ~600 |
| Examples | ~500 |
| Documentation | ~5,000+ |

### Dependencies

- **Runtime:** 0 (zero!)
- **Development:** Standard Rust toolchain only
- **Optional:** mdBook for documentation

## Build Process

### Development Build

```bash
cargo build
# Output: target/debug/
```

### Release Build

```bash
cargo build --release
# Output: target/release/
```

### Documentation Build

```bash
# API docs
cargo doc

# User guide
mdbook build
```

### Running Examples

```bash
cargo run --example basic_usage
cargo run --release --example gdsii_only
```

## CI/CD Pipeline

GitHub Actions workflow:

1. **Test** - Run all tests
2. **Clippy** - Lint checking
3. **Format** - Style checking
4. **Build** - Release builds (on tags)
5. **Docs** - Generate and deploy documentation
6. **Release** - Create GitHub release

## Adding New Features

### Checklist

When adding a new feature:

1. ✅ Add implementation in appropriate module
2. ✅ Add public API in `lib.rs`
3. ✅ Add tests in `tests/tests.rs`
4. ✅ Add example in `examples/`
5. ✅ Update documentation
6. ✅ Update CHANGELOG.md
7. ✅ Run all checks (`cargo test`, `cargo clippy`, `cargo fmt`)

### Example: Adding a New Element Type

1. **Define structure** (e.g., in `gdsii.rs`)
2. **Add to enum** (`GDSElement`)
3. **Implement I/O** (read/write functions)
4. **Add conversion** (in `converter.rs`)
5. **Write tests**
6. **Document**

## Navigation Tips

### Finding Code

- **GDSII features** → `src/gdsii.rs`
- **OASIS features** → `src/oasis.rs`
- **Conversions** → `src/converter.rs`
- **Tests** → `tests/tests.rs`
- **Examples** → `examples/`

### Understanding Flow

1. Start with `examples/` to see usage
2. Check `src/lib.rs` for public API
3. Dive into implementation files
4. Refer to tests for edge cases
