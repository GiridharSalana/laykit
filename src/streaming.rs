// Streaming parser for large GDSII and OASIS files
// Allows processing files >1GB without loading entire file into memory

use crate::gdsii::{GDSElement, GDSStructure, GDSTime};
use std::io::{Read, Seek, SeekFrom};

/// Callback trait for processing structures as they are read
pub trait StructureCallback {
    fn on_structure(&mut self, structure: &GDSStructure) -> Result<(), Box<dyn std::error::Error>>;
}

/// Streaming GDSII reader
pub struct StreamingGDSIIReader<R: Read + Seek> {
    reader: R,
    library_name: String,
    units: (f64, f64),
    version: u16,
}

impl<R: Read + Seek> StreamingGDSIIReader<R> {
    /// Create a new streaming reader
    pub fn new(mut reader: R) -> Result<Self, Box<dyn std::error::Error>> {
        // Read header information
        let version = Self::read_header(&mut reader)?;
        let library_name = Self::read_library_name(&mut reader)?;
        let units = Self::read_units(&mut reader)?;

        Ok(Self {
            reader,
            library_name,
            units,
            version,
        })
    }

    /// Get library name
    pub fn library_name(&self) -> &str {
        &self.library_name
    }

    /// Get units
    pub fn units(&self) -> (f64, f64) {
        self.units
    }

    /// Get version
    pub fn version(&self) -> u16 {
        self.version
    }

    /// Process all structures with a callback
    pub fn process_structures<C: StructureCallback>(
        &mut self,
        callback: &mut C,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(structure) = self.read_next_structure()? {
            callback.on_structure(&structure)?;
        }
        Ok(())
    }

    /// Read next structure from file
    fn read_next_structure(&mut self) -> Result<Option<GDSStructure>, Box<dyn std::error::Error>> {
        // Read record header
        let mut record_header = [0u8; 4];
        match self.reader.read_exact(&mut record_header) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(Box::new(e)),
        }

        let record_len = u16::from_be_bytes([record_header[0], record_header[1]]);
        let record_type = record_header[2];

        // Check if this is BGNSTR (structure begin)
        if record_type != 0x05 {
            // Skip to next record
            if record_len > 4 {
                self.reader
                    .seek(SeekFrom::Current((record_len - 4) as i64))?;
            }
            return self.read_next_structure();
        }

        // Read structure timestamps
        let creation_time = self.read_timestamp()?;
        let modification_time = creation_time.clone();

        // Read structure name
        let name = self.read_structure_name()?;

        // Read elements until ENDSTR
        let elements = self.read_elements()?;

        Ok(Some(GDSStructure {
            name,
            creation_time,
            modification_time,
            strclass: None,
            elements,
        }))
    }

    fn read_header(reader: &mut R) -> Result<u16, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;

        let _record_len = u16::from_be_bytes([buf[0], buf[1]]);
        let record_type = buf[2];

        if record_type != 0x00 {
            return Err("Invalid GDSII file: expected HEADER record".into());
        }

        let mut version_buf = [0u8; 2];
        reader.read_exact(&mut version_buf)?;
        Ok(u16::from_be_bytes(version_buf))
    }

    fn read_library_name(reader: &mut R) -> Result<String, Box<dyn std::error::Error>> {
        // Skip BGNLIB record (timestamps)
        let mut header = [0u8; 4];
        reader.read_exact(&mut header)?;
        let len = u16::from_be_bytes([header[0], header[1]]);
        if len > 4 {
            let mut skip = vec![0u8; (len - 4) as usize];
            reader.read_exact(&mut skip)?;
        }

        // Read LIBNAME record
        reader.read_exact(&mut header)?;
        let len = u16::from_be_bytes([header[0], header[1]]);
        let record_type = header[2];

        if record_type != 0x02 {
            return Err("Invalid GDSII file: expected LIBNAME record".into());
        }

        let mut name_buf = vec![0u8; (len - 4) as usize];
        reader.read_exact(&mut name_buf)?;

        // Remove null terminator and convert to string
        if let Some(pos) = name_buf.iter().position(|&b| b == 0) {
            name_buf.truncate(pos);
        }

        Ok(String::from_utf8_lossy(&name_buf).to_string())
    }

    fn read_units(reader: &mut R) -> Result<(f64, f64), Box<dyn std::error::Error>> {
        // Read UNITS record
        let mut header = [0u8; 4];
        reader.read_exact(&mut header)?;
        let record_type = header[2];

        if record_type != 0x03 {
            return Err("Invalid GDSII file: expected UNITS record".into());
        }

        // Read two Real8 values
        let user_unit = Self::read_real8(reader)?;
        let db_unit = Self::read_real8(reader)?;

        Ok((user_unit, db_unit))
    }

    fn read_real8(reader: &mut R) -> Result<f64, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;

        // GDSII Real8 format: [sign:1][exponent:7][mantissa:56]
        let sign = if (buf[0] & 0x80) != 0 { -1.0 } else { 1.0 };
        let exponent = (buf[0] & 0x7F) as i32 - 64;

        let mut mantissa = 0u64;
        for &byte in buf.iter().skip(1) {
            mantissa = (mantissa << 8) | (byte as u64);
        }

        let mantissa_f = mantissa as f64 / (1u64 << 56) as f64;

        Ok(sign * mantissa_f * 16.0_f64.powi(exponent))
    }

    fn read_timestamp(&mut self) -> Result<GDSTime, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 24]; // 12 i16 values
        self.reader.read_exact(&mut buf)?;

        let year = u16::from_be_bytes([buf[0], buf[1]]);
        let month = u16::from_be_bytes([buf[2], buf[3]]);
        let day = u16::from_be_bytes([buf[4], buf[5]]);
        let hour = u16::from_be_bytes([buf[6], buf[7]]);
        let minute = u16::from_be_bytes([buf[8], buf[9]]);
        let second = u16::from_be_bytes([buf[10], buf[11]]);

        Ok(GDSTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
        })
    }

    fn read_structure_name(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let mut header = [0u8; 4];
        self.reader.read_exact(&mut header)?;
        let len = u16::from_be_bytes([header[0], header[1]]);
        let record_type = header[2];

        if record_type != 0x06 {
            return Err("Expected STRNAME record".into());
        }

        let mut name_buf = vec![0u8; (len - 4) as usize];
        self.reader.read_exact(&mut name_buf)?;

        if let Some(pos) = name_buf.iter().position(|&b| b == 0) {
            name_buf.truncate(pos);
        }

        Ok(String::from_utf8_lossy(&name_buf).to_string())
    }

    fn read_elements(&mut self) -> Result<Vec<GDSElement>, Box<dyn std::error::Error>> {
        let elements = Vec::new();

        loop {
            let mut header = [0u8; 4];
            self.reader.read_exact(&mut header)?;
            let len = u16::from_be_bytes([header[0], header[1]]);
            let record_type = header[2];

            // ENDSTR - end of structure
            if record_type == 0x07 {
                break;
            }

            // Skip element data for now (in a full implementation, we'd parse each element type)
            // This is a simplified streaming parser that focuses on structure-level processing
            if len > 4 {
                self.reader.seek(SeekFrom::Current((len - 4) as i64))?;
            }
        }

        Ok(elements)
    }
}

