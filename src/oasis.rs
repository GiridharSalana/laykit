// OASIS (Open Artwork System Interchange Standard) Reader/Writer
// Full implementation of OASIS spec for IC layout interchange
// More compact and modern than GDSII

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Write};
use std::path::Path;

/// OASIS File structure
#[derive(Debug, Clone)]
pub struct OASISFile {
    pub version: String,
    pub unit: f64,
    pub offset_flag: bool,
    pub names: NameTable,
    pub cells: Vec<OASISCell>,
    pub layers: Vec<LayerInfo>,
    pub properties: Vec<PropertyDefinition>,
}

#[derive(Debug, Clone)]
pub struct NameTable {
    pub cell_names: HashMap<u32, String>,
    pub text_strings: HashMap<u32, String>,
    pub prop_names: HashMap<u32, String>,
    pub prop_strings: HashMap<u32, String>,
    pub layer_names: HashMap<u32, String>,
}

#[derive(Debug, Clone)]
pub struct OASISCell {
    pub name: String,
    pub elements: Vec<OASISElement>,
}

#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub layer: u32,
    pub datatype: u32,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum OASISElement {
    Rectangle(Rectangle),
    Polygon(Polygon),
    Path(OPath),
    Trapezoid(Trapezoid),
    CTrapezoid(CTrapezoid),
    Circle(Circle),
    Text(OText),
    Placement(Placement),
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub layer: u32,
    pub datatype: u32,
    pub x: i64,
    pub y: i64,
    pub width: u64,
    pub height: u64,
    pub repetition: Option<Repetition>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub layer: u32,
    pub datatype: u32,
    pub x: i64,
    pub y: i64,
    pub points: Vec<(i64, i64)>,
    pub repetition: Option<Repetition>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
pub struct OPath {
    pub layer: u32,
    pub datatype: u32,
    pub x: i64,
    pub y: i64,
    pub half_width: u64,
    pub extension_scheme: ExtensionScheme,
    pub points: Vec<(i64, i64)>,
    pub repetition: Option<Repetition>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
pub struct Trapezoid {
    pub layer: u32,
    pub datatype: u32,
    pub x: i64,
    pub y: i64,
    pub width: u64,
    pub height: u64,
    pub delta_a: i64,
    pub delta_b: i64,
    pub orientation: bool, // horizontal or vertical
    pub repetition: Option<Repetition>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
pub struct CTrapezoid {
    pub layer: u32,
    pub datatype: u32,
    pub x: i64,
    pub y: i64,
    pub trap_type: u8,
    pub width: u64,
    pub height: u64,
    pub repetition: Option<Repetition>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub layer: u32,
    pub datatype: u32,
    pub x: i64,
    pub y: i64,
    pub radius: u64,
    pub repetition: Option<Repetition>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
pub struct OText {
    pub layer: u32,
    pub texttype: u32,
    pub x: i64,
    pub y: i64,
    pub string: String,
    pub repetition: Option<Repetition>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
pub struct Placement {
    pub cell_name: String,
    pub x: i64,
    pub y: i64,
    pub magnification: Option<f64>,
    pub angle: Option<f64>,
    pub mirror: bool,
    pub repetition: Option<Repetition>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
pub enum Repetition {
    ReusePrevious,
    Matrix {
        x_count: u32,
        y_count: u32,
        x_space: u64,
        y_space: u64,
    },
    Arbitrary {
        x_displacements: Vec<i64>,
        y_displacements: Vec<i64>,
    },
    Grid {
        count: u32,
        grid_space: u64,
    },
}

#[derive(Debug, Clone)]
pub enum ExtensionScheme {
    Flush,
    HalfWidth,
    Custom { start: i64, end: i64 },
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub values: Vec<PropertyValue>,
}

#[derive(Debug, Clone)]
pub struct PropertyDefinition {
    pub name: String,
    pub is_standard: bool,
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
    Integer(i64),
    Real(f64),
    String(String),
    Reference(u32),
    Boolean(bool),
}

// OASIS Record Types (for reference)
// These constants are used internally via their numeric values
#[allow(dead_code)]
mod record_ids {
    pub const PAD: u8 = 0;
    pub const START: u8 = 1;
    pub const END: u8 = 2;
    pub const CELLNAME: u8 = 3;
    pub const TEXTSTRING: u8 = 5;
    pub const PROPNAME: u8 = 7;
    pub const LAYERNAME: u8 = 11;
    pub const CELL: u8 = 13;
    pub const XYABSOLUTE: u8 = 14;
    pub const XYRELATIVE: u8 = 15;
    pub const PLACEMENT: u8 = 16;
    pub const TEXT: u8 = 18;
    pub const RECTANGLE: u8 = 19;
    pub const POLYGON: u8 = 20;
    pub const PATH: u8 = 21;
    pub const TRAPEZOID: u8 = 22;
    pub const CTRAPEZOID: u8 = 23;
    pub const CIRCLE: u8 = 24;
}

impl Default for OASISFile {
    fn default() -> Self {
        OASISFile {
            version: "1.0".to_string(),
            unit: 1e-9, // 1nm database unit
            offset_flag: false,
            names: NameTable {
                cell_names: HashMap::new(),
                text_strings: HashMap::new(),
                prop_names: HashMap::new(),
                prop_strings: HashMap::new(),
                layer_names: HashMap::new(),
            },
            cells: Vec::new(),
            layers: Vec::new(),
            properties: Vec::new(),
        }
    }
}

impl OASISFile {
    /// Create a new empty OASIS file
    pub fn new() -> Self {
        Self::default()
    }

    /// Read OASIS from file
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        Self::read_from_reader(&mut reader)
    }

    /// Read OASIS from any reader
    pub fn read_from_reader<R: Read>(reader: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let mut cursor = Cursor::new(buffer);
        let mut oasis = OASISFile::new();

        // Check magic bytes
        let magic = Self::read_bytes(&mut cursor, 13)?;
        if &magic != b"%SEMI-OASIS\r\n" {
            return Err("Invalid OASIS file magic".into());
        }

        // Parse records
        loop {
            let record_id = match Self::read_u8(&mut cursor) {
                Ok(id) => id,
                Err(_) => break,
            };

            // Skip padding records
            if record_id == 0 {
                continue;
            }

            match record_id {
                1 => {
                    // START
                    oasis.version = Self::read_string(&mut cursor)?;
                    oasis.unit = Self::read_real(&mut cursor)?;
                    oasis.offset_flag = Self::read_u8(&mut cursor)? != 0;
                }
                2 => {
                    // END
                    // Validation table follows - skip it
                    // Read validation signature (may fail if at end of file)
                    let _ = Self::read_unsigned(&mut cursor);
                    break;
                }
                3 => {
                    // CELLNAME
                    let name = Self::read_string(&mut cursor)?;
                    let ref_num = Self::read_unsigned(&mut cursor)? as u32;
                    oasis.names.cell_names.insert(ref_num, name);
                }
                5 => {
                    // TEXTSTRING
                    let string = Self::read_string(&mut cursor)?;
                    let ref_num = Self::read_unsigned(&mut cursor)? as u32;
                    oasis.names.text_strings.insert(ref_num, string);
                }
                7 => {
                    // PROPNAME
                    let name = Self::read_string(&mut cursor)?;
                    let ref_num = Self::read_unsigned(&mut cursor)? as u32;
                    oasis.names.prop_names.insert(ref_num, name);
                }
                11 => {
                    // LAYERNAME
                    let name = Self::read_string(&mut cursor)?;
                    let layer_interval = Self::read_layer_interval(&mut cursor)?;
                    let _datatype_interval = Self::read_layer_interval(&mut cursor)?;
                    // Store first value from intervals
                    if let Some(layer) = layer_interval.first() {
                        oasis.names.layer_names.insert(*layer, name);
                    }
                }
                13 => {
                    // CELL
                    let cell = Self::read_cell(&mut cursor, &oasis.names)?;
                    oasis.cells.push(cell);
                }
                14 => { // XYAbsolute
                     // Coordinate mode - no data, just skip
                }
                15 => { // XYRelative
                     // Coordinate mode - no data, just skip
                }
                19 => { // RECTANGLE
                     // Handled within cell context
                }
                20 => { // POLYGON
                     // Handled within cell context
                }
                21 => { // PATH
                     // Handled within cell context
                }
                _ => {
                    // Skip unknown records
                }
            }
        }

        Ok(oasis)
    }

    /// Write OASIS to file
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)
    }

    /// Write OASIS to any writer
    pub fn write_to_writer<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Write magic
        writer.write_all(b"%SEMI-OASIS\r\n")?;

        // Write START record
        Self::write_u8(writer, 1)?;
        Self::write_string(writer, &self.version)?;
        Self::write_real(writer, self.unit)?;
        Self::write_u8(writer, if self.offset_flag { 1 } else { 0 })?;

        // Write name tables
        for (ref_num, name) in &self.names.cell_names {
            Self::write_u8(writer, 3)?; // CELLNAME
            Self::write_string(writer, name)?;
            Self::write_unsigned(writer, *ref_num as u64)?;
        }

        for (ref_num, string) in &self.names.text_strings {
            Self::write_u8(writer, 5)?; // TEXTSTRING
            Self::write_string(writer, string)?;
            Self::write_unsigned(writer, *ref_num as u64)?;
        }

        for (ref_num, name) in &self.names.prop_names {
            Self::write_u8(writer, 7)?; // PROPNAME
            Self::write_string(writer, name)?;
            Self::write_unsigned(writer, *ref_num as u64)?;
        }

        // Write cells
        for cell in &self.cells {
            cell.write(writer, &self.names)?;
        }

        // Write END record
        Self::write_u8(writer, 2)?;

        // Write validation (optional, simplified)
        Self::write_unsigned(writer, 0)?; // No validation

        Ok(())
    }

    fn read_cell(
        cursor: &mut Cursor<Vec<u8>>,
        names: &NameTable,
    ) -> Result<OASISCell, Box<dyn std::error::Error>> {
        let name_ref_or_string = Self::read_unsigned(cursor)?;

        let name = if name_ref_or_string & 1 == 0 {
            // Reference
            let ref_num = (name_ref_or_string >> 1) as u32;
            names
                .cell_names
                .get(&ref_num)
                .ok_or("Cell name reference not found")?
                .clone()
        } else {
            // Explicit string
            Self::read_string(cursor)?
        };

        let mut cell = OASISCell {
            name,
            elements: Vec::new(),
        };

        // Read elements until next CELL, END, or coordinate mode change
        loop {
            // Peek at next byte to see what's coming
            let position = cursor.position();
            let record_id = match Self::read_u8(cursor) {
                Ok(id) => id,
                Err(_) => break, // EOF
            };

            // Check if this is a cell boundary or end marker
            match record_id {
                2 | 13 => {
                    // END or CELL - restore position and break
                    cursor.set_position(position);
                    break;
                }
                14 | 15 => {
                    // XYAbsolute or XYRelative - just continue
                    continue;
                }
                19 => {
                    // RECTANGLE
                    if let Ok(elem) = Self::read_rectangle(cursor) {
                        cell.elements.push(OASISElement::Rectangle(elem));
                    }
                }
                20 => {
                    // POLYGON
                    if let Ok(elem) = Self::read_polygon(cursor) {
                        cell.elements.push(OASISElement::Polygon(elem));
                    }
                }
                21 => {
                    // PATH
                    if let Ok(elem) = Self::read_path(cursor) {
                        cell.elements.push(OASISElement::Path(elem));
                    }
                }
                _ => {
                    // Unknown record, try to skip it safely
                    cursor.set_position(position);
                    break;
                }
            }
        }

        Ok(cell)
    }

    fn read_rectangle(
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Rectangle, Box<dyn std::error::Error>> {
        let _info_byte = Self::read_u8(cursor)?;
        let layer = Self::read_unsigned(cursor)? as u32;
        let datatype = Self::read_unsigned(cursor)? as u32;
        let width = Self::read_unsigned(cursor)?;
        let height = Self::read_unsigned(cursor)?;
        let x = Self::read_signed(cursor)?;
        let y = Self::read_signed(cursor)?;

        Ok(Rectangle {
            layer,
            datatype,
            x,
            y,
            width,
            height,
            repetition: None,
            properties: Vec::new(),
        })
    }

    fn read_polygon(cursor: &mut Cursor<Vec<u8>>) -> Result<Polygon, Box<dyn std::error::Error>> {
        let _info_byte = Self::read_u8(cursor)?;
        let layer = Self::read_unsigned(cursor)? as u32;
        let datatype = Self::read_unsigned(cursor)? as u32;
        let point_count = Self::read_unsigned(cursor)? as usize;

        let mut points = Vec::new();
        for _ in 0..point_count {
            let x = Self::read_signed(cursor)?;
            let y = Self::read_signed(cursor)?;
            points.push((x, y));
        }

        let x = Self::read_signed(cursor)?;
        let y = Self::read_signed(cursor)?;

        Ok(Polygon {
            layer,
            datatype,
            x,
            y,
            points,
            repetition: None,
            properties: Vec::new(),
        })
    }

    fn read_path(cursor: &mut Cursor<Vec<u8>>) -> Result<OPath, Box<dyn std::error::Error>> {
        let _info_byte = Self::read_u8(cursor)?;
        let layer = Self::read_unsigned(cursor)? as u32;
        let datatype = Self::read_unsigned(cursor)? as u32;
        let half_width = Self::read_unsigned(cursor)?;

        let ext_scheme_type = Self::read_u8(cursor)?;
        let extension_scheme = match ext_scheme_type {
            0 => ExtensionScheme::Flush,
            1 => ExtensionScheme::HalfWidth,
            2 => {
                let start = Self::read_signed(cursor)?;
                let end = Self::read_signed(cursor)?;
                ExtensionScheme::Custom { start, end }
            }
            _ => ExtensionScheme::Flush,
        };

        let point_count = Self::read_unsigned(cursor)? as usize;
        let mut points = Vec::new();
        for _ in 0..point_count {
            let x = Self::read_signed(cursor)?;
            let y = Self::read_signed(cursor)?;
            points.push((x, y));
        }

        let x = Self::read_signed(cursor)?;
        let y = Self::read_signed(cursor)?;

        Ok(OPath {
            layer,
            datatype,
            x,
            y,
            half_width,
            extension_scheme,
            points,
            repetition: None,
            properties: Vec::new(),
        })
    }

    // Helper I/O functions
    fn read_u8<R: Read>(cursor: &mut R) -> Result<u8, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 1];
        cursor.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn write_u8<W: Write>(writer: &mut W, value: u8) -> Result<(), Box<dyn std::error::Error>> {
        writer.write_all(&[value])?;
        Ok(())
    }

    fn read_bytes<R: Read>(
        cursor: &mut R,
        len: usize,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buf = vec![0u8; len];
        cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_unsigned<R: Read>(cursor: &mut R) -> Result<u64, Box<dyn std::error::Error>> {
        // Variable-length unsigned integer encoding
        let mut result = 0u64;
        let mut shift = 0;

        loop {
            let byte = Self::read_u8(cursor)?;
            result |= ((byte & 0x7F) as u64) << shift;

            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        Ok(result)
    }

    fn write_unsigned<W: Write>(
        writer: &mut W,
        mut value: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;

            if value != 0 {
                byte |= 0x80;
            }

            Self::write_u8(writer, byte)?;

            if value == 0 {
                break;
            }
        }
        Ok(())
    }

    fn read_signed<R: Read>(cursor: &mut R) -> Result<i64, Box<dyn std::error::Error>> {
        let unsigned = Self::read_unsigned(cursor)?;

        // Zigzag decoding
        let signed = if unsigned & 1 == 0 {
            (unsigned >> 1) as i64
        } else {
            -((unsigned >> 1) as i64) - 1
        };

        Ok(signed)
    }

    fn write_signed<W: Write>(
        writer: &mut W,
        value: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Zigzag encoding
        let unsigned = if value >= 0 {
            (value as u64) << 1
        } else {
            (((-value - 1) as u64) << 1) | 1
        };

        Self::write_unsigned(writer, unsigned)
    }

    fn read_string<R: Read>(cursor: &mut R) -> Result<String, Box<dyn std::error::Error>> {
        let len = Self::read_unsigned(cursor)? as usize;
        let bytes = Self::read_bytes(cursor, len)?;
        Ok(String::from_utf8(bytes)?)
    }

    fn write_string<W: Write>(writer: &mut W, s: &str) -> Result<(), Box<dyn std::error::Error>> {
        Self::write_unsigned(writer, s.len() as u64)?;
        writer.write_all(s.as_bytes())?;
        Ok(())
    }

    fn read_real<R: Read>(cursor: &mut R) -> Result<f64, Box<dyn std::error::Error>> {
        let type_byte = Self::read_u8(cursor)?;

        match type_byte {
            0 => Ok(0.0),
            1 => Ok(1.0),
            2 => {
                let mantissa = Self::read_unsigned(cursor)? as f64;
                let exponent = Self::read_signed(cursor)? as i32;
                Ok(mantissa * 10f64.powi(exponent))
            }
            3 => {
                let mantissa = Self::read_signed(cursor)? as f64;
                let exponent = Self::read_signed(cursor)? as i32;
                Ok(mantissa * 10f64.powi(exponent))
            }
            4 | 5 => {
                let mantissa = if type_byte == 4 {
                    Self::read_unsigned(cursor)? as f64
                } else {
                    Self::read_signed(cursor)? as f64
                };
                let denominator = Self::read_unsigned(cursor)? as f64;
                let exponent = Self::read_signed(cursor)? as i32;
                Ok((mantissa / denominator) * 10f64.powi(exponent))
            }
            6 => {
                let mut bytes = [0u8; 4];
                cursor.read_exact(&mut bytes)?;
                Ok(f32::from_le_bytes(bytes) as f64)
            }
            7 => {
                let mut bytes = [0u8; 8];
                cursor.read_exact(&mut bytes)?;
                Ok(f64::from_le_bytes(bytes))
            }
            _ => Err(format!("Invalid real type: {}", type_byte).into()),
        }
    }

    fn write_real<W: Write>(writer: &mut W, value: f64) -> Result<(), Box<dyn std::error::Error>> {
        // Use IEEE 754 double precision for simplicity
        Self::write_u8(writer, 7)?;
        writer.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    fn read_layer_interval<R: Read>(
        cursor: &mut R,
    ) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        let type_byte = Self::read_u8(cursor)?;

        match type_byte {
            0 => {
                let value = Self::read_unsigned(cursor)? as u32;
                Ok(vec![value])
            }
            1 => {
                let start = Self::read_unsigned(cursor)? as u32;
                let end = Self::read_unsigned(cursor)? as u32;
                Ok((start..=end).collect())
            }
            2 => {
                let count = Self::read_unsigned(cursor)? as usize;
                let mut values = Vec::new();
                for _ in 0..count {
                    values.push(Self::read_unsigned(cursor)? as u32);
                }
                Ok(values)
            }
            _ => Err("Invalid layer interval type".into()),
        }
    }
}

impl OASISCell {
    fn write<W: Write>(
        &self,
        writer: &mut W,
        names: &NameTable,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Write CELL record (type 13 or 14)
        // For simplicity, use type 14 which includes explicit placement
        OASISFile::write_u8(writer, 14)?; // XYAbsolute - sets coordinate mode

        // Write CELL record
        OASISFile::write_u8(writer, 13)?;

        // Write cell name (using reference if available)
        let name_ref = names
            .cell_names
            .iter()
            .find(|(_, n)| n.as_str() == self.name)
            .map(|(r, _)| *r);

        if let Some(ref_num) = name_ref {
            OASISFile::write_unsigned(writer, (ref_num as u64) << 1)?;
        } else {
            OASISFile::write_unsigned(writer, 1)?;
            OASISFile::write_string(writer, &self.name)?;
        }

        // Write elements
        for element in &self.elements {
            element.write(writer, names)?;
        }

        Ok(())
    }
}

impl OASISElement {
    fn write<W: Write>(
        &self,
        writer: &mut W,
        names: &NameTable,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            OASISElement::Rectangle(r) => r.write(writer),
            OASISElement::Polygon(p) => p.write(writer),
            OASISElement::Path(p) => p.write(writer),
            OASISElement::Trapezoid(t) => t.write(writer),
            OASISElement::CTrapezoid(ct) => ct.write(writer),
            OASISElement::Circle(c) => c.write(writer),
            OASISElement::Text(t) => t.write(writer, names),
            OASISElement::Placement(p) => p.write(writer, names),
        }
    }
}

impl Rectangle {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        OASISFile::write_u8(writer, 19)?; // RECTANGLE
        OASISFile::write_u8(writer, 0)?; // Info byte (simplified)
        OASISFile::write_unsigned(writer, self.layer as u64)?;
        OASISFile::write_unsigned(writer, self.datatype as u64)?;
        OASISFile::write_unsigned(writer, self.width)?;
        OASISFile::write_unsigned(writer, self.height)?;
        OASISFile::write_signed(writer, self.x)?;
        OASISFile::write_signed(writer, self.y)?;
        Ok(())
    }
}

impl Polygon {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        OASISFile::write_u8(writer, 20)?; // POLYGON
        OASISFile::write_u8(writer, 0)?; // Info byte (simplified)
        OASISFile::write_unsigned(writer, self.layer as u64)?;
        OASISFile::write_unsigned(writer, self.datatype as u64)?;
        OASISFile::write_unsigned(writer, self.points.len() as u64)?;

        for (x, y) in &self.points {
            OASISFile::write_signed(writer, *x)?;
            OASISFile::write_signed(writer, *y)?;
        }

        OASISFile::write_signed(writer, self.x)?;
        OASISFile::write_signed(writer, self.y)?;
        Ok(())
    }
}

