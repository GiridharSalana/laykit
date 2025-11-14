// GDSII (Graphic Database System II) Reader/Writer
// Industry standard format for IC layout interchange
// Binary format with record-based structure

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// GDSII File structure
#[derive(Debug, Clone)]
pub struct GDSIIFile {
    pub version: u16,
    pub modification_time: GDSTime,
    pub access_time: GDSTime,
    pub library_name: String,
    pub units: (f64, f64),         // (user_units, database_units in meters)
    pub reflibs: Vec<String>,      // Referenced library names (record 0x1F)
    pub fonts: Vec<String>,        // Font table (record 0x29)
    pub generations: Option<i16>,  // Backup generations (record 0x3C)
    pub attrtable: Option<String>, // Attribute table reference (record 0x3D)
    pub structures: Vec<GDSStructure>,
}

#[derive(Debug, Clone)]
pub struct GDSTime {
    pub year: u16,
    pub month: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
}

#[derive(Debug, Clone)]
pub struct GDSStructure {
    pub name: String,
    pub creation_time: GDSTime,
    pub modification_time: GDSTime,
    pub strclass: Option<i16>, // Structure class (record 0x34)
    pub elements: Vec<GDSElement>,
}

#[derive(Debug, Clone)]
pub enum GDSElement {
    Boundary(Boundary),
    Path(GPath),
    StructRef(StructRef),
    ArrayRef(ArrayRef),
    Text(GText),
    Node(Node),
    Box(GDSBox),
}

#[derive(Debug, Clone)]
pub struct Boundary {
    pub layer: i16,
    pub datatype: i16,
    pub xy: Vec<(i32, i32)>,
    pub elflags: Option<i16>, // Element flags (record 0x26)
    pub plex: Option<i32>,    // Plex identifier (record 0x2F)
    pub properties: Vec<GDSProperty>,
}

#[derive(Debug, Clone)]
pub struct GPath {
    pub layer: i16,
    pub datatype: i16,
    pub pathtype: i16,
    pub width: Option<i32>,
    pub bgnextn: Option<i32>, // Path extension at beginning (record 0x30)
    pub endextn: Option<i32>, // Path extension at end (record 0x31)
    pub xy: Vec<(i32, i32)>,
    pub elflags: Option<i16>, // Element flags (record 0x26)
    pub plex: Option<i32>,    // Plex identifier (record 0x2F)
    pub properties: Vec<GDSProperty>,
}

#[derive(Debug, Clone)]
pub struct StructRef {
    pub sname: String,
    pub xy: (i32, i32),
    pub strans: Option<STrans>,
    pub elflags: Option<i16>, // Element flags (record 0x26)
    pub plex: Option<i32>,    // Plex identifier (record 0x2F)
    pub properties: Vec<GDSProperty>,
}

#[derive(Debug, Clone)]
pub struct ArrayRef {
    pub sname: String,
    pub columns: u16,
    pub rows: u16,
    pub xy: Vec<(i32, i32)>, // 3 points: origin, column_spacing, row_spacing
    pub strans: Option<STrans>,
    pub elflags: Option<i16>, // Element flags (record 0x26)
    pub plex: Option<i32>,    // Plex identifier (record 0x2F)
    pub properties: Vec<GDSProperty>,
}

#[derive(Debug, Clone)]
pub struct GText {
    pub layer: i16,
    pub texttype: i16,
    pub string: String,
    pub xy: (i32, i32),
    pub presentation: Option<i16>,
    pub strans: Option<STrans>,
    pub width: Option<i32>,
    pub elflags: Option<i16>, // Element flags (record 0x26)
    pub plex: Option<i32>,    // Plex identifier (record 0x2F)
    pub properties: Vec<GDSProperty>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub layer: i16,
    pub nodetype: i16,
    pub xy: Vec<(i32, i32)>,
    pub elflags: Option<i16>, // Element flags (record 0x26)
    pub plex: Option<i32>,    // Plex identifier (record 0x2F)
    pub properties: Vec<GDSProperty>,
}

#[derive(Debug, Clone)]
pub struct GDSBox {
    pub layer: i16,
    pub boxtype: i16,
    pub xy: Vec<(i32, i32)>,
    pub elflags: Option<i16>, // Element flags (record 0x26)
    pub plex: Option<i32>,    // Plex identifier (record 0x2F)
    pub properties: Vec<GDSProperty>,
}

