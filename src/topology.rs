// Topology utilities for IC layout cell hierarchies
// Provides cell flattening, dependency ordering, hierarchy validation
// Mirrors the cell hierarchy API of gdstk

use std::collections::{HashMap, HashSet, VecDeque};
use crate::gdsii::{
    ArrayRef, Boundary, GDSElement, GDSIIFile, GDSStructure, GPath, GText, GDSBox,
    Node, StructRef,
};
#[cfg(test)]
use crate::gdsii::GDSTime;
use crate::geometry::affine_transform;

// ============================================================================
// Cell reference analysis
// ============================================================================

/// Get all cell names directly referenced by a structure (SREF and AREF targets)
#[allow(dead_code)]
pub fn direct_references(structure: &GDSStructure) -> Vec<String> {
    let mut refs = Vec::new();
    for element in &structure.elements {
        match element {
            GDSElement::StructRef(s) => refs.push(s.sname.clone()),
            GDSElement::ArrayRef(a) => refs.push(a.sname.clone()),
            _ => {}
        }
    }
    refs.sort();
    refs.dedup();
    refs
}

/// Get all cell names that `cell_name` depends on (direct and transitive).
/// Returns a set of structure names.
pub fn cell_dependencies(cell_name: &str, library: &GDSIIFile) -> HashSet<String> {
    let cell_map: HashMap<&str, &GDSStructure> = library
        .structures
        .iter()
        .map(|s| (s.name.as_str(), s))
        .collect();

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(cell_name.to_string());

    while let Some(name) = queue.pop_front() {
        if visited.contains(&name) {
            continue;
        }
        if name != cell_name {
            visited.insert(name.clone());
        }
        if let Some(structure) = cell_map.get(name.as_str()) {
            for dep in direct_references(structure) {
                if !visited.contains(&dep) {
                    queue.push_back(dep);
                }
            }
        }
    }

    visited
}

/// Get all top-level cells in a library (cells not referenced by any other cell).
pub fn top_level_cells(library: &GDSIIFile) -> Vec<&GDSStructure> {
    let mut referenced: HashSet<&str> = HashSet::new();
    for structure in &library.structures {
        for dep in direct_references(structure) {
            for s in &library.structures {
                if s.name == dep {
                    referenced.insert(s.name.as_str());
                }
            }
        }
    }
    library
        .structures
        .iter()
        .filter(|s| !referenced.contains(s.name.as_str()))
        .collect()
}

/// Return structures sorted in dependency order (leaves first, roots last).
/// This is a topological sort: if A uses B, then B appears before A.
pub fn dependency_order(library: &GDSIIFile) -> Vec<usize> {
    let n = library.structures.len();
    let name_to_idx: HashMap<&str, usize> = library
        .structures
        .iter()
        .enumerate()
        .map(|(i, s)| (s.name.as_str(), i))
        .collect();

    // Build adjacency: structure[i] → list of structures it depends on
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut in_degree: Vec<usize> = vec![0; n];

    for (i, structure) in library.structures.iter().enumerate() {
        for dep_name in direct_references(structure) {
            if let Some(&j) = name_to_idx.get(dep_name.as_str()) {
                if i != j {
                    adj[j].push(i); // j is needed by i, so j goes first
                    in_degree[i] += 1;
                }
            }
        }
    }

    // Kahn's algorithm for topological sort
    let mut queue: VecDeque<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut order = Vec::new();

    while let Some(u) = queue.pop_front() {
        order.push(u);
        for &v in &adj[u] {
            in_degree[v] -= 1;
            if in_degree[v] == 0 {
                queue.push_back(v);
            }
        }
    }

    // If there are remaining nodes, there's a cycle; append them anyway
    if order.len() < n {
        for i in 0..n {
            if !order.contains(&i) {
                order.push(i);
            }
        }
    }

    order
}

/// Detect circular references in a library.
/// Returns a list of cycles found (each cycle is a vec of cell names).
pub fn detect_cycles(library: &GDSIIFile) -> Vec<Vec<String>> {
    let order = dependency_order(library);
    let mut cycles = Vec::new();

    // If order.len() < n, there are cycles (detected by comparing dependency order count)
    // Simple detection: check if any cell references itself transitively
    let n = library.structures.len();
    if order.len() == n {
        return cycles; // No cycles
    }

    // Find cells not in the valid topological order
    let in_order: HashSet<usize> = order.iter().cloned().collect();
    for i in 0..n {
        if !in_order.contains(&i) {
            cycles.push(vec![library.structures[i].name.clone()]);
        }
    }
    cycles
}