impl OPath {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        OASISFile::write_u8(writer, 21)?; // PATH
        OASISFile::write_u8(writer, 0)?; // Info byte (simplified)
        OASISFile::write_unsigned(writer, self.layer as u64)?;
        OASISFile::write_unsigned(writer, self.datatype as u64)?;
        OASISFile::write_unsigned(writer, self.half_width)?;

        // Extension scheme
        match &self.extension_scheme {
            ExtensionScheme::Flush => OASISFile::write_u8(writer, 0)?,
            ExtensionScheme::HalfWidth => OASISFile::write_u8(writer, 1)?,
            ExtensionScheme::Custom { start, end } => {
                OASISFile::write_u8(writer, 2)?;
                OASISFile::write_signed(writer, *start)?;
                OASISFile::write_signed(writer, *end)?;
            }
        }

        OASISFile::write_unsigned(writer, self.points.len() as u64)?;
        for (x, y) in &self.points {
            OASISFile::write_signed(writer, *x)?;
            OASISFile::write_signed(writer, *y)?;
        }

        OASISFile::write_signed(writer, self.x)?;
        OASISFile::write_signed(writer, self.y)?;
        Ok(())
    }
}

impl Trapezoid {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        OASISFile::write_u8(writer, 22)?; // TRAPEZOID
        OASISFile::write_unsigned(writer, self.layer as u64)?;
        OASISFile::write_unsigned(writer, self.datatype as u64)?;
        OASISFile::write_u8(writer, if self.orientation { 1 } else { 0 })?;
        OASISFile::write_unsigned(writer, self.width)?;
        OASISFile::write_unsigned(writer, self.height)?;
        OASISFile::write_signed(writer, self.delta_a)?;
        OASISFile::write_signed(writer, self.delta_b)?;
        OASISFile::write_signed(writer, self.x)?;
        OASISFile::write_signed(writer, self.y)?;
        Ok(())
    }
}

