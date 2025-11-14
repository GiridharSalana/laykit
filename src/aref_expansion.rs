// AREF (Array Reference) expansion utilities
// Expands array references into individual structure references

use crate::gdsii::{ArrayRef, GDSElement, StructRef};

/// Expand an ArrayRef into individual StructRef elements
pub fn expand_array_ref(aref: &ArrayRef) -> Vec<GDSElement> {
    if aref.xy.len() != 3 {
        // Invalid AREF, return empty vec
        return Vec::new();
    }

    let origin = aref.xy[0];
    let col_ref = aref.xy[1];
    let row_ref = aref.xy[2];

    // Calculate spacing
    let col_spacing_x = (col_ref.0 - origin.0) / aref.columns as i32;
    let col_spacing_y = (col_ref.1 - origin.1) / aref.columns as i32;

    let row_spacing_x = (row_ref.0 - origin.0) / aref.rows as i32;
    let row_spacing_y = (row_ref.1 - origin.1) / aref.rows as i32;

    let mut expanded = Vec::new();

    // Generate individual references
    for row in 0..aref.rows {
        for col in 0..aref.columns {
            let x = origin.0 + col as i32 * col_spacing_x + row as i32 * row_spacing_x;
            let y = origin.1 + col as i32 * col_spacing_y + row as i32 * row_spacing_y;

            expanded.push(GDSElement::StructRef(StructRef {
                sname: aref.sname.clone(),
                xy: (x, y),
                strans: aref.strans.clone(),
                elflags: aref.elflags,
                plex: aref.plex,
                properties: aref.properties.clone(),
            }));
        }
    }

    expanded
}

/// Expand all ArrayRefs in an element list
pub fn expand_all_array_refs(elements: &[GDSElement]) -> Vec<GDSElement> {
    let mut result = Vec::new();

    for element in elements {
        match element {
            GDSElement::ArrayRef(aref) => {
                result.extend(expand_array_ref(aref));
            }
            other => {
                result.push(other.clone());
            }
        }
    }

    result
}

/// Count total instances after expansion
pub fn count_expanded_instances(elements: &[GDSElement]) -> usize {
    let mut count = 0;

    for element in elements {
        match element {
            GDSElement::ArrayRef(aref) => {
                count += (aref.rows as usize) * (aref.columns as usize);
            }
            GDSElement::StructRef(_) => {
                count += 1;
            }
            _ => {}
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gdsii::{ArrayRef, GDSElement, GDSProperty};

    #[test]
    fn test_expand_simple_array() {
        let aref = ArrayRef {
            sname: "CELL".to_string(),
            columns: 3,
            rows: 2,
            xy: vec![(0, 0), (300, 0), (0, 200)],
            strans: None,
            elflags: None,
            plex: None,
            properties: Vec::new(),
        };

        let expanded = expand_array_ref(&aref);

        // 3 columns × 2 rows = 6 instances
        assert_eq!(expanded.len(), 6);

        // Check positions
        if let GDSElement::StructRef(sref) = &expanded[0] {
            assert_eq!(sref.xy, (0, 0)); // First instance at origin
            assert_eq!(sref.sname, "CELL");
        } else {
            panic!("Expected StructRef");
        }

        if let GDSElement::StructRef(sref) = &expanded[1] {
            assert_eq!(sref.xy, (100, 0)); // Second in first row
        }

        if let GDSElement::StructRef(sref) = &expanded[3] {
            assert_eq!(sref.xy, (0, 100)); // First in second row
        }
    }

    #[test]
    fn test_expand_1x1_array() {
        let aref = ArrayRef {
            sname: "SINGLE".to_string(),
            columns: 1,
            rows: 1,
            xy: vec![(100, 200), (100, 200), (100, 200)],
            strans: None,
            elflags: None,
            plex: None,
            properties: Vec::new(),
        };

        let expanded = expand_array_ref(&aref);

        assert_eq!(expanded.len(), 1);

        if let GDSElement::StructRef(sref) = &expanded[0] {
            assert_eq!(sref.xy, (100, 200));
            assert_eq!(sref.sname, "SINGLE");
        }
    }

    #[test]
    fn test_expand_array_with_properties() {
        let props = vec![GDSProperty {
            attribute: 1,
            value: "test".to_string(),
        }];

        let aref = ArrayRef {
            sname: "CELL_PROPS".to_string(),
            columns: 2,
            rows: 1,
            xy: vec![(0, 0), (100, 0), (0, 0)],
            strans: None,
            elflags: None,
            plex: None,
            properties: props.clone(),
        };

        let expanded = expand_array_ref(&aref);

        assert_eq!(expanded.len(), 2);

        // Check that properties are preserved
        for element in &expanded {
            if let GDSElement::StructRef(sref) = element {
                assert_eq!(sref.properties.len(), props.len());
            }
        }
    }

    #[test]
    fn test_expand_all_array_refs() {
        let elements = vec![
            GDSElement::ArrayRef(ArrayRef {
                sname: "CELL1".to_string(),
                columns: 2,
                rows: 2,
                xy: vec![(0, 0), (200, 0), (0, 200)],
                strans: None,
                elflags: None,
                plex: None,
                properties: Vec::new(),
            }),
            GDSElement::StructRef(StructRef {
                sname: "CELL2".to_string(),
                xy: (1000, 1000),
                strans: None,
                elflags: None,
                plex: None,
                properties: Vec::new(),
            }),
            GDSElement::ArrayRef(ArrayRef {
                sname: "CELL3".to_string(),
                columns: 3,
                rows: 1,
                xy: vec![(0, 0), (300, 0), (0, 0)],
                strans: None,
                elflags: None,
                plex: None,
                properties: Vec::new(),
            }),
        ];

        let expanded = expand_all_array_refs(&elements);

        // First AREF: 2×2=4, StructRef: 1, Second AREF: 3×1=3
        // Total: 4 + 1 + 3 = 8
        assert_eq!(expanded.len(), 8);
    }

    #[test]
    fn test_count_expanded_instances() {
        let elements = vec![
            GDSElement::ArrayRef(ArrayRef {
                sname: "A".to_string(),
                columns: 4,
                rows: 3,
                xy: vec![(0, 0), (400, 0), (0, 300)],
                strans: None,
                elflags: None,
                plex: None,
                properties: Vec::new(),
            }),
            GDSElement::StructRef(StructRef {
                sname: "B".to_string(),
                xy: (0, 0),
                strans: None,
                elflags: None,
                plex: None,
                properties: Vec::new(),
            }),
        ];

        let count = count_expanded_instances(&elements);

        // AREF: 4×3=12, StructRef: 1, Total: 13
        assert_eq!(count, 13);
    }

    #[test]
    fn test_expand_invalid_array() {
        let aref = ArrayRef {
            sname: "INVALID".to_string(),
            columns: 2,
            rows: 2,
            xy: vec![(0, 0), (100, 0)], // Only 2 points instead of 3!
            strans: None,
            elflags: None,
            plex: None,
            properties: Vec::new(),
        };

        let expanded = expand_array_ref(&aref);

        // Should return empty vec for invalid AREF
        assert_eq!(expanded.len(), 0);
    }
}
