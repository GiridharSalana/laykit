# Changelog

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
