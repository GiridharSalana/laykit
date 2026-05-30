//! Canonical layout library (gdstk-style), normalized to GDSII structures internally.

use crate::converter::{gdsii_to_oasis, oasis_to_gdsii_with_name};
use crate::error::LaykitError;
use crate::format_detection::FileFormat;
use crate::format_detection::detect_format_from_extension;
use crate::layout::{LayoutFile, LoadOptions, SaveOptions};
use crate::{GDSIIFile, GDSStructure, OASISFile};
use std::path::Path;

/// In-memory layout library with a unified cell list (GDSII structures).
///
/// OASIS files are normalized to GDSII on load so geometry, topology, and boolean
/// helpers can operate on one representation. Use [`Library::original_format`] to
/// see which disk format was loaded.
#[derive(Debug, Clone)]
pub struct Library {
    gds: GDSIIFile,
    original_format: FileFormat,
}

impl Library {
    /// Load any supported layout file into a canonical library.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, LaykitError> {
        Self::load_with_options(path, LoadOptions::default())
    }

    /// Load with [`LoadOptions`].
    pub fn load_with_options<P: AsRef<Path>>(
        path: P,
        options: LoadOptions,
    ) -> Result<Self, LaykitError> {
        let layout = LayoutFile::load_from_file_with_options(path, options)?;
        Self::from_layout(layout)
    }

    /// Build from an already-loaded [`crate::layout::LayoutFile`].
    pub fn from_layout(layout: LayoutFile) -> Result<Self, LaykitError> {
        match layout {
            LayoutFile::Gdsii(gds) => Ok(Self {
                gds,
                original_format: FileFormat::GDSII,
            }),
            LayoutFile::Oasis(oasis) => {
                let gds = oasis_to_gdsii_with_name(&oasis, None)
                    .map_err(|e| LaykitError::Parse(e.to_string()))?;
                Ok(Self {
                    gds,
                    original_format: FileFormat::OASIS,
                })
            }
        }
    }

    /// Format detected when this library was loaded from disk.
    pub fn original_format(&self) -> FileFormat {
        self.original_format
    }

    /// Library name (GDSII LIBNAME).
    pub fn name(&self) -> &str {
        &self.gds.library_name
    }

    /// User and database units in meters.
    pub fn units(&self) -> (f64, f64) {
        self.gds.units
    }

    /// Cells / structures in the library.
    pub fn cells(&self) -> &[GDSStructure] {
        &self.gds.structures
    }

    /// Number of cells.
    pub fn cell_count(&self) -> usize {
        self.gds.structures.len()
    }

    /// Borrow the underlying GDSII representation.
    pub fn as_gdsii(&self) -> &GDSIIFile {
        &self.gds
    }

    /// Consume into a [`GDSIIFile`].
    pub fn into_gdsii(self) -> GDSIIFile {
        self.gds
    }

    /// Convert to OASIS.
    pub fn to_oasis(&self) -> Result<OASISFile, LaykitError> {
        gdsii_to_oasis(&self.gds).map_err(|e| LaykitError::Parse(e.to_string()))
    }

    /// Save to a path; output format is chosen from the file extension.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), LaykitError> {
        self.save_with_options(path, SaveOptions::default())
    }

    /// Save with [`SaveOptions`].
    pub fn save_with_options<P: AsRef<Path>>(
        &self,
        path: P,
        options: SaveOptions,
    ) -> Result<(), LaykitError> {
        let path = path.as_ref();
        let format = options
            .format_hint
            .unwrap_or_else(|| detect_format_from_extension(path));
        match format {
            FileFormat::GDSII => self
                .gds
                .write_to_file(path)
                .map_err(|e| LaykitError::Parse(e.to_string())),
            FileFormat::OASIS => {
                let oasis = self.to_oasis()?;
                oasis
                    .write_to_file(path)
                    .map_err(|e| LaykitError::Parse(e.to_string()))
            }
            FileFormat::Unknown => Err(LaykitError::Parse(
                "cannot determine output format: use .gds or .oas extension".into(),
            )),
        }
    }
}

/// Load a layout file into a canonical [`Library`] (gdstk `read_gds` / `read_oas` equivalent).
pub fn load_library<P: AsRef<Path>>(path: P) -> Result<Library, LaykitError> {
    Library::load(path)
}

impl LayoutFile {
    /// Convert to a canonical [`crate::Library`].
    pub fn into_library(self) -> Result<Library, LaykitError> {
        Library::from_layout(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Boundary, GDSElement, GDSTime};

    #[test]
    fn library_from_gdsii() {
        let mut gds = GDSIIFile::new("LIB".to_string());
        gds.structures.push(GDSStructure {
            name: "TOP".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
            elements: vec![GDSElement::Boundary(Boundary {
                layer: 1,
                datatype: 0,
                xy: vec![(0, 0), (10, 0), (10, 10), (0, 0)],
                elflags: None,
                plex: None,
                properties: Vec::new(),
            })],
        });
        let path = std::env::temp_dir().join("laykit_lib_test.gds");
        gds.write_to_file(&path).unwrap();
        let lib = Library::load(&path).unwrap();
        assert_eq!(lib.cell_count(), 1);
        assert_eq!(lib.original_format(), FileFormat::GDSII);
        std::fs::remove_file(path).ok();
    }
}