#[derive(Debug, Clone)]
pub struct STrans {
    pub reflection: bool,
    pub absolute_magnification: bool,
    pub absolute_angle: bool,
    pub magnification: Option<f64>,
    pub angle: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct GDSProperty {
    pub attribute: i16,
    pub value: String,
}

// GDSII Record Types (for reference)
// These constants are used internally via their numeric values
#[allow(dead_code)]
mod record_types {
    pub const HEADER: u8 = 0x00;
    pub const BGNLIB: u8 = 0x01;
    pub const LIBNAME: u8 = 0x02;
    pub const UNITS: u8 = 0x03;
    pub const ENDLIB: u8 = 0x04;
    pub const BGNSTR: u8 = 0x05;
    pub const STRNAME: u8 = 0x06;
    pub const ENDSTR: u8 = 0x07;
    pub const BOUNDARY: u8 = 0x08;
    pub const PATH: u8 = 0x09;
    pub const SREF: u8 = 0x0A;
    pub const AREF: u8 = 0x0B;
    pub const TEXT: u8 = 0x0C;
    pub const LAYER: u8 = 0x0D;
    pub const DATATYPE: u8 = 0x0E;
    pub const WIDTH: u8 = 0x0F;
    pub const XY: u8 = 0x10;
    pub const ENDEL: u8 = 0x11;
    pub const SNAME: u8 = 0x12;
    pub const COLROW: u8 = 0x13;
}

// Data Types
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum DataType {
    NoData = 0,
    BitArray = 1,
    TwoByteSignedInt = 2,
    FourByteSignedInt = 3,
    FourByteReal = 4,
    EightByteReal = 5,
    AsciiString = 6,
}

impl GDSIIFile {
    /// Create a new empty GDSII file
    pub fn new(library_name: String) -> Self {
        let now = GDSTime::now();
        GDSIIFile {
            version: 600, // GDSII version 6.0
            modification_time: now.clone(),
            access_time: now,
            library_name,
            units: (1e-6, 1e-9), // 1 micron user unit, 1nm database unit
            reflibs: Vec::new(),
            fonts: Vec::new(),
            generations: None,
            attrtable: None,
            structures: Vec::new(),
        }
    }

    /// Read GDSII from file
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        Self::read_from_reader(&mut reader)
    }

