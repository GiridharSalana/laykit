# Introduction

**laykit** is a Rust implementation of [gdstk](https://github.com/heitzmann/gdstk) — a library for reading, writing, and manipulating GDSII and OASIS IC layout files.

## What it does

- Read and write **GDSII** (`.gds`) files — the industry-standard binary format for IC layouts
- Read and write **OASIS** (`.oas`) files — the modern, more compact replacement
- Convert bidirectionally between the two formats
- Perform geometric operations: bounding boxes, transforms, polygon area/perimeter, point-in-polygon, fillet, fracture
- Boolean polygon operations: union, intersection, difference, XOR, slice, offset
- Generate complex shapes: FlexPath with configurable joins/caps, arcs, Bezier curves, ellipses, splines, spirals
- Manage cell hierarchies: flatten, dependency ordering, cycle detection, library merge
- Stream large files without loading them fully into memory

## Design

- **Zero external dependencies** — pure Rust `std` only
- **164 tests**, 0 failures
- Idiomatic Rust: `Result`-based error handling, enums for element types, no unsafe code in public API

## Project Status

**v0.0.1** — initial release covering full gdstk feature parity:

- ✅ GDSII read/write (all 7 element types)
- ✅ OASIS read/write (all element types)
- ✅ Bidirectional format conversion
- ✅ Geometry module
- ✅ Boolean operations
- ✅ FlexPath and curve primitives
- ✅ Cell topology utilities
- ✅ Streaming parser
- ✅ AREF expansion
- ✅ Property management
- ✅ CLI tool (convert, info, validate)
- ✅ File format detection by magic bytes