/// Validate the library hierarchy.
/// Returns Ok(()) if valid, Err with a list of issues if not.
pub fn validate_hierarchy(library: &GDSIIFile) -> Result<(), Vec<String>> {
    let mut issues = Vec::new();
    let cell_names: HashSet<&str> = library.structures.iter().map(|s| s.name.as_str()).collect();

    // Check for undefined references
    for structure in &library.structures {
        for dep in direct_references(structure) {
            if !cell_names.contains(dep.as_str()) {
                issues.push(format!(
                    "Cell '{}' references undefined cell '{}'",
                    structure.name, dep
                ));
            }
        }
    }

    // Check for duplicate cell names
    let mut seen_names: HashSet<&str> = HashSet::new();
    for structure in &library.structures {
        if !seen_names.insert(structure.name.as_str()) {
            issues.push(format!("Duplicate cell name: '{}'", structure.name));
        }
    }

    // Check for cycles
    let cycles = detect_cycles(library);
    for cycle in cycles {
        issues.push(format!("Circular reference involving: {:?}", cycle));
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

// ============================================================================
// Cell flattening
// ============================================================================

/// Flatten a structure by expanding all cell references recursively.
/// Returns a new structure with all references replaced by actual geometry.
/// `max_depth` limits the recursion depth (None = unlimited).
pub fn flatten_structure(
    structure: &GDSStructure,
    library: &GDSIIFile,
    max_depth: Option<u32>,
) -> GDSStructure {
    let cell_map: HashMap<&str, &GDSStructure> = library
        .structures
        .iter()
        .map(|s| (s.name.as_str(), s))
        .collect();

    let mut flat_elements = Vec::new();
    flatten_elements(
        &structure.elements,
        &cell_map,
        (0.0, 0.0),
        0.0,
        1.0,
        false,
        0,
        max_depth.unwrap_or(u32::MAX),
        &mut flat_elements,
    );

    GDSStructure {
        name: structure.name.clone(),
        creation_time: structure.creation_time.clone(),
        modification_time: structure.modification_time.clone(),
        strclass: structure.strclass,
        elements: flat_elements,
    }
}

#[allow(clippy::too_many_arguments)]
fn flatten_elements(
    elements: &[GDSElement],
    cell_map: &HashMap<&str, &GDSStructure>,
    translation: (f64, f64),
    rotation: f64,
    magnification: f64,
    x_reflection: bool,
    depth: u32,
    max_depth: u32,
    output: &mut Vec<GDSElement>,
) {
    for element in elements {
        match element {
            GDSElement::StructRef(sref) => {
                if depth < max_depth {
                    if let Some(&sub) = cell_map.get(sref.sname.as_str()) {
                        let (t, r, m, xr) = compose_transform(
                            translation,
                            rotation,
                            magnification,
                            x_reflection,
                            sref,
                        );
                        flatten_elements(
                            &sub.elements,
                            cell_map,
                            t, r, m, xr,
                            depth + 1,
                            max_depth,
                            output,
                        );
                    }
                } else {
                    output.push(GDSElement::StructRef(transform_sref(
                        sref, translation, rotation, magnification, x_reflection,
                    )));
                }
            }
            GDSElement::ArrayRef(aref) => {
                if depth < max_depth {
                    if let Some(&sub) = cell_map.get(aref.sname.as_str()) {
                        let instances = expand_aref_with_transform(
                            aref, translation, rotation, magnification, x_reflection,
                        );
                        for (t, r, m, xr) in instances {
                            flatten_elements(
                                &sub.elements,
                                cell_map,
                                t, r, m, xr,
                                depth + 1,
                                max_depth,
                                output,
                            );
                        }
                    }
                } else {
                    output.push(GDSElement::ArrayRef(aref.clone()));
                }
            }
            other => {
                let transformed = transform_element(other, translation, rotation, magnification, x_reflection);
                output.push(transformed);
            }
        }
    }
}

fn compose_transform(
    parent_t: (f64, f64),
    parent_r: f64,
    parent_m: f64,
    parent_xr: bool,
    sref: &StructRef,
) -> ((f64, f64), f64, f64, bool) {
    let child_m = sref.strans.as_ref().and_then(|s| s.magnification).unwrap_or(1.0);
    let child_r = sref.strans.as_ref().and_then(|s| s.angle).unwrap_or(0.0).to_radians();
    let child_xr = sref.strans.as_ref().map(|s| s.reflection).unwrap_or(false);

    let child_pos = affine_transform(
        &[(sref.xy.0 as f64, sref.xy.1 as f64)],
        parent_t,
        parent_r,
        parent_m,
        parent_xr,
    )[0];

    let combined_xr = parent_xr ^ child_xr;
    let combined_r = if parent_xr { parent_r - child_r } else { parent_r + child_r };
    let combined_m = parent_m * child_m;

    (child_pos, combined_r, combined_m, combined_xr)
}

fn expand_aref_with_transform(
    aref: &ArrayRef,
    translation: (f64, f64),
    rotation: f64,
    magnification: f64,
    x_reflection: bool,
) -> Vec<((f64, f64), f64, f64, bool)> {
    if aref.xy.len() < 3 {
        return Vec::new();
    }

    let child_m = aref.strans.as_ref().and_then(|s| s.magnification).unwrap_or(1.0);
    let child_r = aref.strans.as_ref().and_then(|s| s.angle).unwrap_or(0.0).to_radians();
    let child_xr = aref.strans.as_ref().map(|s| s.reflection).unwrap_or(false);

    let combined_xr = x_reflection ^ child_xr;
    let combined_r = if x_reflection { rotation - child_r } else { rotation + child_r };
    let combined_m = magnification * child_m;

    let origin = (aref.xy[0].0 as f64, aref.xy[0].1 as f64);
    let col_end = (aref.xy[1].0 as f64, aref.xy[1].1 as f64);
    let row_end = (aref.xy[2].0 as f64, aref.xy[2].1 as f64);

    let cols = aref.columns as i32;
    let rows = aref.rows as i32;

    let col_dx = if cols > 0 { (col_end.0 - origin.0) / cols as f64 } else { 0.0 };
    let col_dy = if cols > 0 { (col_end.1 - origin.1) / cols as f64 } else { 0.0 };
    let row_dx = if rows > 0 { (row_end.0 - origin.0) / rows as f64 } else { 0.0 };
    let row_dy = if rows > 0 { (row_end.1 - origin.1) / rows as f64 } else { 0.0 };

    let mut instances = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            let local_pos = (
                origin.0 + col as f64 * col_dx + row as f64 * row_dx,
                origin.1 + col as f64 * col_dy + row as f64 * row_dy,
            );
            let world_pos = affine_transform(
                &[local_pos],
                translation,
                rotation,
                magnification,
                x_reflection,
            )[0];
            instances.push((world_pos, combined_r, combined_m, combined_xr));
        }
    }
    instances
}