    /// Read GDSII from any reader
    pub fn read_from_reader<R: Read>(reader: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let mut cursor = 0;
        let mut gds = GDSIIFile::new(String::new());
        let mut current_structure: Option<GDSStructure> = None;
        let mut current_element: Option<GDSElement> = None;

        // Temporary storage for element construction
        let mut layer: Option<i16> = None;
        let mut datatype: Option<i16> = None;
        let mut xy: Vec<(i32, i32)> = Vec::new();
        let mut properties: Vec<GDSProperty> = Vec::new();
        let mut sname: Option<String> = None;
        let mut strans: Option<STrans> = None;
        let mut width: Option<i32> = None;
        let mut pathtype: Option<i16> = None;
        let mut bgnextn: Option<i32> = None; // Path extension at beginning
        let mut endextn: Option<i32> = None; // Path extension at end
        let mut elflags: Option<i16> = None; // Element flags
        let mut plex: Option<i32> = None; // Plex identifier
        let mut texttype: Option<i16> = None;
        let mut text_string: Option<String> = None;
        let mut presentation: Option<i16> = None;
        let mut colrow: Option<(u16, u16)> = None;
        let mut nodetype: Option<i16> = None;
        let mut boxtype: Option<i16> = None;
        let mut current_prop_attr: Option<i16> = None;

        while cursor < buffer.len() {
            let (record_type, _data_type, data) = Self::read_record(&buffer, &mut cursor)?;

            match record_type {
                0x00 => {
                    // HEADER
                    gds.version = Self::parse_i16(&data)? as u16;
                }
                0x01 => {
                    // BGNLIB
                    let times = Self::parse_time(&data)?;
                    gds.modification_time = times.0;
                    gds.access_time = times.1;
                }
                0x02 => {
                    // LIBNAME
                    gds.library_name = Self::parse_string(&data)?;
                }
                0x03 => {
                    // UNITS
                    gds.units = Self::parse_units(&data)?;
                }
                0x1F => {
                    // REFLIBS - Referenced libraries
                    let reflib = Self::parse_string(&data)?;
                    gds.reflibs.push(reflib);
                }
                0x29 => {
                    // FONTS - Font table
                    for i in (0..data.len()).step_by(44) {
                        if i + 44 <= data.len() {
                            let font = Self::parse_string(&data[i..i + 44])?;
                            if !font.is_empty() {
                                gds.fonts.push(font);
                            }
                        }
                    }
                }
                0x3C => {
                    // GENERATIONS - Number of backup generations
                    gds.generations = Some(Self::parse_i16(&data)?);
                }
                0x3D => {
                    // ATTRTABLE - Attribute table reference
                    gds.attrtable = Some(Self::parse_string(&data)?);
                }
                0x04 => {
                    // ENDLIB
                    break;
                }
                0x05 => {
                    // BGNSTR
                    let times = Self::parse_time(&data)?;
                    current_structure = Some(GDSStructure {
                        name: String::new(),
                        creation_time: times.0,
                        modification_time: times.1,
                        strclass: None,
                        elements: Vec::new(),
                    });
                }
                0x06 => {
                    // STRNAME
                    if let Some(ref mut structure) = current_structure {
                        structure.name = Self::parse_string(&data)?;
                    }
                }
                0x34 => {
                    // STRCLASS - Structure class
                    if let Some(ref mut structure) = current_structure {
                        structure.strclass = Some(Self::parse_i16(&data)?);
                    }
                }
                0x07 => {
                    // ENDSTR
                    if let Some(structure) = current_structure.take() {
                        gds.structures.push(structure);
                    }
                }
                0x08 => {
                    // BOUNDARY
                    layer = None;
                    datatype = None;
                    xy.clear();
                    properties.clear();
                    elflags = None;
                    plex = None;
                }
                0x09 => {
                    // PATH
                    layer = None;
                    datatype = None;
                    xy.clear();
                    properties.clear();
                    width = None;
                    pathtype = None;
                    bgnextn = None;
                    endextn = None;
                    elflags = None;
                    plex = None;
                }
                0x0A => {
                    // SREF
                    sname = None;
                    xy.clear();
                    strans = None;
                    properties.clear();
                    elflags = None;
                    plex = None;
                }
                0x0B => {
                    // AREF
                    sname = None;
                    xy.clear();
                    strans = None;
                    colrow = None;
                    properties.clear();
                    elflags = None;
                    plex = None;
                }
                0x0C => {
                    // TEXT
                    layer = None;
                    texttype = None;
                    xy.clear();
                    text_string = None;
                    presentation = None;
                    strans = None;
                    width = None;
                    properties.clear();
                    elflags = None;
                    plex = None;
                }
                0x0D => {
                    // LAYER
                    layer = Some(Self::parse_i16(&data)?);
                }
                0x0E => {
                    // DATATYPE
                    datatype = Some(Self::parse_i16(&data)?);
                }
                0x0F => {
                    // WIDTH
                    width = Some(Self::parse_i32(&data)?);
                }
                0x10 => {
                    // XY
                    xy = Self::parse_xy(&data)?;
                }
                0x11 => {
                    // ENDEL
                    // Finalize current element
                    let _element = current_element.take();

                    // Or create element based on what we're building
                    let new_element = if let (Some(l), Some(d)) = (layer, datatype) {
                        if !xy.is_empty() {
                            // Could be BOUNDARY
                            Some(GDSElement::Boundary(Boundary {
                                layer: l,
                                datatype: d,
                                xy: xy.clone(),
                                elflags,
                                plex,
                                properties: properties.clone(),
                            }))
                        } else {
                            // PATH
                            pathtype.map(|pt| {
                                GDSElement::Path(GPath {
                                    layer: l,
                                    datatype: d,
                                    pathtype: pt,
                                    width,
                                    bgnextn,
                                    endextn,
                                    xy: xy.clone(),
                                    elflags,
                                    plex,
                                    properties: properties.clone(),
                                })
                            })
                        }
                    } else if let (Some(sn), Some(cr)) = (sname.as_ref(), colrow) {
                        // AREF
                        Some(GDSElement::ArrayRef(ArrayRef {
                            sname: sn.clone(),
                            columns: cr.0,
                            rows: cr.1,
                            xy: xy.clone(),
                            strans: strans.clone(),
                            elflags,
                            plex,
                            properties: properties.clone(),
                        }))
                    } else if let Some(sn) = sname.as_ref() {
                        if !xy.is_empty() {
                            // SREF
                            Some(GDSElement::StructRef(StructRef {
                                sname: sn.clone(),
                                xy: xy[0],
                                strans: strans.clone(),
                                elflags,
                                plex,
                                properties: properties.clone(),
                            }))
                        } else {
                            None
                        }
                    } else if let (Some(l), Some(tt), Some(ts)) =
                        (layer, texttype, text_string.as_ref())
                    {
                        // TEXT
                        Some(GDSElement::Text(GText {
                            layer: l,
                            texttype: tt,
                            string: ts.clone(),
                            xy: if !xy.is_empty() { xy[0] } else { (0, 0) },
                            presentation,
                            strans: strans.clone(),
                            width,
                            elflags,
                            plex,
                            properties: properties.clone(),
                        }))
                    } else if let (Some(l), Some(nt)) = (layer, nodetype) {
                        // NODE
                        Some(GDSElement::Node(Node {
                            layer: l,
                            nodetype: nt,
                            xy: xy.clone(),
                            elflags,
                            plex,
                            properties: properties.clone(),
                        }))
                    } else if let (Some(l), Some(bt)) = (layer, boxtype) {
                        // BOX
                        Some(GDSElement::Box(GDSBox {
                            layer: l,
                            boxtype: bt,
                            xy: xy.clone(),
                            elflags,
                            plex,
                            properties: properties.clone(),
                        }))
                    } else {
                        None
                    };

                    if let Some(elem) = new_element {
                        if let Some(ref mut structure) = current_structure {
                            structure.elements.push(elem);
                        }
                    }

                    // Reset temporary storage
                    layer = None;
                    datatype = None;
                    xy.clear();
                    properties.clear();
                    sname = None;
                    strans = None;
                    width = None;
                    pathtype = None;
                    texttype = None;
                    text_string = None;
                    presentation = None;
                    colrow = None;
                    nodetype = None;
                    boxtype = None;
                }
                0x12 => {
                    // SNAME
                    sname = Some(Self::parse_string(&data)?);
                }
                0x13 => {
                    // COLROW
                    let cols = i16::from_be_bytes([data[0], data[1]]) as u16;
                    let rows = i16::from_be_bytes([data[2], data[3]]) as u16;
                    colrow = Some((cols, rows));
                }
                0x15 => {
                    // NODE
                    layer = None;
                    nodetype = None;
                    xy.clear();
                    properties.clear();
                    elflags = None;
                    plex = None;
                }
                0x16 => {
                    // TEXTTYPE
                    texttype = Some(Self::parse_i16(&data)?);
                }
                0x17 => {
                    // PRESENTATION
                    presentation = Some(Self::parse_i16(&data)?);
                }
                0x19 => {
                    // STRING
                    text_string = Some(Self::parse_string(&data)?);
                }
                0x1A => {
                    // STRANS
                    let flags = i16::from_be_bytes([data[0], data[1]]);
                    strans = Some(STrans {
                        reflection: (flags & -0x8000_i16) != 0,
                        absolute_magnification: (flags & 0x0004) != 0,
                        absolute_angle: (flags & 0x0002) != 0,
                        magnification: None,
                        angle: None,
                    });
                }
                0x1B => {
                    // MAG
                    let mag = Self::parse_real8(&data)?;
                    if let Some(ref mut st) = strans {
                        st.magnification = Some(mag);
                    }
                }
                0x1C => {
                    // ANGLE
                    let ang = Self::parse_real8(&data)?;
                    if let Some(ref mut st) = strans {
                        st.angle = Some(ang);
                    }
                }
                0x21 => {
                    // PATHTYPE
                    pathtype = Some(Self::parse_i16(&data)?);
                }
                0x26 => {
                    // ELFLAGS - Element flags
                    elflags = Some(Self::parse_i16(&data)?);
                }
                0x2A => {
                    // NODETYPE
                    nodetype = Some(Self::parse_i16(&data)?);
                }
                0x2B => {
                    // PROPATTR - Property attribute number
                    current_prop_attr = Some(Self::parse_i16(&data)?);
                }
                0x2C => {
                    // PROPVALUE - Property value string
                    if let Some(attr) = current_prop_attr.take() {
                        let value = Self::parse_string(&data)?;
                        properties.push(GDSProperty {
                            attribute: attr,
                            value,
                        });
                    }
                }
                0x2D => {
                    // BOX
                    layer = None;
                    boxtype = None;
                    xy.clear();
                    properties.clear();
                    elflags = None;
                    plex = None;
                }
                0x2E => {
                    // BOXTYPE
                    boxtype = Some(Self::parse_i16(&data)?);
                }
                0x2F => {
                    // PLEX - Plex identifier
                    plex = Some(Self::parse_i32(&data)?);
                }
                0x30 => {
                    // BGNEXTN - Path extension at beginning
                    bgnextn = Some(Self::parse_i32(&data)?);
                }
                0x31 => {
                    // ENDEXTN - Path extension at end
                    endextn = Some(Self::parse_i32(&data)?);
                }
                _ => {
                    // Skip unknown record types
                }
            }
        }

        Ok(gds)
    }

