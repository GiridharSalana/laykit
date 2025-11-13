// Conversion utilities between OASIS and GDSII formats

use crate::gdsii::{
    Boundary, GDSElement, GDSIIFile, GDSStructure, GDSTime, GPath, GText, StructRef,
};
use crate::oasis::{
    OASISCell, OASISElement, OASISFile, OPath, OText, Placement, Polygon, Rectangle,
};

/// Convert GDSII to OASIS
pub fn gdsii_to_oasis(gds: &GDSIIFile) -> Result<OASISFile, Box<dyn std::error::Error>> {
    let mut oasis = OASISFile::new();

    // Convert units (GDSII uses database units in meters, OASIS uses unit multiplier)
    oasis.unit = gds.units.1; // database unit in meters
    oasis.version = "1.0".to_string();

    // Convert structures to cells
    for (idx, structure) in gds.structures.iter().enumerate() {
        let cell_ref = idx as u32;
        oasis
            .names
            .cell_names
            .insert(cell_ref, structure.name.clone());

        let mut cell = OASISCell {
            name: structure.name.clone(),
            elements: Vec::new(),
        };

        // Convert elements
        for element in &structure.elements {
            if let Some(oasis_elem) = convert_gds_element_to_oasis(element) {
                cell.elements.push(oasis_elem);
            }
        }

        oasis.cells.push(cell);
    }

    Ok(oasis)
}

/// Convert OASIS to GDSII
pub fn oasis_to_gdsii(oasis: &OASISFile) -> Result<GDSIIFile, Box<dyn std::error::Error>> {
    let mut gds = GDSIIFile::new("CONVERTED".to_string());

    // Convert units
    gds.units = (1e-6, oasis.unit); // 1 micron user unit, OASIS unit as database unit

    // Convert cells to structures
    for cell in &oasis.cells {
        let mut structure = GDSStructure {
            name: cell.name.clone(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            elements: Vec::new(),
        };

        // Convert elements
        for element in &cell.elements {
            if let Some(gds_elem) = convert_oasis_element_to_gds(element) {
                structure.elements.push(gds_elem);
            }
        }

        gds.structures.push(structure);
    }

    Ok(gds)
}

fn convert_gds_element_to_oasis(element: &GDSElement) -> Option<OASISElement> {
    match element {
        GDSElement::Boundary(boundary) => {
            // GDSII Boundary can be converted to OASIS Polygon
            if boundary.xy.len() < 3 {
                return None;
            }

            // Check if it's a rectangle (4 or 5 points with right angles)
            if is_rectangle(&boundary.xy) {
                let (x_min, y_min, width, height) = calculate_rectangle_bounds(&boundary.xy);
                Some(OASISElement::Rectangle(Rectangle {
                    layer: boundary.layer as u32,
                    datatype: boundary.datatype as u32,
                    x: x_min as i64,
                    y: y_min as i64,
                    width: width as u64,
                    height: height as u64,
                    repetition: None,
                    properties: Vec::new(),
                }))
            } else {
                // Convert to polygon
                let points: Vec<(i64, i64)> = boundary
                    .xy
                    .iter()
                    .map(|(x, y)| (*x as i64, *y as i64))
                    .collect();

                if points.is_empty() {
                    return None;
                }

                let x = points[0].0;
                let y = points[0].1;
                let relative_points: Vec<(i64, i64)> =
                    points.iter().map(|(px, py)| (*px - x, *py - y)).collect();

                Some(OASISElement::Polygon(Polygon {
                    layer: boundary.layer as u32,
                    datatype: boundary.datatype as u32,
                    x,
                    y,
                    points: relative_points,
                    repetition: None,
                    properties: Vec::new(),
                }))
            }
        }
        GDSElement::Path(path) => {
            let points: Vec<(i64, i64)> = path
                .xy
                .iter()
                .map(|(x, y)| (*x as i64, *y as i64))
                .collect();

            if points.is_empty() {
                return None;
            }

            let x = points[0].0;
            let y = points[0].1;
            let relative_points: Vec<(i64, i64)> =
                points.iter().map(|(px, py)| (*px - x, *py - y)).collect();

            Some(OASISElement::Path(OPath {
                layer: path.layer as u32,
                datatype: path.datatype as u32,
                x,
                y,
                half_width: path.width.unwrap_or(0) as u64 / 2,
                extension_scheme: crate::oasis::ExtensionScheme::Flush,
                points: relative_points,
                repetition: None,
                properties: Vec::new(),
            }))
        }
        GDSElement::Text(text) => Some(OASISElement::Text(OText {
            layer: text.layer as u32,
            texttype: text.texttype as u32,
            x: text.xy.0 as i64,
            y: text.xy.1 as i64,
            string: text.string.clone(),
            repetition: None,
            properties: Vec::new(),
        })),
        GDSElement::StructRef(sref) => Some(OASISElement::Placement(Placement {
            cell_name: sref.sname.clone(),
            x: sref.xy.0 as i64,
            y: sref.xy.1 as i64,
            magnification: sref.strans.as_ref().and_then(|st| st.magnification),
            angle: sref.strans.as_ref().and_then(|st| st.angle),
            mirror: sref
                .strans
                .as_ref()
                .map(|st| st.reflection)
                .unwrap_or(false),
            repetition: None,
            properties: Vec::new(),
        })),
        GDSElement::ArrayRef(_aref) => {
            // Array references would need special handling
            // For now, skip or implement as multiple placements
            None
        }
        GDSElement::Node(_node) => {
            // Nodes don't have a direct OASIS equivalent
            None
        }
        GDSElement::Box(_box) => {
            // Boxes can be treated as boundaries/polygons
            None
        }
    }
}

fn convert_oasis_element_to_gds(element: &OASISElement) -> Option<GDSElement> {
    match element {
        OASISElement::Rectangle(rect) => {
            let x = rect.x as i32;
            let y = rect.y as i32;
            let w = rect.width as i32;
            let h = rect.height as i32;

            Some(GDSElement::Boundary(Boundary {
                layer: rect.layer as i16,
                datatype: rect.datatype as i16,
                xy: vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x, y)],
                properties: Vec::new(),
            }))
        }
        OASISElement::Polygon(poly) => {
            let xy: Vec<(i32, i32)> = poly
                .points
                .iter()
                .map(|(px, py)| ((poly.x + px) as i32, (poly.y + py) as i32))
                .collect();

            Some(GDSElement::Boundary(Boundary {
                layer: poly.layer as i16,
                datatype: poly.datatype as i16,
                xy,
                properties: Vec::new(),
            }))
        }
        OASISElement::Path(path) => {
            let xy: Vec<(i32, i32)> = path
                .points
                .iter()
                .map(|(px, py)| ((path.x + px) as i32, (path.y + py) as i32))
                .collect();

            Some(GDSElement::Path(GPath {
                layer: path.layer as i16,
                datatype: path.datatype as i16,
                pathtype: 0,
                width: Some((path.half_width * 2) as i32),
                xy,
                properties: Vec::new(),
            }))
        }
        OASISElement::Text(text) => Some(GDSElement::Text(GText {
            layer: text.layer as i16,
            texttype: text.texttype as i16,
            string: text.string.clone(),
            xy: (text.x as i32, text.y as i32),
            presentation: None,
            strans: None,
            width: None,
            properties: Vec::new(),
        })),
        OASISElement::Placement(placement) => {
            let strans = if placement.magnification.is_some()
                || placement.angle.is_some()
                || placement.mirror
            {
                Some(crate::gdsii::STrans {
                    reflection: placement.mirror,
                    absolute_magnification: false,
                    absolute_angle: false,
                    magnification: placement.magnification,
                    angle: placement.angle,
                })
            } else {
                None
            };

            Some(GDSElement::StructRef(StructRef {
                sname: placement.cell_name.clone(),
                xy: (placement.x as i32, placement.y as i32),
                strans,
                properties: Vec::new(),
            }))
        }
        OASISElement::Trapezoid(_) | OASISElement::CTrapezoid(_) | OASISElement::Circle(_) => {
            // These would need to be converted to polygons
            // For now, skip
            None
        }
    }
}

