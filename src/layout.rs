//! Unified layout loading and saving for GDSII and OASIS files.

use crate::converter::{gdsii_to_oasis, oasis_to_gdsii_with_name};
use crate::error::LaykitError;
use crate::format_detection::{
    FileFormat, detect_format_from_bytes, detect_format_from_extension, detect_format_from_file,
};
use crate::{GDSIIFile, OASISFile};
use std::io::{Cursor, Read};
use std::path::Path;

pub use crate::error::LoadError;

/// Options for [`load`] and [`LayoutFile::load_from_file`].
#[derive(Debug, Clone, Copy, Default)]
pub struct LoadOptions {
    /// When magic-byte detection fails, infer format from `.gds` / `.oas` extension.
    pub extension_fallback: bool,
}

/// Options for [`save_layout`] and [`crate::Library::save`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SaveOptions {
    /// When the output extension is ambiguous, use this format instead of returning an error.
    pub format_hint: Option<FileFormat>,
}

/// A layout file in either native GDSII or OASIS representation.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum LayoutFile {
    Gdsii(GDSIIFile),
    Oasis(OASISFile),
}

impl LayoutFile {
    pub fn format(&self) -> FileFormat {
        match self {
            LayoutFile::Gdsii(_) => FileFormat::GDSII,
            LayoutFile::Oasis(_) => FileFormat::OASIS,
        }
    }

    pub fn cell_count(&self) -> usize {
        match self {
            LayoutFile::Gdsii(gds) => gds.structures.len(),
            LayoutFile::Oasis(oas) => oas.cells.len(),
        }
    }

    pub fn as_gdsii(&self) -> Option<&GDSIIFile> {
        match self {
            LayoutFile::Gdsii(gds) => Some(gds),
            LayoutFile::Oasis(_) => None,
        }
    }

    pub fn as_oasis(&self) -> Option<&OASISFile> {
        match self {
            LayoutFile::Gdsii(_) => None,
            LayoutFile::Oasis(oas) => Some(oas),
        }
    }

    pub fn into_gdsii(self) -> Option<GDSIIFile> {
        match self {
            LayoutFile::Gdsii(gds) => Some(gds),
            LayoutFile::Oasis(_) => None,
        }
    }