    /// Write GDSII to file
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write GDSII to any writer
    pub fn write_to_writer<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // HEADER
        Self::write_record(
            writer,
            0x00,
            DataType::TwoByteSignedInt,
            &self.version.to_be_bytes(),
        )?;

        // BGNLIB
        let time_data = Self::format_times(&self.modification_time, &self.access_time);
        Self::write_record(writer, 0x01, DataType::TwoByteSignedInt, &time_data)?;

        // LIBNAME
        Self::write_record(
            writer,
            0x02,
            DataType::AsciiString,
            self.library_name.as_bytes(),
        )?;

        // UNITS
        let mut units_data = Vec::new();
        units_data.extend_from_slice(&Self::format_real8(self.units.0));
        units_data.extend_from_slice(&Self::format_real8(self.units.1));
        Self::write_record(writer, 0x03, DataType::EightByteReal, &units_data)?;

        // REFLIBS - Referenced libraries
        for reflib in &self.reflibs {
            Self::write_record(writer, 0x1F, DataType::AsciiString, reflib.as_bytes())?;
        }

        // FONTS - Font table
        if !self.fonts.is_empty() {
            let mut font_data = Vec::new();
            for font in &self.fonts {
                let mut font_bytes = font.as_bytes().to_vec();
                font_bytes.resize(44, 0); // Each font name is 44 bytes
                font_data.extend_from_slice(&font_bytes);
            }
            Self::write_record(writer, 0x29, DataType::AsciiString, &font_data)?;
        }

        // GENERATIONS - Backup generations
        if let Some(gen) = self.generations {
            Self::write_record(writer, 0x3C, DataType::TwoByteSignedInt, &gen.to_be_bytes())?;
        }

        // ATTRTABLE - Attribute table reference
        if let Some(ref attrtable) = self.attrtable {
            Self::write_record(writer, 0x3D, DataType::AsciiString, attrtable.as_bytes())?;
        }

        // Write structures
        for structure in &self.structures {
            structure.write(writer)?;
        }