fn transform_sref(
    sref: &StructRef,
    translation: (f64, f64),
    rotation: f64,
    magnification: f64,
    x_reflection: bool,
) -> StructRef {
    let pos = affine_transform(
        &[(sref.xy.0 as f64, sref.xy.1 as f64)],
        translation,
        rotation,
        magnification,
        x_reflection,
    )[0];
    StructRef {
        sname: sref.sname.clone(),
        xy: (pos.0.round() as i32, pos.1.round() as i32),
        strans: sref.strans.clone(),
        elflags: sref.elflags,
        plex: sref.plex,
        properties: sref.properties.clone(),
    }
}

fn transform_element(
    element: &GDSElement,
    translation: (f64, f64),
    rotation: f64,
    magnification: f64,
    x_reflection: bool,
) -> GDSElement {
    let transform_pts = |pts: &[(i32, i32)]| -> Vec<(i32, i32)> {
        let fpts: Vec<(f64, f64)> = pts.iter().map(|&(x, y)| (x as f64, y as f64)).collect();
        let tpts = affine_transform(&fpts, translation, rotation, magnification, x_reflection);
        tpts.iter().map(|&(x, y)| (x.round() as i32, y.round() as i32)).collect()
    };

    let transform_pt = |pt: (i32, i32)| -> (i32, i32) {
        let t = affine_transform(
            &[(pt.0 as f64, pt.1 as f64)],
            translation, rotation, magnification, x_reflection,
        )[0];
        (t.0.round() as i32, t.1.round() as i32)
    };

    match element {
        GDSElement::Boundary(b) => GDSElement::Boundary(Boundary {
            layer: b.layer,
            datatype: b.datatype,
            xy: transform_pts(&b.xy),
            elflags: b.elflags,
            plex: b.plex,
            properties: b.properties.clone(),
        }),
        GDSElement::Path(p) => GDSElement::Path(GPath {
            layer: p.layer,
            datatype: p.datatype,
            pathtype: p.pathtype,
            width: p.width,
            bgnextn: p.bgnextn,
            endextn: p.endextn,
            xy: transform_pts(&p.xy),
            elflags: p.elflags,
            plex: p.plex,
            properties: p.properties.clone(),
        }),
        GDSElement::Text(t) => GDSElement::Text(GText {
            layer: t.layer,
            texttype: t.texttype,
            string: t.string.clone(),
            xy: transform_pt(t.xy),
            presentation: t.presentation,
            strans: t.strans.clone(),
            width: t.width,
            elflags: t.elflags,
            plex: t.plex,
            properties: t.properties.clone(),
        }),
        GDSElement::Node(n) => GDSElement::Node(Node {
            layer: n.layer,
            nodetype: n.nodetype,
            xy: transform_pts(&n.xy),
            elflags: n.elflags,
            plex: n.plex,
            properties: n.properties.clone(),
        }),
        GDSElement::Box(b) => GDSElement::Box(GDSBox {
            layer: b.layer,
            boxtype: b.boxtype,
            xy: transform_pts(&b.xy),
            elflags: b.elflags,
            plex: b.plex,
            properties: b.properties.clone(),
        }),
        // References are handled separately
        other => other.clone(),
    }
}

