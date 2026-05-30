// OASIS (Open Artwork System Interchange Standard) Reader/Writer
// Full implementation of OASIS spec for IC layout interchange
// More compact and modern than GDSII

use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::{decompress_to_vec, decompress_to_vec_zlib};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Write};
use std::path::Path;

/// Reserved PROPNAME/PROPSTRING reference for embedded GDSII library name.
pub const LIBNAME_PROP_REF: u32 = 0xFFFF_FF00;
pub const LIBNAME_PROP_NAME: &str = "LAYKIT_LIBNAME";

/// OASIS File structure
#[derive(Debug, Clone)]
pub struct OASISFile {
    pub version: String,
    /// Database unit in meters (OASIS START record).
    pub unit: f64,
    /// GDSII user unit in meters (preserved for round-trip; not in OASIS START).
    pub user_unit: f64,
    /// OASIS START real = user_unit / database_unit (gdstk scaling factor).
    pub start_scaling: f64,
    /// Multiply OASIS integer coordinates by this (gdstk: 1 / START real).
    pub coord_factor: f64,
    pub offset_flag: bool,
    /// GDSII LIBNAME preserved across GDS↔OASIS conversion (stored in PROPNAME table).
    pub library_name: String,
    /// Raw-deflate CBLOCK level for cell bodies (0 = off, gdstk default ~6).
    pub compression_level: u8,
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
    /// Set when loaded from CELL_REF_NUM before CELLNAME table is complete.
    pub name_ref: Option<u32>,
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
    /// Resolved after load when TEXTSTRING table is populated after TEXT record.
    pub string_ref: Option<u32>,
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
#[derive(Default)]
struct OasisReadModal {
    absolute: bool,
    placement_pos: (i64, i64),
    placement_cell: Option<String>,
    layer: u32,
    datatype: u32,
    geom_dim: (i64, i64),
    geom_pos: (i64, i64),
    text_layer: u32,
    texttype: u32,
    text_string: Option<String>,
    text_pos: (i64, i64),
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
    pub const CELL_REF_NUM: u8 = 13;
    pub const CELL: u8 = 14;
    pub const XYABSOLUTE: u8 = 15;
    pub const XYRELATIVE: u8 = 16;
    pub const PLACEMENT: u8 = 17;
    pub const PLACEMENT_TRANSFORM: u8 = 18;
    pub const TEXT: u8 = 19;
    pub const RECTANGLE: u8 = 20;
    pub const POLYGON: u8 = 21;
    pub const PATH: u8 = 22;
    pub const TRAPEZOID_AB: u8 = 23;
    pub const CTRAPEZOID: u8 = 26;
    pub const CIRCLE: u8 = 27;
}

impl Default for OASISFile {
    fn default() -> Self {
        OASISFile {
            version: "1.0".to_string(),
            unit: 1e-9, // 1nm database unit
            user_unit: 1e-6,
            start_scaling: 1000.0,
            coord_factor: 1.0,
            offset_flag: false,
            library_name: String::new(),
            compression_level: 6,
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
                    oasis.start_scaling = Self::read_real(&mut cursor)?;
                    oasis.offset_flag = Self::read_u8(&mut cursor)? != 0;
                    // gdstk writes start_scaling = user_unit / precision; default micron user unit
                    oasis.user_unit = 1e-6;
                    oasis.unit = if oasis.start_scaling > 0.0 {
                        oasis.user_unit / oasis.start_scaling
                    } else {
                        1e-9
                    };
                    oasis.coord_factor = if oasis.start_scaling > 0.0 {
                        1.0 / oasis.start_scaling
                    } else {
                        1.0
                    };
                }
                2 => {
                    // END
                    // Validation table follows - skip it
                    // Read validation signature (may fail if at end of file)
                    let _ = Self::read_unsigned(&mut cursor);
                    break;
                }
                3 => {
                    // CELLNAME_IMPLICIT — append to name table
                    let name = Self::read_string(&mut cursor)?;
                    let ref_num = oasis.names.cell_names.len() as u32;
                    oasis.names.cell_names.insert(ref_num, name);
                }
                4 => {
                    // CELLNAME — assign at reference number
                    let name = Self::read_string(&mut cursor)?;
                    let ref_num = Self::read_unsigned(&mut cursor)? as u32;
                    oasis.names.cell_names.insert(ref_num, name);
                }
                5 | 6 => {
                    // TEXTSTRING (explicit and implicit)
                    let string = Self::read_string(&mut cursor)?;
                    let ref_num = Self::read_unsigned(&mut cursor)? as u32;
                    oasis.names.text_strings.insert(ref_num, string);
                }
                7 | 8 => {
                    // PROPNAME (explicit and implicit)
                    let name = Self::read_string(&mut cursor)?;
                    let ref_num = Self::read_unsigned(&mut cursor)? as u32;
                    oasis.names.prop_names.insert(ref_num, name);
                }
                9 | 10 => {
                    // PROPSTRING (explicit and implicit)
                    let string = Self::read_string(&mut cursor)?;
                    let ref_num = Self::read_unsigned(&mut cursor)? as u32;
                    oasis.names.prop_strings.insert(ref_num, string);
                }
                11 | 12 => {
                    // LAYERNAME (explicit and implicit)
                    let name = Self::read_string(&mut cursor)?;
                    let layer_interval = Self::read_layer_interval(&mut cursor)?;
                    let _datatype_interval = Self::read_layer_interval(&mut cursor)?;
                    // Store first value from intervals
                    if let Some(layer) = layer_interval.first() {
                        oasis.names.layer_names.insert(*layer, name);
                    }
                }
                13 => {
                    let cell =
                        Self::read_cell_ref_num(&mut cursor, &mut oasis.names, oasis.coord_factor)?;
                    oasis.cells.push(cell);
                }
                14 => {
                    let cell =
                        Self::read_cell_named(&mut cursor, &mut oasis.names, oasis.coord_factor)?;
                    oasis.cells.push(cell);
                }
                15 => { /* XYABSOLUTE — handled in cell body */ }
                16 => { /* XYRELATIVE — handled in cell body */ }
                _ => {
                    // Skip unknown records
                }
            }
        }

