# Roadmap

## Current Release ✅

- ✅ GDSII read/write (all 7 element types)
- ✅ OASIS read/write (all element types)
- ✅ Bidirectional format conversion
- ✅ Geometry: bounding box, transforms, area, point-in-polygon, fillet, fracture
- ✅ Boolean operations: union, intersection, difference, XOR, slice, offset, convex hull
- ✅ FlexPath with miter/bevel/round joins and configurable end caps
- ✅ Curve primitives: arc, Bezier, ellipse, spline, regular polygon, star, spiral
- ✅ Cell topology: flatten, dependency order, cycle detection, library merge
- ✅ Streaming parser for large files
- ✅ AREF expansion
- ✅ Property management
- ✅ CLI tool (convert, info, validate)
- ✅ File format detection by magic bytes
- ✅ 178 tests, unified I/O (`load` / `load_library`), OASIS CBLOCK support

## Future

- [ ] **Python bindings** via PyO3 — drop-in replacement for gdstk
- [ ] **WebAssembly** — browser-based layout tools
- [ ] **SIMD acceleration** — vectorised coordinate operations
- [ ] **Parallel parsing** — multi-threaded structure processing via Rayon (optional feature flag)
- [ ] **Design Rule Checking** — basic geometric DRC (min width, spacing, overlap)
- [ ] **Layer mapping** — transform layer numbers during conversion
- [ ] **Partial file reading** — read a single named cell without parsing the whole file
- [ ] **GUI viewer** — simple pan/zoom layout visualisation (egui)