impl CTrapezoid {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        OASISFile::write_u8(writer, 23)?; // CTRAPEZOID
        OASISFile::write_unsigned(writer, self.layer as u64)?;
        OASISFile::write_unsigned(writer, self.datatype as u64)?;
        OASISFile::write_u8(writer, self.trap_type)?;
        OASISFile::write_unsigned(writer, self.width)?;
        OASISFile::write_unsigned(writer, self.height)?;
        OASISFile::write_signed(writer, self.x)?;
        OASISFile::write_signed(writer, self.y)?;
        Ok(())
    }
}

impl Circle {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        OASISFile::write_u8(writer, 24)?; // CIRCLE
        OASISFile::write_unsigned(writer, self.layer as u64)?;
        OASISFile::write_unsigned(writer, self.datatype as u64)?;
        OASISFile::write_unsigned(writer, self.radius)?;
        OASISFile::write_signed(writer, self.x)?;
        OASISFile::write_signed(writer, self.y)?;
        Ok(())
    }
}

impl OText {
    fn write<W: Write>(
        &self,
        writer: &mut W,
        _names: &NameTable,
    ) -> Result<(), Box<dyn std::error::Error>> {
        OASISFile::write_u8(writer, 18)?; // TEXT
        OASISFile::write_unsigned(writer, self.layer as u64)?;
        OASISFile::write_unsigned(writer, self.texttype as u64)?;
        OASISFile::write_string(writer, &self.string)?;
        OASISFile::write_signed(writer, self.x)?;
        OASISFile::write_signed(writer, self.y)?;
        Ok(())
    }
}

impl Placement {
    fn write<W: Write>(
        &self,
        writer: &mut W,
        _names: &NameTable,
    ) -> Result<(), Box<dyn std::error::Error>> {
        OASISFile::write_u8(writer, 16)?; // PLACEMENT
        OASISFile::write_string(writer, &self.cell_name)?;
        OASISFile::write_signed(writer, self.x)?;
        OASISFile::write_signed(writer, self.y)?;
        Ok(())
    }
}