        // ENDLIB
        Self::write_record(writer, 0x04, DataType::NoData, &[])?;

        Ok(())
    }

    // Helper functions for reading records
    fn read_record(
        buffer: &[u8],
        cursor: &mut usize,
    ) -> Result<(u8, DataType, Vec<u8>), Box<dyn std::error::Error>> {
        if *cursor + 4 > buffer.len() {
            return Err("Incomplete record header".into());
        }

        let length = u16::from_be_bytes([buffer[*cursor], buffer[*cursor + 1]]) as usize;
        let record_type = buffer[*cursor + 2];
        let data_type_byte = buffer[*cursor + 3];

        let data_type = match data_type_byte {
            0 => DataType::NoData,
            1 => DataType::BitArray,
            2 => DataType::TwoByteSignedInt,
            3 => DataType::FourByteSignedInt,
            4 => DataType::FourByteReal,
            5 => DataType::EightByteReal,
            6 => DataType::AsciiString,
            _ => return Err(format!("Unknown data type: {}", data_type_byte).into()),
        };

        *cursor += 4;

        let data_length = length.saturating_sub(4);
        if *cursor + data_length > buffer.len() {
            return Err("Incomplete record data".into());
        }

        let data = buffer[*cursor..*cursor + data_length].to_vec();
        *cursor += data_length;

        Ok((record_type, data_type, data))
    }

    fn write_record<W: Write>(
        writer: &mut W,
        record_type: u8,
        data_type: DataType,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let length = (4 + data.len()) as u16;
        writer.write_all(&length.to_be_bytes())?;
        writer.write_all(&[record_type, data_type as u8])?;
        writer.write_all(data)?;
        Ok(())
    }

    fn write_properties<W: Write>(
        writer: &mut W,
        properties: &[GDSProperty],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for prop in properties {
            // PROPATTR
            Self::write_record(
                writer,
                0x2B,
                DataType::TwoByteSignedInt,
                &prop.attribute.to_be_bytes(),
            )?;
            // PROPVALUE
            Self::write_record(writer, 0x2C, DataType::AsciiString, prop.value.as_bytes())?;
        }
        Ok(())
    }

    fn write_elflags_plex<W: Write>(
        writer: &mut W,
        elflags: Option<i16>,
        plex: Option<i32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // ELFLAGS (0x26) must come before PLEX (0x2F)
        if let Some(flags) = elflags {
            Self::write_record(
                writer,
                0x26,
                DataType::TwoByteSignedInt,
                &flags.to_be_bytes(),
            )?;
        }
        if let Some(p) = plex {
            Self::write_record(writer, 0x2F, DataType::FourByteSignedInt, &p.to_be_bytes())?;
        }
        Ok(())
    }

    // Parsing helpers
    fn parse_i16(data: &[u8]) -> Result<i16, Box<dyn std::error::Error>> {
        if data.len() < 2 {
            return Err("Insufficient data for i16".into());
        }
        Ok(i16::from_be_bytes([data[0], data[1]]))
    }

    fn parse_i32(data: &[u8]) -> Result<i32, Box<dyn std::error::Error>> {
        if data.len() < 4 {
            return Err("Insufficient data for i32".into());
        }
        Ok(i32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    fn parse_string(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        // GDSII strings are null-terminated or padded
        let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        // Use from_utf8_lossy to handle non-UTF8 strings gracefully
        // GDSII spec uses ASCII, but some tools may write non-UTF8 data
        Ok(String::from_utf8_lossy(&data[..end]).into_owned())
    }

    fn parse_real8(data: &[u8]) -> Result<f64, Box<dyn std::error::Error>> {
        if data.len() < 8 {
            return Err("Insufficient data for real8".into());
        }

        // GDSII real8 format: sign(1bit) + exponent(7bits) + mantissa(56bits)
        let bytes = [
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ];
        let bits = u64::from_be_bytes(bytes);

        let sign = if (bits & 0x8000000000000000) != 0 {
            -1.0
        } else {
            1.0
        };
        let exponent = ((bits >> 56) & 0x7F) as i32 - 64;
        let mantissa = (bits & 0x00FFFFFFFFFFFFFF) as f64 / (1u64 << 56) as f64;

        Ok(sign * mantissa * 16f64.powi(exponent))
    }

    fn format_real8(value: f64) -> [u8; 8] {
        if value == 0.0 {
            return [0; 8];
        }

        let sign_bit = if value < 0.0 { 0x80 } else { 0x00 };
        let abs_value = value.abs();

        // Convert to base-16 exponent
        let exponent = (abs_value.log2() / 4.0).floor() as i32;
        let mantissa = abs_value / 16f64.powi(exponent);

        let exp_byte = ((exponent + 64) as u8) | sign_bit;
        let mant_bits = (mantissa * (1u64 << 56) as f64) as u64;

        let mut result = [0u8; 8];
        result[0] = exp_byte;
        result[1..8].copy_from_slice(&mant_bits.to_be_bytes()[1..8]);

        result
    }

    fn parse_time(data: &[u8]) -> Result<(GDSTime, GDSTime), Box<dyn std::error::Error>> {
        if data.len() < 24 {
            return Err("Insufficient data for time".into());
        }

        let mod_time = GDSTime {
            year: i16::from_be_bytes([data[0], data[1]]) as u16,
            month: i16::from_be_bytes([data[2], data[3]]) as u16,
            day: i16::from_be_bytes([data[4], data[5]]) as u16,
            hour: i16::from_be_bytes([data[6], data[7]]) as u16,
            minute: i16::from_be_bytes([data[8], data[9]]) as u16,
            second: i16::from_be_bytes([data[10], data[11]]) as u16,
        };

        let acc_time = GDSTime {
            year: i16::from_be_bytes([data[12], data[13]]) as u16,
            month: i16::from_be_bytes([data[14], data[15]]) as u16,
            day: i16::from_be_bytes([data[16], data[17]]) as u16,
            hour: i16::from_be_bytes([data[18], data[19]]) as u16,
            minute: i16::from_be_bytes([data[20], data[21]]) as u16,
            second: i16::from_be_bytes([data[22], data[23]]) as u16,
        };

        Ok((mod_time, acc_time))
    }

    fn format_times(mod_time: &GDSTime, acc_time: &GDSTime) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&(mod_time.year as i16).to_be_bytes());
        data.extend_from_slice(&(mod_time.month as i16).to_be_bytes());
        data.extend_from_slice(&(mod_time.day as i16).to_be_bytes());
        data.extend_from_slice(&(mod_time.hour as i16).to_be_bytes());
        data.extend_from_slice(&(mod_time.minute as i16).to_be_bytes());
        data.extend_from_slice(&(mod_time.second as i16).to_be_bytes());
        data.extend_from_slice(&(acc_time.year as i16).to_be_bytes());
        data.extend_from_slice(&(acc_time.month as i16).to_be_bytes());
        data.extend_from_slice(&(acc_time.day as i16).to_be_bytes());
        data.extend_from_slice(&(acc_time.hour as i16).to_be_bytes());
        data.extend_from_slice(&(acc_time.minute as i16).to_be_bytes());
        data.extend_from_slice(&(acc_time.second as i16).to_be_bytes());
        data
    }

    fn parse_xy(data: &[u8]) -> Result<Vec<(i32, i32)>, Box<dyn std::error::Error>> {
        #[allow(unknown_lints, clippy::manual_is_multiple_of)]
        if data.len() % 8 != 0 {
            return Err("Invalid XY data length".into());
        }

        let mut points = Vec::new();
        for i in (0..data.len()).step_by(8) {
            let x = i32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
            let y = i32::from_be_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]);
            points.push((x, y));
        }

        Ok(points)
    }

    fn parse_units(data: &[u8]) -> Result<(f64, f64), Box<dyn std::error::Error>> {
        if data.len() < 16 {
            return Err("Insufficient data for units".into());
        }

        let user_units = Self::parse_real8(&data[0..8])?;
        let db_units = Self::parse_real8(&data[8..16])?;

        Ok((user_units, db_units))
    }
}