// Helper functions
pub fn is_rectangle(points: &[(i32, i32)]) -> bool {
    // A rectangle has 4 or 5 points (5 if closed) with right angles
    if points.len() != 4 && points.len() != 5 {
        return false;
    }

    // Check if first and last points are the same (closed polygon)
    let pts = if points.len() == 5 && points[0] == points[4] {
        &points[0..4]
    } else if points.len() == 4 {
        points
    } else {
        return false;
    };

    // Check for right angles
    let v1 = (pts[1].0 - pts[0].0, pts[1].1 - pts[0].1);
    let v2 = (pts[2].0 - pts[1].0, pts[2].1 - pts[1].1);
    let v3 = (pts[3].0 - pts[2].0, pts[3].1 - pts[2].1);
    let v4 = (pts[0].0 - pts[3].0, pts[0].1 - pts[3].1);

    // Check if vectors are perpendicular (dot product = 0) and opposite sides are parallel
    v1.0 * v2.0 + v1.1 * v2.1 == 0
        && v2.0 * v3.0 + v2.1 * v3.1 == 0
        && v3.0 * v4.0 + v3.1 * v4.1 == 0
        && v4.0 * v1.0 + v4.1 * v1.1 == 0
}

fn calculate_rectangle_bounds(points: &[(i32, i32)]) -> (i32, i32, i32, i32) {
    let mut x_min = i32::MAX;
    let mut y_min = i32::MAX;
    let mut x_max = i32::MIN;
    let mut y_max = i32::MIN;

    for (x, y) in points {
        x_min = x_min.min(*x);
        y_min = y_min.min(*y);
        x_max = x_max.max(*x);
        y_max = y_max.max(*y);
    }

    (x_min, y_min, x_max - x_min, y_max - y_min)
}