    pub fn into_oasis(self) -> Option<OASISFile> {
        match self {
            LayoutFile::Gdsii(_) => None,
            LayoutFile::Oasis(oas) => Some(oas),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, LaykitError> {
        Self::load_from_file_with_options(path, LoadOptions::default())
    }

    pub fn load_from_file_with_options<P: AsRef<Path>>(
        path: P,
        options: LoadOptions,
    ) -> Result<Self, LaykitError> {
        let path = path.as_ref();
        let mut format =
            detect_format_from_file(path).map_err(|e| LaykitError::Io(e.to_string()))?;
        if format == FileFormat::Unknown && options.extension_fallback {
            format = detect_format_from_extension(path);
        }

        match format {
            FileFormat::GDSII => GDSIIFile::read_from_file(path)
                .map(LayoutFile::Gdsii)
                .map_err(|e| LaykitError::Parse(e.to_string())),
            FileFormat::OASIS => OASISFile::read_from_file(path)
                .map(LayoutFile::Oasis)
                .map_err(|e| LaykitError::Parse(e.to_string())),
            FileFormat::Unknown => Err(LaykitError::UnknownFormat),
        }
    }

    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, LaykitError> {
        match detect_format_from_bytes(bytes) {
            FileFormat::GDSII => {
                let mut reader = Cursor::new(bytes);
                GDSIIFile::read_from_reader(&mut reader)
                    .map(LayoutFile::Gdsii)
                    .map_err(|e| LaykitError::Parse(e.to_string()))
            }
            FileFormat::OASIS => {
                let mut reader = Cursor::new(bytes);
                OASISFile::read_from_reader(&mut reader)
                    .map(LayoutFile::Oasis)
                    .map_err(|e| LaykitError::Parse(e.to_string()))
            }
            FileFormat::Unknown => Err(LaykitError::UnknownFormat),
        }
    }

    pub fn load_from_reader<R: Read>(reader: &mut R) -> Result<Self, LaykitError> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Self::load_from_bytes(&data)
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LaykitError> {
        match self {
            LayoutFile::Gdsii(gds) => gds
                .write_to_file(path)
                .map_err(|e| LaykitError::Parse(e.to_string())),
            LayoutFile::Oasis(oas) => oas
                .write_to_file(path)
                .map_err(|e| LaykitError::Parse(e.to_string())),
        }
    }
}

/// Load GDSII or OASIS, auto-detecting format from file content.
pub fn load<P: AsRef<Path>>(path: P) -> Result<LayoutFile, LaykitError> {
    LayoutFile::load_from_file(path)
}

/// Load with explicit [`LoadOptions`].
pub fn load_with_options<P: AsRef<Path>>(
    path: P,
    options: LoadOptions,
) -> Result<LayoutFile, LaykitError> {
    LayoutFile::load_from_file_with_options(path, options)
}

/// Save a [`LayoutFile`] to a path, converting format if the extension requires it.
pub fn save_layout<P: AsRef<Path>>(
    path: P,
    layout: &LayoutFile,
    options: SaveOptions,
) -> Result<(), LaykitError> {
    let path = path.as_ref();
    let format = resolve_output_format(path, options)?;
    match (layout, format) {
        (LayoutFile::Gdsii(gds), FileFormat::GDSII) => gds
            .write_to_file(path)
            .map_err(|e| LaykitError::Parse(e.to_string())),
        (LayoutFile::Oasis(oas), FileFormat::OASIS) => oas
            .write_to_file(path)
            .map_err(|e| LaykitError::Parse(e.to_string())),
        (LayoutFile::Gdsii(gds), FileFormat::OASIS) => {
            let oasis = gdsii_to_oasis(gds).map_err(|e| LaykitError::Parse(e.to_string()))?;
            oasis
                .write_to_file(path)
                .map_err(|e| LaykitError::Parse(e.to_string()))
        }
        (LayoutFile::Oasis(oas), FileFormat::GDSII) => {
            let gds = oasis_to_gdsii_with_name(oas, path.to_str())
                .map_err(|e| LaykitError::Parse(e.to_string()))?;
            gds.write_to_file(path)
                .map_err(|e| LaykitError::Parse(e.to_string()))
        }
        (_, FileFormat::Unknown) => Err(LaykitError::Parse(
            "cannot determine output format: use .gds or .oas extension".into(),
        )),
    }
}

fn resolve_output_format(path: &Path, options: SaveOptions) -> Result<FileFormat, LaykitError> {
    if let Some(hint) = options.format_hint {
        return Ok(hint);
    }
    Ok(detect_format_from_extension(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Boundary, GDSElement, GDSStructure, GDSTime, OASISCell, OASISElement, Rectangle};

    #[test]
    fn load_gdsii_round_trip() {
        let mut gds = GDSIIFile::new("T".to_string());
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
        let path = std::env::temp_dir().join("laykit_save_gds.gds");
        gds.write_to_file(&path).unwrap();
        let layout = load(&path).unwrap();
        assert_eq!(layout.format(), FileFormat::GDSII);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn save_layout_converts_to_oasis() {
        let gds = GDSIIFile::new("T".to_string());
        let gds_path = std::env::temp_dir().join("laykit_save_layout.gds");
        gds.write_to_file(&gds_path).unwrap();
        let layout = load(&gds_path).unwrap();
        let oas_path = std::env::temp_dir().join("laykit_save_layout_out.oas");
        save_layout(&oas_path, &layout, SaveOptions::default()).unwrap();
        assert_eq!(load(&oas_path).unwrap().format(), FileFormat::OASIS);
        std::fs::remove_file(gds_path).ok();
        std::fs::remove_file(oas_path).ok();
    }

    #[test]
    fn extension_fallback_option() {
        let gds = GDSIIFile::new("E".to_string());
        let path = std::env::temp_dir().join("laykit_ext_fallback.gds");
        gds.write_to_file(&path).unwrap();
        let opts = LoadOptions {
            extension_fallback: true,
        };
        assert!(LayoutFile::load_from_file_with_options(&path, opts).is_ok());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn load_oasis_file() {
        let mut oasis = OASISFile::new();
        oasis.cells.push(OASISCell {
            name: "C".to_string(),
            name_ref: None,
            elements: vec![OASISElement::Rectangle(Rectangle {
                layer: 1,
                datatype: 0,
                x: 0,
                y: 0,
                width: 10,
                height: 10,
                repetition: None,
                properties: Vec::new(),
            })],
        });
        let path = std::env::temp_dir().join("laykit_load_oas2.oas");
        oasis.write_to_file(&path).unwrap();
        assert_eq!(load(&path).unwrap().cell_count(), 1);
        std::fs::remove_file(path).ok();
    }
}
