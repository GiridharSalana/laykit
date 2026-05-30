//! Clipper2 PolyTree execution + gdstk-style `link_holes` (see gdstk `clipper_tools.cpp`).

use crate::geometry::polygon_signed_area;
use clipper2c_sys::{
    clipper_allocate, clipper_clipper64, clipper_clipper64_add_clip, clipper_clipper64_add_subject,
    clipper_clipper64_execute_tree_with_open, clipper_clipper64_size, clipper_delete_clipper64,
    clipper_delete_path64, clipper_delete_paths64, clipper_delete_polytree64,
    clipper_path64_get_point, clipper_path64_length, clipper_path64_of_points, clipper_path64_size,
    clipper_paths64, clipper_paths64_of_paths, clipper_paths64_size, clipper_polytree64,
    clipper_polytree64_count, clipper_polytree64_get_child, clipper_polytree64_is_hole,
    clipper_polytree64_polygon, clipper_polytree64_size, ClipperClipType,
    ClipperClipType_INTERSECTION, ClipperClipType_UNION, ClipperFillRule_NON_ZERO, ClipperPoint64,
};
use std::ptr;

type IntPoint = (i64, i64);

/// gdstk uses `scaling = 1 / precision` for integer coordinates.
pub fn scaling_from_precision(precision: f64) -> f64 {
    if precision > 0.0 {
        1.0 / precision
    } else {
        1e3
    }
}

fn polygon_to_int_path(poly: &[(f64, f64)], scaling: f64) -> Vec<IntPoint> {
    let reverse = polygon_signed_area(poly) < 0.0;
    let n = poly.len();
    let mut out = Vec::with_capacity(n);
    if reverse {
        for p in poly.iter().rev() {
            out.push((p.0.mul_add(scaling, 0.0).round() as i64, p.1.mul_add(scaling, 0.0).round() as i64));
        }
    } else {
        for p in poly {
            out.push((p.0.mul_add(scaling, 0.0).round() as i64, p.1.mul_add(scaling, 0.0).round() as i64));
        }
    }
    out
}

fn int_path_to_polygon(path: &[IntPoint], inv_scaling: f64) -> Vec<(f64, f64)> {
    path
        .iter()
        .map(|&(x, y)| (x as f64 * inv_scaling, y as f64 * inv_scaling))
        .collect()
}

unsafe fn path64_from_points(points: &[IntPoint]) -> *mut clipper2c_sys::ClipperPath64 { unsafe {
    let mut pts: Vec<ClipperPoint64> = points
        .iter()
        .map(|&(x, y)| ClipperPoint64 { x, y })
        .collect();
    let mem = clipper_allocate(clipper_path64_size());
    clipper_path64_of_points(mem, pts.as_mut_ptr(), pts.len())
}}

unsafe fn paths64_from_polygons(polys: &[Vec<IntPoint>]) -> *mut clipper2c_sys::ClipperPaths64 { unsafe {
    let mut path_ptrs: Vec<*mut clipper2c_sys::ClipperPath64> = Vec::with_capacity(polys.len());
    for poly in polys {
        path_ptrs.push(path64_from_points(poly));
    }
    let mem = clipper_allocate(clipper_paths64_size());
    clipper_paths64_of_paths(mem, path_ptrs.as_mut_ptr(), path_ptrs.len())
}}

unsafe fn extract_int_path(node: *mut clipper2c_sys::ClipperPolyTree64) -> Vec<IntPoint> { unsafe {
    let mem = clipper_allocate(clipper_path64_size());
    let path = clipper_polytree64_polygon(mem, node);
    let len = clipper_path64_length(path);
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let pt = clipper_path64_get_point(path, i as i32);
        out.push((pt.x, pt.y));
    }
    clipper_delete_path64(path);
    out
}}

fn point_less(p1: IntPoint, p2: IntPoint) -> bool {
    p1.0 < p2.0 || (p1.0 == p2.0 && p1.1 < p2.1)
}

struct SortingHole {
    path: Vec<IntPoint>,
    min_idx: usize,
}

