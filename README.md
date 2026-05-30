# laykit

A production-ready Rust library for reading, writing, and manipulating GDSII and OASIS IC layout files.

[![crates.io](https://img.shields.io/crates/v/laykit.svg)](https://crates.io/crates/laykit)
[![book](https://img.shields.io/badge/guide-mdBook-orange)](https://giridharsalana.github.io/laykit/)
[![API on docs.rs](https://docs.rs/laykit/badge.svg)](https://docs.rs/laykit/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## Installation

```bash
cargo add laykit
```

Or add to `Cargo.toml` manually (use the crates.io badge for the latest version):

```toml
[dependencies]
laykit = "0"
```

## Quick Start

```rust
use laykit::load_library;

// Read GDSII or OASIS (format auto-detected); normalized Library API
let lib = load_library("layout.gds")?; // or layout.oas
println!("{} cells", lib.cell_count());
lib.save("copy.oas")?;
```

```rust
use laykit::GDSIIFile;

// Read GDSII explicitly
let gds = GDSIIFile::read_from_file("layout.gds")?;
println!("{} structures", gds.structures.len());

// Write
gds.write_to_file("output.gds")?;
```

```rust
use laykit::{GDSIIFile, converter};

// Convert GDS → OASIS
let gds = GDSIIFile::read_from_file("input.gds")?;
let oasis = converter::gdsii_to_oasis(&gds)?;
oasis.write_to_file("output.oas")?;
```

## Modules

| Module | Description |
|--------|-------------|
| `gdsii` | GDSII read/write (Boundary, Path, Text, SREF, AREF, Node, Box) |
| `oasis` | OASIS read/write (Rectangle, Polygon, Path, Trapezoid, Circle, Text, Placement) |
| `converter` | Bidirectional GDSII ↔ OASIS conversion |
| `geometry` | Bounding box, area, perimeter, transforms, point-in-polygon, fillet, fracture |
| `boolean_ops` | Union, intersection, difference, XOR, slice, offset, convex hull |
| `flexpath` | Flexible paths with miter/bevel/round joins and configurable end caps |
| `curve` | Arc, Bezier, ellipse, spline, regular polygon, star, spiral |
| `topology` | Cell flatten, dependency order, top-level cells, hierarchy validation, library merge |
| `streaming` | Streaming parser for large files |
| `aref_expansion` | Array reference expansion |
| `properties` | Property builders and managers |
| `layout` | Unified [`load`](https://docs.rs/laykit/latest/laykit/fn.load.html) for GDSII and OASIS |
| `format_detection` | File format detection by magic bytes |

## Documentation

- **User guide (mdBook)** — primary docs: [giridharsalana.github.io/laykit](https://giridharsalana.github.io/laykit)  
  Built from [`book/`](book/) on each release tag; includes getting started, GDSII/OASIS, conversion, and examples.
- **API reference (rustdoc on docs.rs)** — [docs.rs/laykit](https://docs.rs/laykit/)  
  Also mirrored under [giridharsalana.github.io/laykit/api](https://giridharsalana.github.io/laykit/api/) after each release.

## Testing

```bash
cargo test
```

Run `./tests/run_all_tests.sh` locally before tagging a release. Pushing a version tag (e.g. `v0.0.5`) runs the full test, docs, GitHub release, and crates.io publish pipeline.

## License

MIT

---

## Credits

The test suite for this library was developed with reference to [gdstk](https://github.com/heitzmann/gdstk) by Lucas Heitzmann Gabrielli, an excellent Python library for GDSII and OASIS IC layout. gdstk served as the behavioral reference for verifying correctness of geometry, boolean operations, path generation, and format I/O. It is licensed under the [Boost Software License 1.0](https://github.com/heitzmann/gdstk/blob/main/LICENSE).
