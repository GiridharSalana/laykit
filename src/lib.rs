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
//! - **Production Ready** - Comprehensive test suite with 53 tests
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
//! - [`gdsii`] - GDSII format support
//! - [`oasis`] - OASIS format support
//! - [`converter`] - Format conversion utilities
//! - [`streaming`] - Streaming parser for large files
//! - [`properties`] - Property management utilities
//! - [`aref_expansion`] - Array reference expansion tools
//! - [`format_detection`] - File format detection by magic bytes

pub mod aref_expansion;
pub mod converter;
pub mod format_detection;
pub mod gdsii;
pub mod oasis;
pub mod properties;
pub mod streaming;

pub use aref_expansion::*;
pub use gdsii::*;
pub use oasis::*;
pub use properties::*;
pub use streaming::*;
