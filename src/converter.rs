// Conversion utilities between OASIS and GDSII formats

use crate::curve::ellipse;
use crate::gdsii::ArrayRef;
use crate::oasis::Repetition;
use crate::gdsii::{
    Boundary, GDSElement, GDSBox, GDSIIFile, GDSProperty, GDSStructure, GDSTime, GPath, GText,
    Node, StructRef,
};
use crate::oasis::{
    Circle, CTrapezoid, OASISCell, OASISElement, OASISFile, OPath, OText, Placement, Polygon,
    Property, PropertyValue, Rectangle, Trapezoid,
};

/// Map GDSII element properties to OASIS properties.
pub fn gds_properties_to_oasis(props: &[GDSProperty]) -> Vec<Property> {
    props
        .iter()
        .map(|p| Property {
            name: format!("GDS_ATTR_{}", p.attribute),
            values: vec![PropertyValue::String(p.value.clone())],
        })
        .collect()
}

/// Map OASIS properties back to GDSII PROPATTR/PROPVALUE pairs.
pub fn oasis_properties_to_gds(props: &[Property]) -> Vec<GDSProperty> {
    props
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            let attribute = parse_gds_attribute_name(&p.name).unwrap_or(1000 + idx as i16);
            GDSProperty {
                attribute,
                value: property_value_as_string(p),
            }
        })
        .collect()
}

fn parse_gds_attribute_name(name: &str) -> Option<i16> {
    name.strip_prefix("GDS_ATTR_").and_then(|s| s.parse().ok())
}

fn property_value_as_string(prop: &Property) -> String {
    prop.values
        .first()
        .map(|v| match v {
            PropertyValue::String(s) => s.clone(),
            PropertyValue::Integer(n) => n.to_string(),
            PropertyValue::Real(r) => r.to_string(),
            PropertyValue::Boolean(b) => b.to_string(),
            PropertyValue::Reference(r) => r.to_string(),
        })
        .unwrap_or_default()
}

/// Convert GDSII to OASIS
pub fn gdsii_to_oasis(gds: &GDSIIFile) -> Result<OASISFile, Box<dyn std::error::Error>> {
    let mut oasis = OASISFile::new();

    // Convert units (GDSII uses database units in meters, OASIS uses unit multiplier)
    oasis.unit = gds.units.1; // database unit in meters
    oasis.user_unit = gds.units.0;
    oasis.start_scaling = if gds.units.1 > 0.0 {
        gds.units.0 / gds.units.1
    } else {
        1000.0
    };
    oasis.version = "1.0".to_string();
    oasis.library_name = gds.library_name.clone();

    // Convert structures to cells
    for (idx, structure) in gds.structures.iter().enumerate() {
        let cell_ref = idx as u32;
        oasis
            .names
            .cell_names
            .insert(cell_ref, structure.name.clone());

        let mut cell = OASISCell {
            name: structure.name.clone(),
            name_ref: None,
            elements: Vec::new(),
        };

        // Convert elements (AREF expands to multiple placements, matching gdstk geometry)
        for element in &structure.elements {
            push_gds_as_oasis(element, &mut cell.elements);
        }

        oasis.cells.push(cell);
    }

    Ok(oasis)
}

fn push_gds_as_oasis(element: &GDSElement, out: &mut Vec<OASISElement>) {
    if let Some(elem) = convert_gds_element_to_oasis(element) {
        out.push(elem);
    }
}

fn aref_to_placement(aref: &ArrayRef) -> Option<OASISElement> {
    if aref.xy.len() != 3 || aref.columns == 0 || aref.rows == 0 {
        return None;
    }
    let origin = aref.xy[0];
    let col_ref = aref.xy[1];
    let row_ref = aref.xy[2];
    let x_space = if aref.columns > 1 {
        ((col_ref.0 - origin.0).unsigned_abs() as u64) / (aref.columns as u32 - 1) as u64
    } else {
        0
    };
    let y_space = if aref.rows > 1 {
        ((row_ref.1 - origin.1).unsigned_abs() as u64) / (aref.rows as u32 - 1) as u64
    } else {
        0
    };
    Some(OASISElement::Placement(Placement {
        cell_name: aref.sname.clone(),
        x: origin.0 as i64,
        y: origin.1 as i64,
        magnification: aref.strans.as_ref().and_then(|st| st.magnification),
        angle: aref.strans.as_ref().and_then(|st| st.angle),
        mirror: aref
            .strans
            .as_ref()
            .map(|st| st.reflection)
            .unwrap_or(false),
        repetition: Some(Repetition::Matrix {
            x_count: aref.columns as u32,
            y_count: aref.rows as u32,
            x_space,
            y_space,
        }),
        properties: gds_properties_to_oasis(&aref.properties),
    }))
}