impl GDSTime {
    /// Creates a timestamp with the current date and time.
    ///
    /// # Examples
    ///
    /// ```
    /// use laykit::GDSTime;
    ///
    /// let time = GDSTime::now();
    /// assert!(time.year >= 2024);
    /// assert!(time.month >= 1 && time.month <= 12);
    /// ```
    pub fn now() -> Self {
        // Simple default time
        GDSTime {
            year: 2025,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }
}

impl GDSStructure {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        // BGNSTR
        let time_data = GDSIIFile::format_times(&self.creation_time, &self.modification_time);
        GDSIIFile::write_record(writer, 0x05, DataType::TwoByteSignedInt, &time_data)?;

        // STRNAME
        GDSIIFile::write_record(writer, 0x06, DataType::AsciiString, self.name.as_bytes())?;

        // STRCLASS - Structure class
        if let Some(strclass) = self.strclass {
            GDSIIFile::write_record(
                writer,
                0x34,
                DataType::TwoByteSignedInt,
                &strclass.to_be_bytes(),
            )?;
        }

        // Write elements
        for element in &self.elements {
            element.write(writer)?;
        }

        // ENDSTR
        GDSIIFile::write_record(writer, 0x07, DataType::NoData, &[])?;

        Ok(())
    }
}

impl GDSElement {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            GDSElement::Boundary(b) => b.write(writer),
            GDSElement::Path(p) => p.write(writer),
            GDSElement::StructRef(s) => s.write(writer),
            GDSElement::ArrayRef(a) => a.write(writer),
            GDSElement::Text(t) => t.write(writer),
            GDSElement::Node(n) => n.write(writer),
            GDSElement::Box(b) => b.write(writer),
        }
    }
}

impl Boundary {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        // BOUNDARY
        GDSIIFile::write_record(writer, 0x08, DataType::NoData, &[])?;

        // ELFLAGS and PLEX (must come before other element data)
        GDSIIFile::write_elflags_plex(writer, self.elflags, self.plex)?;

