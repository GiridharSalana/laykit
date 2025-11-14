//! File format detection utilities
//!
//! This module provides utilities to detect GDSII and OASIS file formats
//! by reading magic bytes, independent of file extensions.

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Supported file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// GDSII Stream Format
    GDSII,
    /// OASIS (Open Artwork System Interchange Standard)
    OASIS,
    /// Unknown or unsupported format
    Unknown,
}

impl FileFormat {
    /// Returns the typical file extension for this format
    pub fn extension(&self) -> &str {
        match self {
            FileFormat::GDSII => "gds",
            FileFormat::OASIS => "oas",
            FileFormat::Unknown => "",
        }
    }

    /// Returns a human-readable name for this format
    pub fn name(&self) -> &str {
        match self {
            FileFormat::GDSII => "GDSII",
            FileFormat::OASIS => "OASIS",
            FileFormat::Unknown => "Unknown",
        }
    }
}

/// Detect file format from a file path by reading magic bytes
///
/// # Example
///
/// ```no_run
/// use laykit::format_detection::detect_format_from_file;
///
/// match detect_format_from_file("layout.dat") {
///     Ok(format) => println!("Detected format: {:?}", format),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn detect_format_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<FileFormat, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 16]; // Read first 16 bytes
    let bytes_read = file.read(&mut buffer)?;

    Ok(detect_format_from_bytes(&buffer[..bytes_read]))
}

/// Detect file format from raw bytes
///
/// This function examines the first few bytes of data to determine
/// if it matches the magic bytes of GDSII or OASIS formats.
///
/// # Example
///
/// ```
/// use laykit::format_detection::{detect_format_from_bytes, FileFormat};
///
/// // GDSII magic bytes (HEADER record)
/// let gdsii_bytes = [0x00, 0x06, 0x00, 0x02, 0x02, 0x58];
/// assert_eq!(detect_format_from_bytes(&gdsii_bytes), FileFormat::GDSII);
///
/// // OASIS magic bytes
/// let oasis_bytes = b"%SEMI-OASIS\r\n";
/// assert_eq!(detect_format_from_bytes(oasis_bytes), FileFormat::OASIS);
///
/// // Unknown format
/// let unknown_bytes = [0xFF, 0xFF, 0xFF, 0xFF];
/// assert_eq!(detect_format_from_bytes(&unknown_bytes), FileFormat::Unknown);
/// ```
pub fn detect_format_from_bytes(bytes: &[u8]) -> FileFormat {
    // Need at least 4 bytes to detect anything
    if bytes.len() < 4 {
        return FileFormat::Unknown;
    }

    // Check for OASIS magic: "%SEMI-OASIS\r\n" (13 bytes)
    if bytes.len() >= 13 && &bytes[0..13] == b"%SEMI-OASIS\r\n" {
        return FileFormat::OASIS;
    }

    // Check for GDSII magic: First record should be HEADER (0x00)
    // GDSII record format: [2 bytes length][1 byte record_type][1 byte data_type]
    // HEADER record: length=6, type=0x00, data_type=0x02 (INT2)
    let record_length = u16::from_be_bytes([bytes[0], bytes[1]]);
    let record_type = bytes[2];
    let data_type = bytes[3];

    if record_length == 6 && record_type == 0x00 && data_type == 0x02 {
        // Additional validation: check if the next 2 bytes are a reasonable version
        // GDSII version is typically 3, 5, or 600
        if bytes.len() >= 6 {
            let version = u16::from_be_bytes([bytes[4], bytes[5]]);
            // Sanity check: version should be reasonable (0 < version < 10000)
            if version > 0 && version < 10000 {
                return FileFormat::GDSII;
            }
        } else {
            // Not enough bytes to validate version, but structure looks right
            return FileFormat::GDSII;
        }
    }

    FileFormat::Unknown
}

/// Detect file format from a reader
///
/// Reads up to 16 bytes from the reader and attempts to detect the format.
/// The reader position will be advanced by the number of bytes read.
///
/// # Example
///
/// ```no_run
/// use laykit::format_detection::detect_format_from_reader;
/// use std::fs::File;
///
/// let mut file = File::open("layout.gds")?;
/// let format = detect_format_from_reader(&mut file)?;
/// println!("Detected: {:?}", format);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn detect_format_from_reader<R: Read>(
    reader: &mut R,
) -> Result<FileFormat, Box<dyn std::error::Error>> {
    let mut buffer = [0u8; 16];
    let bytes_read = reader.read(&mut buffer)?;
    Ok(detect_format_from_bytes(&buffer[..bytes_read]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_gdsii_format() {
        // Valid GDSII header with version 600
        let gdsii_bytes = [0x00, 0x06, 0x00, 0x02, 0x02, 0x58];
        assert_eq!(detect_format_from_bytes(&gdsii_bytes), FileFormat::GDSII);
    }

    #[test]
    fn test_detect_gdsii_format_version_3() {
        // Valid GDSII header with version 3
        let gdsii_bytes = [0x00, 0x06, 0x00, 0x02, 0x00, 0x03];
        assert_eq!(detect_format_from_bytes(&gdsii_bytes), FileFormat::GDSII);
    }

    #[test]
    fn test_detect_oasis_format() {
        let oasis_bytes = b"%SEMI-OASIS\r\n";
        assert_eq!(detect_format_from_bytes(oasis_bytes), FileFormat::OASIS);
    }

    #[test]
    fn test_detect_unknown_format() {
        let unknown_bytes = [0xFF, 0xFF, 0xFF, 0xFF];
        assert_eq!(
            detect_format_from_bytes(&unknown_bytes),
            FileFormat::Unknown
        );
    }

    #[test]
    fn test_detect_empty_file() {
        let empty_bytes = [];
        assert_eq!(detect_format_from_bytes(&empty_bytes), FileFormat::Unknown);
    }

    #[test]
    fn test_detect_too_short() {
        let short_bytes = [0x00, 0x06];
        assert_eq!(detect_format_from_bytes(&short_bytes), FileFormat::Unknown);
    }

    #[test]
    fn test_detect_invalid_gdsii_version() {
        // Header looks like GDSII but version is 0 (invalid)
        let invalid_bytes = [0x00, 0x06, 0x00, 0x02, 0x00, 0x00];
        assert_eq!(
            detect_format_from_bytes(&invalid_bytes),
            FileFormat::Unknown
        );
    }

    #[test]
    fn test_detect_gdsii_partial() {
        // Only 4 bytes - can't validate version but structure is right
        let partial_bytes = [0x00, 0x06, 0x00, 0x02];
        assert_eq!(detect_format_from_bytes(&partial_bytes), FileFormat::GDSII);
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(FileFormat::GDSII.extension(), "gds");
        assert_eq!(FileFormat::OASIS.extension(), "oas");
        assert_eq!(FileFormat::Unknown.extension(), "");
    }

    #[test]
    fn test_format_name() {
        assert_eq!(FileFormat::GDSII.name(), "GDSII");
        assert_eq!(FileFormat::OASIS.name(), "OASIS");
        assert_eq!(FileFormat::Unknown.name(), "Unknown");
    }
}