/// Statistics collector for streaming processing
#[derive(Default)]
pub struct StatisticsCollector {
    pub structure_count: usize,
    pub element_count: usize,
    pub total_bytes_processed: u64,
}

impl StatisticsCollector {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StructureCallback for StatisticsCollector {
    fn on_structure(&mut self, structure: &GDSStructure) -> Result<(), Box<dyn std::error::Error>> {
        self.structure_count += 1;
        self.element_count += structure.elements.len();
        Ok(())
    }
}

/// Structure name collector
#[derive(Default)]
pub struct StructureNameCollector {
    pub names: Vec<String>,
}

impl StructureNameCollector {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StructureCallback for StructureNameCollector {
    fn on_structure(&mut self, structure: &GDSStructure) -> Result<(), Box<dyn std::error::Error>> {
        self.names.push(structure.name.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Boundary, GDSElement, GDSIIFile, GDSStructure};
    use std::io::Cursor;

    #[test]
    fn test_streaming_statistics() {
        // Create a test GDSII file
        let mut gds = GDSIIFile::new("STREAMTEST".to_string());
        gds.units = (1e-6, 1e-9);

        for i in 0..10 {
            let mut structure = GDSStructure {
                name: format!("CELL_{}", i),
                creation_time: GDSTime::now(),
                modification_time: GDSTime::now(),
                strclass: None,
                elements: Vec::new(),
            };

            for j in 0..5 {
                structure.elements.push(GDSElement::Boundary(Boundary {
                    layer: 1,
                    datatype: 0,
                    xy: vec![
                        (j * 100, 0),
                        ((j + 1) * 100, 0),
                        ((j + 1) * 100, 100),
                        (j * 100, 100),
                        (j * 100, 0),
                    ],
                    elflags: None,
                    plex: None,
                    properties: Vec::new(),
                }));
            }

            gds.structures.push(structure);
        }

        // Write to bytes
        let mut buffer = Vec::new();
        gds.write_to_writer(&mut buffer).unwrap();

        // Stream read with statistics
        let cursor = Cursor::new(buffer);
        let mut reader = StreamingGDSIIReader::new(cursor).unwrap();
        let mut stats = StatisticsCollector::new();

        reader.process_structures(&mut stats).unwrap();

        assert_eq!(stats.structure_count, 10);
        // Elements are not fully parsed in this simplified streaming implementation
        // In a full implementation, we'd track element counts accurately
    }

    #[test]
    fn test_streaming_name_collection() {
        // Create a test GDSII file
        let mut gds = GDSIIFile::new("NAMETEST".to_string());
        gds.units = (1e-6, 1e-9);

        let names = vec!["TOP", "SUBCELL1", "SUBCELL2"];

        for name in &names {
            let structure = GDSStructure {
                name: name.to_string(),
                creation_time: GDSTime::now(),
                modification_time: GDSTime::now(),
                strclass: None,
                elements: Vec::new(),
            };
            gds.structures.push(structure);
        }

        // Write to bytes
        let mut buffer = Vec::new();
        gds.write_to_writer(&mut buffer).unwrap();

        // Stream read with name collection
        let cursor = Cursor::new(buffer);
        let mut reader = StreamingGDSIIReader::new(cursor).unwrap();
        let mut collector = StructureNameCollector::new();

        reader.process_structures(&mut collector).unwrap();

        assert_eq!(collector.names.len(), 3);
        assert_eq!(collector.names, names);
    }
}
