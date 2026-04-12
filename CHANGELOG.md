# Changelog

## [0.0.2] - 2026-04-12

### Changed

- Updated to Rust edition 2024
- Removed all version pins from documentation — use `cargo add laykit` or see crates.io badge
- README: removed gdstk framing, added crates.io version badge and documentation links
- Book: updated installation page (library is now on crates.io), introduction, project structure, roadmap, and testing pages
- `Cargo.toml`: added `documentation` and `homepage` fields pointing to GitHub Pages

### Fixed

- `src/gdsii.rs`: renamed local variable `gen` → `generations` (reserved keyword in edition 2024)
- `src/curve.rs`, `src/gdsii.rs`, `src/topology.rs`: collapsed nested `if let` patterns per clippy `collapsible_if` lint
- `LICENSE`: corrected copyright year and author name

## [0.0.1] - 2026-04-12

Initial release.

### Added

- GDSII read/write — all element types (Boundary, Path, Text, StructRef, ArrayRef, Node, Box)
- OASIS read/write — all element types (Rectangle, Polygon, Path, Trapezoid, CTrapezoid, Circle, Text, Placement)
- Bidirectional format conversion (GDSII ↔ OASIS)
- Geometry module — bounding box, polygon area/perimeter/centroid, point-in-polygon, affine transforms, mirror, fillet, fracture
- Boolean operations — union, intersection, difference, XOR, polygon slice, offset, convex hull
- FlexPath — flexible paths with miter/bevel/round joins and flush/half-width/extended/round end caps
- Curve primitives — arc, quadratic/cubic/smooth Bezier, elliptical arc, spline interpolation, regular polygon, ellipse, rounded rectangle, star, spiral
- Topology utilities — cell flatten, dependency order, top-level cell detection, cycle detection, hierarchy validation, library merge
- Streaming parser — process large GDSII files without loading into memory
- Array reference expansion (AREF → individual SREFs)
- Property builders and managers for GDSII and OASIS metadata
- File format detection by magic bytes
- CLI tool — `convert`, `info`, `validate` commands
- 164 tests, zero external dependencies