fn placement_strans(placement: &Placement) -> Option<crate::gdsii::STrans> {
    if placement.magnification.is_some() || placement.angle.is_some() || placement.mirror {
        Some(crate::gdsii::STrans {
            reflection: placement.mirror,
            absolute_magnification: false,
            absolute_angle: false,
            magnification: placement.magnification,
            angle: placement.angle,
        })
    } else {
        None
    }
}

fn placement_to_aref(
    placement: &Placement,
    columns: u32,
    rows: u32,
    x_space: u64,
    y_space: u64,
) -> Option<GDSElement> {
    let origin = (placement.x as i32, placement.y as i32);
    let col_ref = if columns > 1 {
        (
            origin.0 + (columns as i32 - 1) * x_space as i32,
            origin.1,
        )
    } else {
        origin
    };
    let row_ref = if rows > 1 {
        (
            origin.0,
            origin.1 + (rows as i32 - 1) * y_space as i32,
        )
    } else {
        origin
    };
    Some(GDSElement::ArrayRef(ArrayRef {
        sname: placement.cell_name.clone(),
        columns: columns as u16,
        rows: rows as u16,
        xy: vec![origin, col_ref, row_ref],
        strans: placement_strans(placement),
        elflags: None,
        plex: None,
        properties: oasis_properties_to_gds(&placement.properties),
    }))
}

/// Convert OASIS to GDSII
///
/// If `output_path` is provided, the library name will be derived from the filename.
/// Otherwise, defaults to "CONVERTED".
pub fn oasis_to_gdsii(oasis: &OASISFile) -> Result<GDSIIFile, Box<dyn std::error::Error>> {
    oasis_to_gdsii_with_name(oasis, None)
}

/// Convert OASIS to GDSII with a specific output filename
///
/// The library name will be derived from the filename stem.
/// Example: "output.gds" → library name "OUTPUT"
pub fn oasis_to_gdsii_with_name(
    oasis: &OASISFile,
    output_path: Option<&str>,
) -> Result<GDSIIFile, Box<dyn std::error::Error>> {
    use std::path::Path;

    // Prefer embedded library name from GDS→OASIS conversion, then output path, then default.
    let lib_name = if !oasis.library_name.is_empty() {
        oasis.library_name.clone()
    } else if let Some(path) = output_path {
        Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_uppercase())
            .unwrap_or_else(|| "CONVERTED".to_string())
    } else {
        "CONVERTED".to_string()
    };

    let mut gds = GDSIIFile::new(lib_name);

    let user_unit = if oasis.user_unit > 0.0 {
        oasis.user_unit
    } else {
        1e-6
    };
    gds.units = (user_unit, oasis.unit);

    // Convert cells to structures
    for cell in &oasis.cells {
        let mut structure = GDSStructure {
            name: cell.name.clone(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
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
                    properties: gds_properties_to_oasis(&boundary.properties),
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
                    properties: gds_properties_to_oasis(&boundary.properties),
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
                properties: gds_properties_to_oasis(&path.properties),
            }))
        }
        GDSElement::Text(text) => Some(OASISElement::Text(OText {
            layer: text.layer as u32,
            texttype: text.texttype as u32,
            x: text.xy.0 as i64,
            y: text.xy.1 as i64,
            string: text.string.clone(),
            string_ref: None,
            repetition: None,
            properties: gds_properties_to_oasis(&text.properties),
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
            properties: gds_properties_to_oasis(&sref.properties),
        })),
        GDSElement::ArrayRef(aref) => aref_to_placement(aref),
        GDSElement::Node(node) => node_to_oasis_polygon(node),
        GDSElement::Box(box_elem) => box_to_oasis(box_elem),
    }
}

fn node_to_oasis_polygon(node: &Node) -> Option<OASISElement> {
    if node.xy.len() < 3 {
        return None;
    }
    boundary_xy_to_oasis_polygon(
        node.layer,
        node.nodetype,
        &node.xy,
        &node.properties,
    )
}

fn box_to_oasis(box_elem: &GDSBox) -> Option<OASISElement> {
    if box_elem.xy.len() < 4 {
        return None;
    }
    boundary_xy_to_oasis_polygon(
        box_elem.layer,
        box_elem.boxtype,
        &box_elem.xy,
        &box_elem.properties,
    )
}