        if oasis.library_name.is_empty()
            && oasis.names.prop_names.get(&LIBNAME_PROP_REF) == Some(&LIBNAME_PROP_NAME.to_string())
        {
            if let Some(name) = oasis.names.prop_strings.get(&LIBNAME_PROP_REF) {
                oasis.library_name = name.clone();
            }
        }

        Self::resolve_deferred_cell_names(&mut oasis);
        Self::resolve_deferred_text_strings(&mut oasis);

        Ok(oasis)
    }

    fn resolve_deferred_text_strings(oasis: &mut OASISFile) {
        for cell in &mut oasis.cells {
            for element in &mut cell.elements {
                if let OASISElement::Text(t) = element {
                    if t.string.is_empty() {
                        if let Some(ref_num) = t.string_ref {
                            if let Some(s) = oasis.names.text_strings.get(&ref_num) {
                                t.string = s.clone();
                            }
                        }
                    }
                    t.string_ref = None;
                }
            }
        }
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
        let scaling = if self.start_scaling > 0.0 {
            self.start_scaling
        } else if self.unit > 0.0 {
            self.user_unit / self.unit
        } else {
            1000.0
        };
        Self::write_real(writer, scaling)?;
        Self::write_u8(writer, if self.offset_flag { 1 } else { 0 })?;

        let mut prop_names = self.names.prop_names.clone();
        let mut prop_strings = self.names.prop_strings.clone();
        if !self.library_name.is_empty() {
            prop_names.insert(LIBNAME_PROP_REF, LIBNAME_PROP_NAME.to_string());
            prop_strings.insert(LIBNAME_PROP_REF, self.library_name.clone());
        }

        // Write name tables (ensure every cell has a name reference)
        let mut names = self.names.clone();
        for cell in &self.cells {
            if !names.cell_names.values().any(|n| n == &cell.name) {
                let id = names.cell_names.len() as u32;
                names.cell_names.insert(id, cell.name.clone());
            }
        }

        let mut cell_name_entries: Vec<_> = names.cell_names.iter().collect();
        cell_name_entries.sort_by_key(|(k, _)| *k);
        for (_, name) in cell_name_entries {
            Self::write_u8(writer, 3)?; // CELLNAME_IMPLICIT
            Self::write_string(writer, name)?;
        }

        for cell in &self.cells {
            for element in &cell.elements {
                if let OASISElement::Text(t) = element {
                    if !names.text_strings.values().any(|s| s == &t.string) {
                        let id = names.text_strings.len() as u32;
                        names.text_strings.insert(id, t.string.clone());
                    }
                }
            }
        }

        for (ref_num, string) in &names.text_strings {
            Self::write_u8(writer, 5)?; // TEXTSTRING
            Self::write_string(writer, string)?;
            Self::write_unsigned(writer, *ref_num as u64)?;
        }

        for (ref_num, name) in &prop_names {
            Self::write_u8(writer, 7)?; // PROPNAME
            Self::write_string(writer, name)?;
            Self::write_unsigned(writer, *ref_num as u64)?;
        }

        for (ref_num, string) in &prop_strings {
            Self::write_u8(writer, 9)?; // PROPSTRING
            Self::write_string(writer, string)?;
            Self::write_unsigned(writer, *ref_num as u64)?;
        }

        let scaling = if self.start_scaling > 0.0 {
            self.start_scaling
        } else if self.unit > 0.0 {
            self.user_unit / self.unit
        } else {
            1000.0
        };

        // Write cells
        for cell in &self.cells {
            cell.write(writer, &names, scaling, self.compression_level)?;
        }

        // Write END record
        Self::write_u8(writer, 2)?;

        // Write validation (optional, simplified)
        Self::write_unsigned(writer, 0)?; // No validation

        Ok(())
    }

    fn resolve_deferred_cell_names(oasis: &mut OASISFile) {
        for cell in &mut oasis.cells {
            if let Some(ref_num) = cell.name_ref {
                if let Some(name) = oasis.names.cell_names.get(&ref_num) {
                    cell.name = name.clone();
                } else if cell.name.is_empty() {
                    cell.name = format!("CELL_{ref_num}");
                }
                cell.name_ref = None;
            }
        }
    }

    fn read_cell_ref_num(
        cursor: &mut Cursor<Vec<u8>>,
        names: &mut NameTable,
        factor: f64,
    ) -> Result<OASISCell, Box<dyn std::error::Error>> {
        let ref_num = Self::read_unsigned(cursor)? as u32;
        let mut cell = OASISCell {
            name: String::new(),
            name_ref: Some(ref_num),
            elements: Vec::new(),
        };
        let mut modal = OasisReadModal::default();
        modal.absolute = true;
        Self::read_cell_body(cursor, names, &mut modal, factor, &mut cell)?;
        Ok(cell)
    }

    fn read_cell_named(
        cursor: &mut Cursor<Vec<u8>>,
        names: &mut NameTable,
        factor: f64,
    ) -> Result<OASISCell, Box<dyn std::error::Error>> {
        let name = Self::read_string(cursor)?;
        let mut cell = OASISCell {
            name,
            name_ref: None,
            elements: Vec::new(),
        };
        let mut modal = OasisReadModal::default();
        modal.absolute = true;
        Self::read_cell_body(cursor, names, &mut modal, factor, &mut cell)?;
        Ok(cell)
    }

    fn scale_coord(factor: f64, v: i64) -> i64 {
        (factor * v as f64).round() as i64
    }

    fn scale_coord_u(factor: f64, v: u64) -> i64 {
        (factor * v as f64).round() as i64
    }

    fn read_repetition(
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Option<Repetition>, Box<dyn std::error::Error>> {
        let rep_type = Self::read_u8(cursor)?;
        if rep_type == 0 {
            return Ok(None);
        }
        match rep_type {
            1 => Ok(Some(Repetition::Matrix {
                x_count: (2 + Self::read_unsigned(cursor)?) as u32,
                y_count: (2 + Self::read_unsigned(cursor)?) as u32,
                x_space: Self::read_unsigned(cursor)?,
                y_space: Self::read_unsigned(cursor)?,
            })),
            2 => Ok(Some(Repetition::Matrix {
                x_count: (2 + Self::read_unsigned(cursor)?) as u32,
                y_count: 1,
                x_space: Self::read_unsigned(cursor)?,
                y_space: 0,
            })),
            3 => Ok(Some(Repetition::Matrix {
                x_count: 1,
                y_count: (2 + Self::read_unsigned(cursor)?) as u32,
                x_space: 0,
                y_space: Self::read_unsigned(cursor)?,
            })),
            _ => Ok(None),
        }
    }

    fn write_repetition<W: Write>(
        writer: &mut W,
        repetition: &Repetition,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match repetition {
            Repetition::Matrix {
                x_count,
                y_count,
                x_space,
                y_space,
            } if *x_count > 1 && *y_count > 1 => {
                Self::write_u8(writer, 1)?;
                Self::write_unsigned(writer, (*x_count - 2) as u64)?;
                Self::write_unsigned(writer, (*y_count - 2) as u64)?;
                Self::write_unsigned(writer, *x_space)?;
                Self::write_unsigned(writer, *y_space)?;
            }
            Repetition::Matrix {
                x_count,
                y_count: 1,
                x_space,
                y_space: 0,
            } if *x_count > 1 => {
                Self::write_u8(writer, 2)?;
                Self::write_unsigned(writer, (*x_count - 2) as u64)?;
                Self::write_unsigned(writer, *x_space)?;
            }
            Repetition::Matrix {
                x_count: 1,
                y_count,
                x_space: 0,
                y_space,
            } if *y_count > 1 => {
                Self::write_u8(writer, 3)?;
                Self::write_unsigned(writer, (*y_count - 2) as u64)?;
                Self::write_unsigned(writer, *y_space)?;
            }
            _ => Self::write_u8(writer, 0)?,
        }
        Ok(())
    }

    fn read_cell_body(
        cursor: &mut Cursor<Vec<u8>>,
        names: &mut NameTable,
        modal: &mut OasisReadModal,
        factor: f64,
        cell: &mut OASISCell,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let position = cursor.position();
            let record_id = match Self::read_u8(cursor) {
                Ok(id) => id,
                Err(_) => break,
            };

            if record_id == 0 {
                continue;
            }

            match record_id {
                2 | 13 | 14 => {
                    cursor.set_position(position);
                    break;
                }
                3 => {
                    let name = Self::read_string(cursor)?;
                    let ref_num = names.cell_names.len() as u32;
                    names.cell_names.insert(ref_num, name);
                }
                4 => {
                    let name = Self::read_string(cursor)?;
                    let ref_num = Self::read_unsigned(cursor)? as u32;
                    names.cell_names.insert(ref_num, name);
                }
                15 => modal.absolute = true,
                16 => modal.absolute = false,
                17 => {
                    if let Ok(elem) = Self::read_placement(cursor, names, modal, false) {
                        cell.elements.push(OASISElement::Placement(elem));
                    }
                }
                18 => {
                    if let Ok(elem) = Self::read_placement(cursor, names, modal, true) {
                        cell.elements.push(OASISElement::Placement(elem));
                    }
                }
                19 => {
                    if let Ok(elem) = Self::read_text(cursor, names, modal, factor) {
                        cell.elements.push(OASISElement::Text(elem));
                    }
                }
                20 => {
                    if let Ok(elem) = Self::read_rectangle(cursor, modal, factor) {
                        cell.elements.push(OASISElement::Rectangle(elem));
                    }
                }
                21 => {
                    if let Ok(elem) = Self::read_polygon(cursor) {
                        cell.elements.push(OASISElement::Polygon(elem));
                    }
                }
                22 => {
                    if let Ok(elem) = Self::read_path(cursor) {
                        cell.elements.push(OASISElement::Path(elem));
                    }
                }
                23 | 24 | 25 => {
                    if let Ok(elem) = Self::read_trapezoid(cursor) {
                        cell.elements.push(OASISElement::Trapezoid(elem));
                    }
                }
                26 => {
                    if let Ok(elem) = Self::read_ctrapezoid(cursor) {
                        cell.elements.push(OASISElement::CTrapezoid(elem));
                    }
                }
                27 => {
                    if let Ok(elem) = Self::read_circle(cursor) {
                        cell.elements.push(OASISElement::Circle(elem));
                    }
                }
                28 | 29 => {
                    if let Ok(prop) = Self::read_property_record(cursor, names) {
                        Self::attach_property(cell, prop);
                    }
                }
                30 | 31 => {
                    let _ = Self::read_unsigned(cursor);
                    let _ = Self::read_string(cursor);
                }
                32 | 33 | 34 => {
                    // XELEMENT / XGEOMETRY / CBLOCK (gdstk writes geometry in 34)
                    if let Ok(chunk) = Self::read_cblock(cursor) {
                        let mut sub = Cursor::new(chunk);
                        let mut sub_modal = OasisReadModal::default();
                        sub_modal.absolute = modal.absolute;
                        Self::read_cell_body(&mut sub, names, &mut sub_modal, factor, cell)?;
                    }
                }
                _ => {
                    cursor.set_position(position);
                    break;
                }
            }
        }
        Ok(())
    }

    fn write_cblock<W: Write>(
        writer: &mut W,
        uncompressed: &[u8],
        level: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let compressed = compress_to_vec(uncompressed, level.min(9));
        Self::write_u8(writer, 34)?; // CBLOCK
        Self::write_u8(writer, 0)?; // raw deflate
        Self::write_unsigned(writer, uncompressed.len() as u64)?;
        Self::write_unsigned(writer, compressed.len() as u64)?;
        writer.write_all(&compressed)?;
        Ok(())
    }

    fn read_cblock(cursor: &mut Cursor<Vec<u8>>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let comp_type = Self::read_unsigned(cursor)?;
        let uncomp_byte_count = Self::read_unsigned(cursor)?;
        let comp_byte_count = Self::read_unsigned(cursor)?;
        let compressed = Self::read_bytes(cursor, comp_byte_count as usize)?;
        match comp_type {
            0 => {
                // OASIS / gdstk use raw deflate (no zlib header) for type 0
                let out = decompress_to_vec(&compressed)
                    .or_else(|_| decompress_to_vec_zlib(&compressed))
                    .map_err(|e| format!("CBLOCK decompress failed: {e}"))?;
                if uncomp_byte_count > 0 && out.len() != uncomp_byte_count as usize {
                    return Err(format!(
                        "CBLOCK size mismatch: expected {uncomp_byte_count}, got {}",
                        out.len()
                    )
                    .into());
                }
                Ok(out)
            }
            1 => Ok(compressed),
            other => Err(format!("unsupported CBLOCK compression type {other}").into()),
        }
    }

    fn read_property_record(
        cursor: &mut Cursor<Vec<u8>>,
        names: &NameTable,
    ) -> Result<Property, Box<dyn std::error::Error>> {
        let name_ref = Self::read_unsigned(cursor)?;
        let name = if name_ref & 1 == 0 {
            let ref_num = (name_ref >> 1) as u32;
            names
                .prop_names
                .get(&ref_num)
                .cloned()
                .unwrap_or_else(|| format!("PROP_{ref_num}"))
        } else {
            Self::read_string(cursor)?
        };
        let values_count = Self::read_unsigned(cursor)? as usize;
        let mut values = Vec::with_capacity(values_count);
        for _ in 0..values_count {
            let value_ref = Self::read_unsigned(cursor)?;
            values.push(Self::decode_property_value(value_ref, cursor, names)?);
        }
        Ok(Property { name, values })
    }

    fn decode_property_value(
        value_ref: u64,
        cursor: &mut Cursor<Vec<u8>>,
        _names: &NameTable,
    ) -> Result<PropertyValue, Box<dyn std::error::Error>> {
        match value_ref & 0x0F {
            0 | 1 => {
                if value_ref & 1 == 0 {
                    Ok(PropertyValue::Integer((value_ref >> 1) as i64))
                } else {
                    Ok(PropertyValue::String(Self::read_string(cursor)?))
                }
            }
            2 => Ok(PropertyValue::Real(f64::from_bits(value_ref >> 4))),
            3 => Ok(PropertyValue::Reference((value_ref >> 1) as u32)),
            4 => Ok(PropertyValue::Boolean(value_ref != 0)),
            _ => Ok(PropertyValue::Integer(value_ref as i64)),
        }
    }

    fn attach_property(cell: &mut OASISCell, prop: Property) {
        if let Some(last) = cell.elements.last_mut() {
            match last {
                OASISElement::Rectangle(r) => r.properties.push(prop),
                OASISElement::Polygon(p) => p.properties.push(prop),
                OASISElement::Path(p) => p.properties.push(prop),
                OASISElement::Trapezoid(t) => t.properties.push(prop),
                OASISElement::CTrapezoid(t) => t.properties.push(prop),
                OASISElement::Circle(c) => c.properties.push(prop),
                OASISElement::Text(t) => t.properties.push(prop),
                OASISElement::Placement(p) => p.properties.push(prop),
            }
        }
    }

    fn read_rectangle(
        cursor: &mut Cursor<Vec<u8>>,
        modal: &mut OasisReadModal,
        factor: f64,
    ) -> Result<Rectangle, Box<dyn std::error::Error>> {
        let info = Self::read_u8(cursor)?;
        if info & 0x01 != 0 {
            modal.layer = Self::read_unsigned(cursor)? as u32;
        }
        if info & 0x02 != 0 {
            modal.datatype = Self::read_unsigned(cursor)? as u32;
        }
        if info & 0x40 != 0 {
            modal.geom_dim.0 = Self::scale_coord_u(factor, Self::read_unsigned(cursor)?);
        }
        if info & 0x20 != 0 {
            modal.geom_dim.1 = Self::scale_coord_u(factor, Self::read_unsigned(cursor)?);
        } else if info & 0x80 != 0 {
            modal.geom_dim.1 = modal.geom_dim.0;
        }
        if info & 0x10 != 0 {
            let x = Self::scale_coord(factor, Self::read_signed(cursor)?);
            modal.geom_pos.0 = if modal.absolute {
                x
            } else {
                modal.geom_pos.0 + x
            };
        }
        if info & 0x08 != 0 {
            let y = Self::scale_coord(factor, Self::read_signed(cursor)?);
            modal.geom_pos.1 = if modal.absolute {
                y
            } else {
                modal.geom_pos.1 + y
            };
        }
        let repetition = if info & 0x04 != 0 {
            Self::read_repetition(cursor)?
        } else {
            None
        };

        Ok(Rectangle {
            layer: modal.layer,
            datatype: modal.datatype,
            x: modal.geom_pos.0,
            y: modal.geom_pos.1,
            width: modal.geom_dim.0.max(0) as u64,
            height: modal.geom_dim.1.max(0) as u64,
            repetition,
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

    fn read_trapezoid(
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Trapezoid, Box<dyn std::error::Error>> {
        let layer = Self::read_unsigned(cursor)? as u32;
        let datatype = Self::read_unsigned(cursor)? as u32;
        let orientation = Self::read_u8(cursor)? != 0;
        let width = Self::read_unsigned(cursor)?;
        let height = Self::read_unsigned(cursor)?;
        let delta_a = Self::read_signed(cursor)?;
        let delta_b = Self::read_signed(cursor)?;
        let x = Self::read_signed(cursor)?;
        let y = Self::read_signed(cursor)?;

        Ok(Trapezoid {
            layer,
            datatype,
            orientation,
            width,
            height,
            delta_a,
            delta_b,
            x,
            y,
            repetition: None,
            properties: Vec::new(),
        })
    }

    fn read_ctrapezoid(
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<CTrapezoid, Box<dyn std::error::Error>> {
        let layer = Self::read_unsigned(cursor)? as u32;
        let datatype = Self::read_unsigned(cursor)? as u32;
        let trap_type = Self::read_u8(cursor)?;
        let width = Self::read_unsigned(cursor)?;
        let height = Self::read_unsigned(cursor)?;
        let x = Self::read_signed(cursor)?;
        let y = Self::read_signed(cursor)?;

        Ok(CTrapezoid {
            layer,
            datatype,
            trap_type,
            width,
            height,
            x,
            y,
            repetition: None,
            properties: Vec::new(),
        })
    }

    fn read_circle(cursor: &mut Cursor<Vec<u8>>) -> Result<Circle, Box<dyn std::error::Error>> {
        let layer = Self::read_unsigned(cursor)? as u32;
        let datatype = Self::read_unsigned(cursor)? as u32;
        let radius = Self::read_unsigned(cursor)?;
        let x = Self::read_signed(cursor)?;
        let y = Self::read_signed(cursor)?;

        Ok(Circle {
            layer,
            datatype,
            radius,
            x,
            y,
            repetition: None,
            properties: Vec::new(),
        })
    }

    fn read_text(
        cursor: &mut Cursor<Vec<u8>>,
        names: &NameTable,
        modal: &mut OasisReadModal,
        factor: f64,
    ) -> Result<OText, Box<dyn std::error::Error>> {
        let info = Self::read_u8(cursor)?;
        let (string, string_ref) = if info & 0x40 != 0 {
            if info & 0x20 != 0 {
                let ref_num = Self::read_unsigned(cursor)? as u32;
                let s = names.text_strings.get(&ref_num).cloned().unwrap_or_default();
                (s, if names.text_strings.contains_key(&ref_num) {
                    None
                } else {
                    Some(ref_num)
                })
            } else {
                (Self::read_string(cursor)?, None)
            }
        } else {
            (
                modal
                    .text_string
                    .clone()
                    .ok_or("modal text string undefined")?,
                None,
            )
        };
        if !string.is_empty() {
            modal.text_string = Some(string.clone());
        }

        if info & 0x01 != 0 {
            modal.text_layer = Self::read_unsigned(cursor)? as u32;
        }
        if info & 0x02 != 0 {
            modal.texttype = Self::read_unsigned(cursor)? as u32;
        }
        if info & 0x10 != 0 {
            let x = Self::scale_coord(factor, Self::read_signed(cursor)?);
            modal.text_pos.0 = if modal.absolute {
                x
            } else {
                modal.text_pos.0 + x
            };
        }
        if info & 0x08 != 0 {
            let y = Self::scale_coord(factor, Self::read_signed(cursor)?);
            modal.text_pos.1 = if modal.absolute {
                y
            } else {
                modal.text_pos.1 + y
            };
        }
        let repetition = if info & 0x04 != 0 {
            Self::read_repetition(cursor)?
        } else {
            None
        };

        Ok(OText {
            layer: modal.text_layer,
            texttype: modal.texttype,
            string,
            string_ref,
            x: modal.text_pos.0,
            y: modal.text_pos.1,
            repetition,
            properties: Vec::new(),
        })
    }

    fn read_placement(
        cursor: &mut Cursor<Vec<u8>>,
        names: &NameTable,
        modal: &mut OasisReadModal,
        transform_record: bool,
    ) -> Result<Placement, Box<dyn std::error::Error>> {
        let info = Self::read_u8(cursor)?;

        let cell_name = if info & 0x80 != 0 {
            if info & 0x40 != 0 {
                let ref_num = Self::read_unsigned(cursor)? as u32;
                names
                    .cell_names
                    .get(&ref_num)
                    .cloned()
                    .ok_or_else(|| format!("cell name reference {ref_num} not found"))?
            } else {
                Self::read_string(cursor)?
            }
        } else {
            modal
                .placement_cell
                .clone()
                .ok_or("modal placement cell undefined")?
        };
        modal.placement_cell = Some(cell_name.clone());

        let mut magnification = 1.0f64;
        let mut angle = 0.0f64;
        if transform_record {
            if info & 0x04 != 0 {
                magnification = Self::read_real(cursor)?;
            }
            if info & 0x02 != 0 {
                angle = Self::read_real(cursor)? * (std::f64::consts::PI / 180.0);
            }
        } else {
            match info & 0x06 {
                0x02 => angle = std::f64::consts::FRAC_PI_2,
                0x04 => angle = std::f64::consts::PI,
                0x06 => angle = 3.0 * std::f64::consts::FRAC_PI_2,
                _ => {}
            }
        }
        let mirror = info & 0x01 != 0;

        if info & 0x20 != 0 {
            let x = Self::read_signed(cursor)?;
            modal.placement_pos.0 = if modal.absolute {
                x
            } else {
                modal.placement_pos.0 + x
            };
        }
        if info & 0x10 != 0 {
            let y = Self::read_signed(cursor)?;
            modal.placement_pos.1 = if modal.absolute {
                y
            } else {
                modal.placement_pos.1 + y
            };
        }

        let repetition = if info & 0x08 != 0 {
            Self::read_repetition(cursor)?
        } else {
            None
        };

        Ok(Placement {
            cell_name,
            x: modal.placement_pos.0,
            y: modal.placement_pos.1,
            magnification: if (magnification - 1.0).abs() > 1e-12 {
                Some(magnification)
            } else {
                None
            },
            angle: if angle.abs() > 1e-12 { Some(angle) } else { None },
            mirror,
            repetition,
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
        // Use from_utf8_lossy to handle non-UTF8 strings gracefully
        // OASIS spec uses ASCII, but some tools may write non-UTF8 data
        Ok(String::from_utf8_lossy(&bytes).into_owned())
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
            _ => Err(format!(
                "Invalid real type: {} (may indicate corrupted file or misaligned read)",
                type_byte
            )
            .into()),
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
        scaling: f64,
        compression_level: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        OASISFile::write_u8(writer, 15)?; // XYABSOLUTE
        OASISFile::write_u8(writer, 13)?; // CELL_REF_NUM

        let name_ref = names
            .cell_names
            .iter()
            .find(|(_, n)| n.as_str() == self.name)
            .map(|(r, _)| *r)
            .unwrap_or(0);
        OASISFile::write_unsigned(writer, name_ref as u64)?;

        let mut body = Vec::new();
        {
            let mut buf = std::io::Cursor::new(&mut body);
            for element in &self.elements {
                element.write(&mut buf, names, scaling)?;
            }
        }

        if compression_level > 0 && !body.is_empty() {
            OASISFile::write_cblock(writer, &body, compression_level)?;
        } else {
            writer.write_all(&body)?;
        }

        Ok(())
    }
}

impl OASISElement {
    fn write<W: Write>(
        &self,
        writer: &mut W,
        names: &NameTable,
        scaling: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            OASISElement::Rectangle(r) => r.write(writer, scaling),
            OASISElement::Polygon(p) => p.write(writer),
            OASISElement::Path(p) => p.write(writer),
            OASISElement::Trapezoid(t) => t.write(writer),
            OASISElement::CTrapezoid(ct) => ct.write(writer),
            OASISElement::Circle(c) => c.write(writer),
            OASISElement::Text(t) => t.write(writer, names, scaling),
            OASISElement::Placement(p) => p.write(writer, names),
        }
    }
}

impl Rectangle {
    fn write<W: Write>(
        &self,
        writer: &mut W,
        scaling: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let w = (self.width as f64 * scaling).round() as u64;
        let h = (self.height as f64 * scaling).round() as u64;
        let x = (self.x as f64 * scaling).round() as i64;
        let y = (self.y as f64 * scaling).round() as i64;
        let square = w == h;
        let mut info: u8 = if square { 0xDB } else { 0x7B };
        if self.repetition.is_some() {
            info |= 0x04;
        }
        OASISFile::write_u8(writer, 20)?; // RECTANGLE
        OASISFile::write_u8(writer, info)?;
        OASISFile::write_unsigned(writer, self.layer as u64)?;
        OASISFile::write_unsigned(writer, self.datatype as u64)?;
        OASISFile::write_unsigned(writer, w)?;
        if !square {
            OASISFile::write_unsigned(writer, h)?;
        }
        OASISFile::write_signed(writer, x)?;
        OASISFile::write_signed(writer, y)?;
        if let Some(rep) = &self.repetition {
            OASISFile::write_repetition(writer, rep)?;
        }
        Ok(())
    }
}

impl Polygon {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        OASISFile::write_u8(writer, 21)?; // POLYGON
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
        OASISFile::write_u8(writer, 22)?; // PATH
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
        OASISFile::write_u8(writer, 23)?; // TRAPEZOID_AB
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
        OASISFile::write_u8(writer, 26)?; // CTRAPEZOID
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
        OASISFile::write_u8(writer, 27)?; // CIRCLE
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
        names: &NameTable,
        scaling: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text_ref = names
            .text_strings
            .iter()
            .find(|(_, s)| s.as_str() == self.string)
            .map(|(r, _)| *r)
            .unwrap_or(0);
        let mut info: u8 = 0x7B; // explicit text ref + layer + texttype + x + y (gdstk)
        if self.repetition.is_some() {
            info |= 0x04;
        }
        let x = (self.x as f64 * scaling).round() as i64;
        let y = (self.y as f64 * scaling).round() as i64;
        OASISFile::write_u8(writer, 19)?; // TEXT
        OASISFile::write_u8(writer, info)?;
        OASISFile::write_unsigned(writer, text_ref as u64)?;
        OASISFile::write_unsigned(writer, self.layer as u64)?;
        OASISFile::write_unsigned(writer, self.texttype as u64)?;
        OASISFile::write_signed(writer, x)?;
        OASISFile::write_signed(writer, y)?;
        if let Some(rep) = &self.repetition {
            OASISFile::write_repetition(writer, rep)?;
        }
        Ok(())
    }
}

impl Placement {
    fn write<W: Write>(
        &self,
        writer: &mut W,
        names: &NameTable,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cell_ref = names
            .cell_names
            .iter()
            .find(|(_, n)| n.as_str() == self.cell_name)
            .map(|(r, _)| *r)
            .unwrap_or(0);

        let has_repetition = self.repetition.as_ref().is_some_and(|r| {
            matches!(
                r,
                Repetition::Matrix {
                    x_count,
                    y_count,
                    ..
                } if *x_count > 1 || *y_count > 1
            )
        });

        let mut info: u8 = 0xF0 | 0x40 | 0x80; // explicit cell ref number
        if has_repetition {
            info |= 0x08;
        }
        if self.mirror {
            info |= 0x01;
        }

        OASISFile::write_u8(writer, 17)?; // PLACEMENT
        OASISFile::write_u8(writer, info)?;
        OASISFile::write_unsigned(writer, cell_ref as u64)?;
        OASISFile::write_signed(writer, self.x)?;
        OASISFile::write_signed(writer, self.y)?;
        if let Some(ref rep) = self.repetition {
            OASISFile::write_repetition(writer, rep)?;
        }
        Ok(())
    }
}