// ============================================================================
// Library merging
// ============================================================================

/// Merge another library into this one.
/// Existing cells with the same name are kept (no overwrite by default).
/// Returns the number of cells added.
pub fn merge_library(target: &mut GDSIIFile, source: &GDSIIFile) -> usize {
    let existing: HashSet<String> = target.structures.iter().map(|s| s.name.clone()).collect();
    let mut to_add = Vec::new();
    for structure in &source.structures {
        if !existing.contains(&structure.name) {
            to_add.push(structure.clone());
        }
    }
    let added = to_add.len();
    target.structures.extend(to_add);
    added
}

/// Merge another library into this one, overwriting cells with the same name.
/// Returns the number of cells replaced.
pub fn merge_library_overwrite(target: &mut GDSIIFile, source: &GDSIIFile) -> usize {
    let mut replaced = 0;
    for source_struct in &source.structures {
        if let Some(existing) = target.structures.iter_mut().find(|s| s.name == source_struct.name) {
            *existing = source_struct.clone();
            replaced += 1;
        } else {
            target.structures.push(source_struct.clone());
        }
    }
    replaced
}

// ============================================================================
// Element filtering / querying
// ============================================================================

/// Filter elements in a structure by layer
pub fn filter_by_layer(structure: &GDSStructure, layer: i16) -> Vec<&GDSElement> {
    structure.elements.iter().filter(|e| element_layer(e) == Some(layer)).collect()
}

/// Get the layer of a GDSII element (None for elements without a layer)
pub fn element_layer(element: &GDSElement) -> Option<i16> {
    match element {
        GDSElement::Boundary(b) => Some(b.layer),
        GDSElement::Path(p) => Some(p.layer),
        GDSElement::Text(t) => Some(t.layer),
        GDSElement::Node(n) => Some(n.layer),
        GDSElement::Box(b) => Some(b.layer),
        GDSElement::StructRef(_) | GDSElement::ArrayRef(_) => None,
    }
}

/// Collect all layers used in a structure
pub fn layers_in_structure(structure: &GDSStructure) -> Vec<i16> {
    let mut layers: HashSet<i16> = HashSet::new();
    for element in &structure.elements {
        if let Some(l) = element_layer(element) {
            layers.insert(l);
        }
    }
    let mut result: Vec<i16> = layers.into_iter().collect();
    result.sort();
    result
}