fn boundary_xy_to_oasis_polygon(
    layer: i16,
    datatype: i16,
    xy: &[(i32, i32)],
    properties: &[GDSProperty],
) -> Option<OASISElement> {
    if is_rectangle(xy) {
        let (x_min, y_min, width, height) = calculate_rectangle_bounds(xy);
        Some(OASISElement::Rectangle(Rectangle {
            layer: layer as u32,
            datatype: datatype as u32,
            x: x_min as i64,
            y: y_min as i64,
            width: width as u64,
            height: height as u64,
            repetition: None,
            properties: gds_properties_to_oasis(properties),
        }))
    } else {
        let points: Vec<(i64, i64)> = xy.iter().map(|(x, y)| (*x as i64, *y as i64)).collect();
        let x = points[0].0;
        let y = points[0].1;
        let relative_points: Vec<(i64, i64)> =
            points.iter().map(|(px, py)| (*px - x, *py - y)).collect();
        Some(OASISElement::Polygon(Polygon {
            layer: layer as u32,
            datatype: datatype as u32,
            x,
            y,
            points: relative_points,
            repetition: None,
            properties: gds_properties_to_oasis(properties),
        }))
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
                elflags: None,
                plex: None,
                properties: oasis_properties_to_gds(&rect.properties),
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
                elflags: None,
                plex: None,
                properties: oasis_properties_to_gds(&poly.properties),
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
                bgnextn: None,
                endextn: None,
                xy,
                elflags: None,
                plex: None,
                properties: oasis_properties_to_gds(&path.properties),
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
            elflags: None,
            plex: None,
            properties: oasis_properties_to_gds(&text.properties),
        })),
        OASISElement::Placement(placement) => {
            if let Some(Repetition::Matrix {
                x_count,
                y_count,
                x_space,
                y_space,
            }) = placement.repetition
            {
                if x_count > 1 || y_count > 1 {
                    return placement_to_aref(placement, x_count, y_count, x_space, y_space);
                }
            }
            let strans = placement_strans(placement);
            Some(GDSElement::StructRef(StructRef {
                sname: placement.cell_name.clone(),
                xy: (placement.x as i32, placement.y as i32),
                strans,
                elflags: None,
                plex: None,
                properties: oasis_properties_to_gds(&placement.properties),
            }))
        }
        OASISElement::Trapezoid(trap) => trapezoid_to_boundary(trap).map(GDSElement::Boundary),
        OASISElement::CTrapezoid(trap) => ctrapezoid_to_boundary(trap).map(GDSElement::Boundary),
        OASISElement::Circle(circle) => circle_to_boundary(circle).map(GDSElement::Boundary),
    }
}

fn trapezoid_to_boundary(trap: &Trapezoid) -> Option<Boundary> {
    let x = trap.x as i32;
    let y = trap.y as i32;
    let w = trap.width as i32;
    let h = trap.height as i32;
    let da = trap.delta_a as i32;
    let db = trap.delta_b as i32;
    let xy = if trap.orientation {
        vec![
            (x, y),
            (x, y + h),
            (x + w + db, y + h),
            (x + w + da, y),
            (x, y),
        ]
    } else {
        vec![
            (x, y),
            (x + w, y),
            (x + w - db, y + h),
            (x + da, y + h),
            (x, y),
        ]
    };
    Some(Boundary {
        layer: trap.layer as i16,
        datatype: trap.datatype as i16,
        xy,
        elflags: None,
        plex: None,
        properties: oasis_properties_to_gds(&trap.properties),
    })
}

fn ctrapezoid_to_boundary(trap: &CTrapezoid) -> Option<Boundary> {
    let x = trap.x as i32;
    let y = trap.y as i32;
    let w = trap.width as i32;
    let h = trap.height as i32;
    let xy = vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x, y)];
    Some(Boundary {
        layer: trap.layer as i16,
        datatype: trap.datatype as i16,
        xy,
        elflags: None,
        plex: None,
        properties: oasis_properties_to_gds(&trap.properties),
    })
}

fn circle_to_boundary(circle: &Circle) -> Option<Boundary> {
    let cx = circle.x as f64;
    let cy = circle.y as f64;
    let r = circle.radius as f64;
    let points = ellipse((cx, cy), r, r, 0.0, std::f64::consts::TAU, 1e-3, Some(64));
    if points.len() < 3 {
        return None;
    }
    let mut xy: Vec<(i32, i32)> = points
        .iter()
        .map(|(px, py)| (px.round() as i32, py.round() as i32))
        .collect();
    if xy.first() != xy.last() {
        if let Some(&first) = xy.first() {
            xy.push(first);
        }
    }
    Some(Boundary {
        layer: circle.layer as i16,
        datatype: circle.datatype as i16,
        xy,
        elflags: None,
        plex: None,
        properties: oasis_properties_to_gds(&circle.properties),
    })
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
