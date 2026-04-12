//! # laykit - GDSII and OASIS File Format Library
//!
//! A production-ready Rust library for reading, writing, and converting between
//! GDSII and OASIS file formats used in integrated circuit layout design.
//!
//! ## Features
//!
//! - **Full GDSII Support** - Read and write `.gds` files with all element types
//! - **Full OASIS Support** - Read and write `.oas` files with all element types
//! - **Format Conversion** - Convert between GDSII and OASIS formats
//! - **Streaming Parser** - Process large files without loading into memory
//! - **CLI Tool** - Command-line utility for file operations
//! - **Property Utilities** - Enhanced property management and builders
//! - **AREF Expansion** - Array reference expansion utilities
//! - **Zero Dependencies** - Pure Rust implementation using only `std`
//! - **Memory Safe** - Leverages Rust's ownership system
 //! - **Production Ready** - Comprehensive test suite with 164 tests
//!
//! ## Quick Start
//!
//! ### Reading a GDSII File
//!
//! ```no_run
//! use laykit::GDSIIFile;
//!
//! let gds = GDSIIFile::read_from_file("layout.gds")?;
//! println!("Library: {}", gds.library_name);
//! println!("Structures: {}", gds.structures.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Creating a GDSII File
//!
//! ```
//! use laykit::{GDSIIFile, GDSStructure, GDSTime, GDSElement, Boundary};
//!
//! let mut gds = GDSIIFile::new("MY_LIBRARY".to_string());
//! gds.units = (1e-6, 1e-9); // 1 micron user unit, 1nm database unit
//!
//! let mut structure = GDSStructure {
//!     name: "TOP".to_string(),
//!     creation_time: GDSTime::now(),
//!     modification_time: GDSTime::now(),
//!     strclass: None,
//!     elements: Vec::new(),
//! };
//!
//! structure.elements.push(GDSElement::Boundary(Boundary {
//!     layer: 1,
//!     datatype: 0,
//!     xy: vec![(0, 0), (1000, 0), (1000, 1000), (0, 1000), (0, 0)],
//!     elflags: None,
//!     plex: None,
//!     properties: Vec::new(),
//! }));
//!
//! gds.structures.push(structure);
//! ```
//!
//! ### Reading an OASIS File
//!
//! ```no_run
//! use laykit::OASISFile;
//!
//! let oasis = OASISFile::read_from_file("layout.oas")?;
//! println!("Cells: {}", oasis.cells.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Format Conversion
//!
//! ```no_run
//! use laykit::{GDSIIFile, converter};
//!
//! // Convert GDSII to OASIS
//! let gds = GDSIIFile::read_from_file("input.gds")?;
//! let oasis = converter::gdsii_to_oasis(&gds)?;
//! oasis.write_to_file("output.oas")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Modules
//!
//! - [`gdsii`] - GDSII format read/write (all 7 element types)
//! - [`oasis`] - OASIS format read/write (all element types)
//! - [`geometry`] - Geometric operations: bounding box, area, transforms, point-in-polygon
//! - [`boolean_ops`] - Boolean operations: union, intersection, difference, xor, slice, offset
//! - [`flexpath`] - Flexible paths with configurable joins and end caps
//! - [`curve`] - Curve primitives: arc, Bezier, ellipse, spline, regular polygon
//! - [`topology`] - Cell hierarchy: flatten, dependency order, top-level cells, library merge
//! - [`converter`] - Format conversion (GDSII ↔ OASIS)
//! - [`streaming`] - Streaming parser for large files (>1GB)
//! - [`properties`] - Property management utilities
//! - [`aref_expansion`] - Array reference expansion
//! - [`format_detection`] - File format detection by magic bytes

pub mod aref_expansion;
pub mod boolean_ops;
pub mod converter;
pub mod curve;
pub mod flexpath;
pub mod format_detection;
pub mod gdsii;
pub mod geometry;
pub mod oasis;
pub mod properties;
pub mod streaming;
pub mod topology;

pub use aref_expansion::*;
pub use boolean_ops::{boolean, offset, slice, BooleanOp, Axis, convex_hull};
pub use curve::{Curve, ellipse, regular_polygon, rounded_rectangle, star, spiral};
pub use flexpath::{EndCap, FlexPath, Join, RobustPath};
pub use gdsii::*;
pub use geometry::{
    BoundingBox,
    affine_transform, bounding_box, bounding_box_i32,
    close_polygon,
    distance, ensure_counter_clockwise, ensure_clockwise,
    fillet, fracture_to_rectangles,
    gds_element_bounding_box, library_bounding_box,
    inside, is_counter_clockwise, mirror_x, mirror_y,
    oasis_element_bounding_box,
    point_in_polygon, point_in_any_polygon,
    polygon_area, polygon_centroid, polygon_perimeter, polygon_signed_area,
    remove_duplicates, rotate, scale, structure_bounding_box, translate,
};
pub use oasis::*;
pub use properties::*;
pub use streaming::*;
pub use topology::{
    cell_dependencies, dependency_order, detect_cycles, direct_references,
    element_layer, filter_by_layer, flatten_structure,
    layers_in_library, layers_in_structure,
    merge_library, merge_library_overwrite,
    top_level_cells, total_element_count, validate_hierarchy,
};