/// Port of gdstk `link_holes` (Angus Johnson Clipper PolyTree hole linking).
fn link_holes(contour: &mut Vec<IntPoint>, holes: &mut [SortingHole]) {
    holes.sort_by(|a, b| {
        let pa = a.path[a.min_idx];
        let pb = b.path[b.min_idx];
        if point_less(pa, pb) {
            std::cmp::Ordering::Less
        } else if point_less(pb, pa) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });

    for hole in holes.iter() {
        let hole_min = hole.path[hole.min_idx];
        let n = contour.len();
        if n < 2 {
            continue;
        }

        let mut p_closest: Option<usize> = None;
        let mut xnew: i64 = 0;

        for i in 0..n {
            let p_next = contour[i];
            let p_prev = contour[(i + n - 1) % n];

            if (p_next.1 <= hole_min.1 && hole_min.1 < p_prev.1)
                || (p_prev.1 < hole_min.1 && hole_min.1 <= p_next.1)
            {
                let temp = (p_prev.0 - p_next.0) as f64 * (hole_min.1 - p_next.1) as f64
                    / (p_prev.1 - p_next.1) as f64;
                let x = p_next.0 + temp.round() as i64;
                if (x > xnew || p_closest.is_none()) && x <= hole_min.0 {
                    xnew = x;
                    p_closest = Some(i);
                }
            } else if p_next.1 == hole_min.1
                && p_prev.1 == hole_min.1
                && ((p_next.0 <= hole_min.0 && hole_min.0 <= p_prev.0)
                    || (p_prev.0 <= hole_min.0 && hole_min.0 <= p_next.0))
            {
                xnew = hole_min.0;
                p_closest = Some(i);
                break;
            }
        }

        let Some(idx) = p_closest else {
            continue;
        };

        let p_new = (xnew, hole_min.1);
        let insert_at = idx;
        if contour[insert_at] != p_new {
            contour.insert(insert_at, p_new);
        }
        let h = &hole.path;
        let min_i = hole.min_idx;
        // Both parts insert before `insert_at` (gdstk: before p_closest / p_new).
        let part1: Vec<IntPoint> = h[..=min_i].to_vec();
        contour.splice(insert_at..insert_at, part1.iter().copied());
        let part2: Vec<IntPoint> = h[min_i..].to_vec();
        contour.splice(insert_at..insert_at, part2.iter().copied());
        contour.insert(insert_at, p_new);
    }
}

unsafe fn visit_shell(
    node: *mut clipper2c_sys::ClipperPolyTree64,
    _inv_scaling: f64,
    out: &mut Vec<Vec<IntPoint>>,
) { unsafe {
    if node.is_null() {
        return;
    }
    if clipper_polytree64_is_hole(node) != 0 {
        let n = clipper_polytree64_count(node);
        for i in 0..n {
            let child = clipper_polytree64_get_child(node, i) as *mut clipper2c_sys::ClipperPolyTree64;
            visit_shell(child, 1.0, out);
        }
        return;
    }

    let mut contour = extract_int_path(node);
    let child_count = clipper_polytree64_count(node);
    if child_count > 0 {
        let mut holes: Vec<SortingHole> = Vec::new();
        for i in 0..child_count {
            let child = clipper_polytree64_get_child(node, i) as *mut clipper2c_sys::ClipperPolyTree64;
            if clipper_polytree64_is_hole(child) == 0 {
                continue;
            }
            let path = extract_int_path(child);
            let mut min_idx = 0usize;
            for (i, p) in path.iter().enumerate().skip(1) {
                if point_less(*p, path[min_idx]) {
                    min_idx = i;
                }
            }
            holes.push(SortingHole { path, min_idx });
        }
        if !holes.is_empty() {
            link_holes(&mut contour, &mut holes);
        }
    }
    if contour.len() >= 3 {
        out.push(contour);
    }

    for i in 0..child_count {
        let child = clipper_polytree64_get_child(node, i) as *mut clipper2c_sys::ClipperPolyTree64;
        if clipper_polytree64_is_hole(child) != 0 {
            let n = clipper_polytree64_count(child);
            for j in 0..n {
                let grand = clipper_polytree64_get_child(child, j)
                    as *mut clipper2c_sys::ClipperPolyTree64;
                visit_shell(grand, 1.0, out);
            }
        }
    }
}}

unsafe fn tree_to_polygons(
    root: *mut clipper2c_sys::ClipperPolyTree64,
    _inv_scaling: f64,
) -> Vec<Vec<IntPoint>> { unsafe {
    let mut out = Vec::new();
    let n = clipper_polytree64_count(root);
    if n == 0 {
        visit_shell(root, 1.0, &mut out);
    } else {
        for i in 0..n {
            let child =
                clipper_polytree64_get_child(root, i) as *mut clipper2c_sys::ClipperPolyTree64;
            visit_shell(child, 1.0, &mut out);
        }
    }
    out
}}