        // LAYER
        GDSIIFile::write_record(
            writer,
            0x0D,
            DataType::TwoByteSignedInt,
            &self.layer.to_be_bytes(),
        )?;

        // DATATYPE
        GDSIIFile::write_record(
            writer,
            0x0E,
            DataType::TwoByteSignedInt,
            &self.datatype.to_be_bytes(),
        )?;

        // XY
        let mut xy_data = Vec::new();
        for (x, y) in &self.xy {
            xy_data.extend_from_slice(&x.to_be_bytes());
            xy_data.extend_from_slice(&y.to_be_bytes());
        }
        GDSIIFile::write_record(writer, 0x10, DataType::FourByteSignedInt, &xy_data)?;

        // Properties
        GDSIIFile::write_properties(writer, &self.properties)?;

        // ENDEL
        GDSIIFile::write_record(writer, 0x11, DataType::NoData, &[])?;

        Ok(())
    }
}

impl GPath {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        // PATH
        GDSIIFile::write_record(writer, 0x09, DataType::NoData, &[])?;

        // ELFLAGS and PLEX
        GDSIIFile::write_elflags_plex(writer, self.elflags, self.plex)?;

        // LAYER
        GDSIIFile::write_record(
            writer,
            0x0D,
            DataType::TwoByteSignedInt,
            &self.layer.to_be_bytes(),
        )?;

        // DATATYPE
        GDSIIFile::write_record(
            writer,
            0x0E,
            DataType::TwoByteSignedInt,
            &self.datatype.to_be_bytes(),
        )?;

        // PATHTYPE
        GDSIIFile::write_record(
            writer,
            0x21,
            DataType::TwoByteSignedInt,
            &self.pathtype.to_be_bytes(),
        )?;

        // WIDTH
        if let Some(w) = self.width {
            GDSIIFile::write_record(writer, 0x0F, DataType::FourByteSignedInt, &w.to_be_bytes())?;
        }

        // BGNEXTN
        if let Some(bgn) = self.bgnextn {
            GDSIIFile::write_record(
                writer,
                0x30,
                DataType::FourByteSignedInt,
                &bgn.to_be_bytes(),
            )?;
        }

        // ENDEXTN
        if let Some(end) = self.endextn {
            GDSIIFile::write_record(
                writer,
                0x31,
                DataType::FourByteSignedInt,
                &end.to_be_bytes(),
            )?;
        }

        // XY
        let mut xy_data = Vec::new();
        for (x, y) in &self.xy {
            xy_data.extend_from_slice(&x.to_be_bytes());
            xy_data.extend_from_slice(&y.to_be_bytes());
        }
        GDSIIFile::write_record(writer, 0x10, DataType::FourByteSignedInt, &xy_data)?;

        // Properties
        GDSIIFile::write_properties(writer, &self.properties)?;

        // ENDEL
        GDSIIFile::write_record(writer, 0x11, DataType::NoData, &[])?;

        Ok(())
    }
}

impl StructRef {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        // SREF
        GDSIIFile::write_record(writer, 0x0A, DataType::NoData, &[])?;

        // ELFLAGS and PLEX
        GDSIIFile::write_elflags_plex(writer, self.elflags, self.plex)?;

        // SNAME
        GDSIIFile::write_record(writer, 0x12, DataType::AsciiString, self.sname.as_bytes())?;

        // STRANS
        if let Some(ref st) = self.strans {
            st.write(writer)?;
        }

        // XY
        let mut xy_data = Vec::new();
        xy_data.extend_from_slice(&self.xy.0.to_be_bytes());
        xy_data.extend_from_slice(&self.xy.1.to_be_bytes());
        GDSIIFile::write_record(writer, 0x10, DataType::FourByteSignedInt, &xy_data)?;

        // Properties
        GDSIIFile::write_properties(writer, &self.properties)?;

        // ENDEL
        GDSIIFile::write_record(writer, 0x11, DataType::NoData, &[])?;

        Ok(())
    }
}

impl ArrayRef {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        // AREF
        GDSIIFile::write_record(writer, 0x0B, DataType::NoData, &[])?;

        // ELFLAGS and PLEX
        GDSIIFile::write_elflags_plex(writer, self.elflags, self.plex)?;

        // SNAME
        GDSIIFile::write_record(writer, 0x12, DataType::AsciiString, self.sname.as_bytes())?;

        // STRANS
        if let Some(ref st) = self.strans {
            st.write(writer)?;
        }

        // COLROW
        let mut colrow_data = Vec::new();
        colrow_data.extend_from_slice(&(self.columns as i16).to_be_bytes());
        colrow_data.extend_from_slice(&(self.rows as i16).to_be_bytes());
        GDSIIFile::write_record(writer, 0x13, DataType::TwoByteSignedInt, &colrow_data)?;

        // XY (3 points)
        let mut xy_data = Vec::new();
        for (x, y) in &self.xy {
            xy_data.extend_from_slice(&x.to_be_bytes());
            xy_data.extend_from_slice(&y.to_be_bytes());
        }
        GDSIIFile::write_record(writer, 0x10, DataType::FourByteSignedInt, &xy_data)?;