/// Collect all layers used in an entire library
pub fn layers_in_library(library: &GDSIIFile) -> Vec<i16> {
    let mut layers: HashSet<i16> = HashSet::new();
    for structure in &library.structures {
        for l in layers_in_structure(structure) {
            layers.insert(l);
        }
    }
    let mut result: Vec<i16> = layers.into_iter().collect();
    result.sort();
    result
}

/// Count total elements in a library
pub fn total_element_count(library: &GDSIIFile) -> usize {
    library.structures.iter().map(|s| s.elements.len()).sum()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_library() -> GDSIIFile {
        let mut lib = GDSIIFile::new("TEST".to_string());

        let leaf = GDSStructure {
            name: "LEAF".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
            elements: vec![GDSElement::Boundary(Boundary {
                layer: 1,
                datatype: 0,
                xy: vec![(0, 0), (10, 0), (10, 10), (0, 10), (0, 0)],
                elflags: None, plex: None, properties: Vec::new(),
            })],
        };

        let mid = GDSStructure {
            name: "MID".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
            elements: vec![GDSElement::StructRef(StructRef {
                sname: "LEAF".to_string(),
                xy: (100, 200),
                strans: None,
                elflags: None, plex: None, properties: Vec::new(),
            })],
        };

        let top = GDSStructure {
            name: "TOP".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
            elements: vec![GDSElement::StructRef(StructRef {
                sname: "MID".to_string(),
                xy: (0, 0),
                strans: None,
                elflags: None, plex: None, properties: Vec::new(),
            })],
        };

        lib.structures.push(leaf);
        lib.structures.push(mid);
        lib.structures.push(top);
        lib
    }

    #[test]
    fn test_direct_references() {
        let lib = make_library();
        let refs = direct_references(&lib.structures[1]); // MID
        assert_eq!(refs, vec!["LEAF"]);
    }

    #[test]
    fn test_top_level_cells() {
        let lib = make_library();
        let tops = top_level_cells(&lib);
        assert_eq!(tops.len(), 1);
        assert_eq!(tops[0].name, "TOP");
    }

    #[test]
    fn test_dependency_order() {
        let lib = make_library();
        let order = dependency_order(&lib);
        assert_eq!(order.len(), 3);
        // LEAF should come before MID, MID before TOP
        let leaf_pos = order.iter().position(|&i| lib.structures[i].name == "LEAF").unwrap();
        let mid_pos = order.iter().position(|&i| lib.structures[i].name == "MID").unwrap();
        let top_pos = order.iter().position(|&i| lib.structures[i].name == "TOP").unwrap();
        assert!(leaf_pos < mid_pos);
        assert!(mid_pos < top_pos);
    }

    #[test]
    fn test_validate_hierarchy_valid() {
        let lib = make_library();
        assert!(validate_hierarchy(&lib).is_ok());
    }

    #[test]
    fn test_validate_hierarchy_missing_ref() {
        let mut lib = GDSIIFile::new("TEST".to_string());
        lib.structures.push(GDSStructure {
            name: "TOP".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
            elements: vec![GDSElement::StructRef(StructRef {
                sname: "NONEXISTENT".to_string(),
                xy: (0, 0),
                strans: None,
                elflags: None, plex: None, properties: Vec::new(),
            })],
        });
        let result = validate_hierarchy(&lib);
        assert!(result.is_err());
        let issues = result.unwrap_err();
        assert!(issues.iter().any(|i| i.contains("NONEXISTENT")));
    }

    #[test]
    fn test_flatten_structure_simple() {
        let lib = make_library();
        let top = &lib.structures[2]; // TOP
        let flat = flatten_structure(top, &lib, None);

        // Flattened should have a Boundary from the leaf (not a StructRef)
        let has_boundary = flat.elements.iter().any(|e| matches!(e, GDSElement::Boundary(_)));
        let has_sref = flat.elements.iter().any(|e| matches!(e, GDSElement::StructRef(_)));
        assert!(has_boundary, "Flattened structure should contain Boundary elements");
        assert!(!has_sref, "Flattened structure should not contain StructRef elements");
    }

    #[test]
    fn test_flatten_structure_translation() {
        let lib = make_library();
        let top = &lib.structures[2]; // TOP references MID at (0,0), MID references LEAF at (100,200)
        let flat = flatten_structure(top, &lib, None);

        // The leaf boundary at (0,0)-(10,10) should be translated by (100, 200)
        if let Some(GDSElement::Boundary(b)) = flat.elements.first() {
            // After translation by (100,200) from MID's SREF
            assert_eq!(b.xy[0], (100, 200));
        }
    }

    #[test]
    fn test_flatten_max_depth() {
        let lib = make_library();
        let top = &lib.structures[2];
        // Only flatten 1 level deep: TOP → MID (not MID → LEAF)
        let flat = flatten_structure(top, &lib, Some(1));
        // Should have flattened MID but not LEAF
        let has_sref = flat.elements.iter().any(|e| matches!(e, GDSElement::StructRef(_)));
        assert!(has_sref, "With max_depth=1, should still have StructRefs for deeper cells");
    }

    #[test]
    fn test_merge_library() {
        let mut lib_a = GDSIIFile::new("LIBA".to_string());
        lib_a.structures.push(GDSStructure {
            name: "CELL_A".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
            elements: Vec::new(),
        });

        let mut lib_b = GDSIIFile::new("LIBB".to_string());
        lib_b.structures.push(GDSStructure {
            name: "CELL_B".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
            elements: Vec::new(),
        });
        lib_b.structures.push(GDSStructure {
            name: "CELL_A".to_string(), // duplicate
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
            elements: Vec::new(),
        });

        let added = merge_library(&mut lib_a, &lib_b);
        assert_eq!(added, 1); // Only CELL_B was added
        assert_eq!(lib_a.structures.len(), 2);
    }

    #[test]
    fn test_layers_in_structure() {
        let lib = make_library();
        let layers = layers_in_structure(&lib.structures[0]); // LEAF
        assert_eq!(layers, vec![1]);
    }

    #[test]
    fn test_layers_in_library() {
        let lib = make_library();
        let layers = layers_in_library(&lib);
        assert!(layers.contains(&1));
    }

    #[test]
    fn test_filter_by_layer() {
        let lib = make_library();
        let elements_on_layer1 = filter_by_layer(&lib.structures[0], 1);
        assert_eq!(elements_on_layer1.len(), 1);
        let elements_on_layer2 = filter_by_layer(&lib.structures[0], 2);
        assert_eq!(elements_on_layer2.len(), 0);
    }

    #[test]
    fn test_total_element_count() {
        let lib = make_library();
        assert_eq!(total_element_count(&lib), 3); // 1 boundary + 2 srefs
    }

    #[test]
    fn test_cell_dependencies() {
        let lib = make_library();
        let deps = cell_dependencies("TOP", &lib);
        assert!(deps.contains("MID"), "Expected MID in deps: {:?}", deps);
        assert!(deps.contains("LEAF"), "Expected LEAF in deps: {:?}", deps);
        assert!(!deps.contains("TOP"), "TOP should not be in its own deps");
    }

    #[test]
    fn test_aref_flattening() {
        let mut lib = GDSIIFile::new("TEST".to_string());

        let leaf = GDSStructure {
            name: "UNIT".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
            elements: vec![GDSElement::Boundary(Boundary {
                layer: 1,
                datatype: 0,
                xy: vec![(0, 0), (5, 0), (5, 5), (0, 5), (0, 0)],
                elflags: None, plex: None, properties: Vec::new(),
            })],
        };

        let top = GDSStructure {
            name: "ARRAY_TOP".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
            elements: vec![GDSElement::ArrayRef(ArrayRef {
                sname: "UNIT".to_string(),
                columns: 2,
                rows: 2,
                xy: vec![(0, 0), (20, 0), (0, 20)],
                strans: None,
                elflags: None, plex: None, properties: Vec::new(),
            })],
        };

        lib.structures.push(leaf);
        lib.structures.push(top);

        let flat = flatten_structure(&lib.structures[1], &lib, None);
        // 2×2 array = 4 instances of UNIT, each with 1 Boundary = 4 boundaries
        let boundary_count = flat.elements.iter().filter(|e| matches!(e, GDSElement::Boundary(_))).count();
        assert_eq!(boundary_count, 4, "Expected 4 boundaries from 2x2 array, got {}", boundary_count);
    }
}