unsafe fn execute_boolean_tree(
    subjects: &[Vec<IntPoint>],
    clips: &[Vec<IntPoint>],
    clip_type: ClipperClipType,
) -> Result<Vec<Vec<IntPoint>>, ()> { unsafe {
    if subjects.is_empty() {
        return Ok(Vec::new());
    }

    let subj_ptr = paths64_from_polygons(subjects);
    let clip_ptr = if clips.is_empty() {
        ptr::null_mut()
    } else {
        paths64_from_polygons(clips)
    };

    let clipper_mem = clipper_allocate(clipper_clipper64_size());
    let clipper = clipper_clipper64(clipper_mem);
    clipper_clipper64_add_subject(clipper, subj_ptr);
    if !clip_ptr.is_null() {
        clipper_clipper64_add_clip(clipper, clip_ptr);
    }

    let tree_mem = clipper_allocate(clipper_polytree64_size());
    let tree = clipper_polytree64(tree_mem, ptr::null_mut());
    let open_mem = clipper_allocate(clipper_paths64_size());
    let open = clipper_paths64(open_mem);

    let ok = clipper_clipper64_execute_tree_with_open(
        clipper,
        clip_type,
        ClipperFillRule_NON_ZERO,
        tree,
        open,
    );

    let mut result = Vec::new();
    if ok == 1 {
        result = tree_to_polygons(tree, 1.0);
    }

    clipper_delete_paths64(open);
    clipper_delete_polytree64(tree);
    clipper_delete_clipper64(clipper);
    clipper_delete_paths64(subj_ptr);
    if !clip_ptr.is_null() {
        clipper_delete_paths64(clip_ptr);
    }

    if ok == 1 {
        Ok(result)
    } else {
        Err(())
    }
}}

fn union_operand_int(polys: &[Vec<(f64, f64)>], scaling: f64) -> Vec<Vec<IntPoint>> {
    if polys.is_empty() {
        return Vec::new();
    }
    if polys.len() == 1 {
        return vec![polygon_to_int_path(&polys[0], scaling)];
    }
    let paths: Vec<Vec<IntPoint>> = polys
        .iter()
        .map(|p| polygon_to_int_path(p, scaling))
        .collect();
    unsafe {
        let merged = execute_boolean_tree(&paths, &[], ClipperClipType_UNION).unwrap_or(paths);
        merged
    }
}

pub fn boolean_polytree(
    operand_a: &[Vec<(f64, f64)>],
    operand_b: &[Vec<(f64, f64)>],
    clip_type: ClipperClipType,
    precision: f64,
) -> Vec<Vec<(f64, f64)>> {
    let scaling = scaling_from_precision(precision);
    let inv = 1.0 / scaling;
    let subjects = union_operand_int(operand_a, scaling);
    let clips = union_operand_int(operand_b, scaling);
    let int_result =
        unsafe { execute_boolean_tree(&subjects, &clips, clip_type).unwrap_or_default() };
    int_result
        .into_iter()
        .map(|p| int_path_to_polygon(&p, inv))
        .collect()
}

pub fn slice_polytree(
    polygons: &[Vec<(f64, f64)>],
    positions: &[f64],
    x_axis: bool,
    precision: f64,
) -> Vec<Vec<Vec<(f64, f64)>>> {
    let scaling = scaling_from_precision(precision);
    let inv = 1.0 / scaling;
    let mut results: Vec<Vec<Vec<(f64, f64)>>> = vec![Vec::new(); positions.len() + 1];

    for poly in polygons {
        let subj = vec![polygon_to_int_path(poly, scaling)];
        if subj[0].is_empty() {
            continue;
        }
        let bb = bounding_box_int(&subj[0]);
        let mut clip = vec![
            (bb[0], bb[2]),
            (bb[1], bb[2]),
            (bb[1], bb[3]),
            (bb[0], bb[3]),
        ];
        let mut pos = if x_axis { bb[0] } else { bb[2] };

        for (i, result_slot) in results.iter_mut().enumerate() {
            if x_axis {
                clip[0].0 = pos;
                clip[3].0 = pos;
                pos = if i < positions.len() {
                    (positions[i] * scaling).round() as i64
                } else {
                    bb[1]
                };
                clip[1].0 = pos;
                clip[2].0 = pos;
                if clip[1].0 == clip[0].0 {
                    continue;
                }
            } else {
                clip[0].1 = pos;
                clip[1].1 = pos;
                pos = if i < positions.len() {
                    (positions[i] * scaling).round() as i64
                } else {
                    bb[3]
                };
                clip[2].1 = pos;
                clip[3].1 = pos;
                if clip[2].1 == clip[0].1 {
                    continue;
                }
            }

            let clips = vec![clip.clone()];
            if let Ok(polys) =
                unsafe { execute_boolean_tree(&subj, &clips, ClipperClipType_INTERSECTION) }
            {
                for p in polys {
                    if p.len() >= 3 {
                        result_slot.push(int_path_to_polygon(&p, inv));
                    }
                }
            }
        }
    }

    results
}

fn bounding_box_int(path: &[IntPoint]) -> [i64; 4] {
    let mut bb = [path[0].0, path[0].0, path[0].1, path[0].1];
    for &(x, y) in path.iter().skip(1) {
        if x < bb[0] {
            bb[0] = x;
        }
        if x > bb[1] {
            bb[1] = x;
        }
        if y < bb[2] {
            bb[2] = y;
        }
        if y > bb[3] {
            bb[3] = y;
        }
    }
    bb
}