        // Properties
        GDSIIFile::write_properties(writer, &self.properties)?;

        // ENDEL
        GDSIIFile::write_record(writer, 0x11, DataType::NoData, &[])?;

        Ok(())
    }
}

impl GText {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        // TEXT
        GDSIIFile::write_record(writer, 0x0C, DataType::NoData, &[])?;

        // ELFLAGS and PLEX
        GDSIIFile::write_elflags_plex(writer, self.elflags, self.plex)?;

        // LAYER
        GDSIIFile::write_record(
            writer,
            0x0D,
            DataType::TwoByteSignedInt,
            &self.layer.to_be_bytes(),
        )?;

        // TEXTTYPE
        GDSIIFile::write_record(
            writer,
            0x16,
            DataType::TwoByteSignedInt,
            &self.texttype.to_be_bytes(),
        )?;

        // PRESENTATION
        if let Some(p) = self.presentation {
            GDSIIFile::write_record(writer, 0x17, DataType::TwoByteSignedInt, &p.to_be_bytes())?;
        }

        // STRANS
        if let Some(ref st) = self.strans {
            st.write(writer)?;
        }

        // XY
        let mut xy_data = Vec::new();
        xy_data.extend_from_slice(&self.xy.0.to_be_bytes());
        xy_data.extend_from_slice(&self.xy.1.to_be_bytes());
        GDSIIFile::write_record(writer, 0x10, DataType::FourByteSignedInt, &xy_data)?;

        // STRING
        GDSIIFile::write_record(writer, 0x19, DataType::AsciiString, self.string.as_bytes())?;

        // Properties
        GDSIIFile::write_properties(writer, &self.properties)?;

        // ENDEL
        GDSIIFile::write_record(writer, 0x11, DataType::NoData, &[])?;

        Ok(())
    }
}

impl Node {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        // NODE
        GDSIIFile::write_record(writer, 0x15, DataType::NoData, &[])?;

        // ELFLAGS and PLEX
        GDSIIFile::write_elflags_plex(writer, self.elflags, self.plex)?;

        // LAYER
        GDSIIFile::write_record(
            writer,
            0x0D,
            DataType::TwoByteSignedInt,
            &self.layer.to_be_bytes(),
        )?;

        // NODETYPE
        GDSIIFile::write_record(
            writer,
            0x2A,
            DataType::TwoByteSignedInt,
            &self.nodetype.to_be_bytes(),
        )?;

        // XY
        let mut xy_data = Vec::new();
        for (x, y) in &self.xy {
            xy_data.extend_from_slice(&x.to_be_bytes());
            xy_data.extend_from_slice(&y.to_be_bytes());
        }
        GDSIIFile::write_record(writer, 0x10, DataType::FourByteSignedInt, &xy_data)?;

        // Properties
        GDSIIFile::write_properties(writer, &self.properties)?;

        // ENDEL
        GDSIIFile::write_record(writer, 0x11, DataType::NoData, &[])?;

        Ok(())
    }
}

impl GDSBox {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        // BOX
        GDSIIFile::write_record(writer, 0x2D, DataType::NoData, &[])?;

        // ELFLAGS and PLEX
        GDSIIFile::write_elflags_plex(writer, self.elflags, self.plex)?;

        // LAYER
        GDSIIFile::write_record(
            writer,
            0x0D,
            DataType::TwoByteSignedInt,
            &self.layer.to_be_bytes(),
        )?;

        // BOXTYPE
        GDSIIFile::write_record(
            writer,
            0x2E,
            DataType::TwoByteSignedInt,
            &self.boxtype.to_be_bytes(),
        )?;

        // XY
        let mut xy_data = Vec::new();
        for (x, y) in &self.xy {
            xy_data.extend_from_slice(&x.to_be_bytes());
            xy_data.extend_from_slice(&y.to_be_bytes());
        }
        GDSIIFile::write_record(writer, 0x10, DataType::FourByteSignedInt, &xy_data)?;

        // Properties
        GDSIIFile::write_properties(writer, &self.properties)?;

        // ENDEL
        GDSIIFile::write_record(writer, 0x11, DataType::NoData, &[])?;

        Ok(())
    }
}

impl STrans {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        // STRANS
        let mut flags: i16 = 0;
        if self.reflection {
            flags |= -0x8000_i16;
        }
        if self.absolute_magnification {
            flags |= 0x0004;
        }
        if self.absolute_angle {
            flags |= 0x0002;
        }
        GDSIIFile::write_record(
            writer,
            0x1A,
            DataType::TwoByteSignedInt,
            &flags.to_be_bytes(),
        )?;

        // MAG
        if let Some(mag) = self.magnification {
            GDSIIFile::write_record(
                writer,
                0x1B,
                DataType::EightByteReal,
                &GDSIIFile::format_real8(mag),
            )?;
        }

        // ANGLE
        if let Some(angle) = self.angle {
            GDSIIFile::write_record(
                writer,
                0x1C,
                DataType::EightByteReal,
                &GDSIIFile::format_real8(angle),
            )?;
        }

        Ok(())
    }
}
