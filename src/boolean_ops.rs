// Boolean polygon operations for EDA layout
// Implements Sutherland-Hodgman clipping and general polygon boolean operations
// Mirrors the boolean operation API of gdstk

use crate::geometry::{point_in_polygon, polygon_signed_area};

/// Polygon set: a list of polygons, each a list of (x, y) points.
type PolySet = Vec<Vec<(f64, f64)>>;
/// A pair of polygon sets (used by slice).
type PolySetPair = (PolySet, PolySet);
/// A pair of point-lists (used internally by slice_polygon).
type PointPair = (Vec<(f64, f64)>, Vec<(f64, f64)>);

/// Boolean operation type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BooleanOp {
    /// Union (OR): regions in A or B
    Or,
    /// Intersection (AND): regions in both A and B
    And,
    /// Difference (NOT): regions in A but not B
    Not,
    /// Symmetric difference (XOR): regions in A or B but not both
    Xor,
}

/// Axis for slice operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    /// Horizontal cut (constant y)
    X,
    /// Vertical cut (constant x)
    Y,
}

// ============================================================================
// Sutherland-Hodgman polygon clipping
// ============================================================================

fn sh_clip_polygon_by_half_plane(
    polygon: &[(f64, f64)],
    a: (f64, f64),
    b: (f64, f64),
) -> Vec<(f64, f64)> {
    if polygon.is_empty() {
        return Vec::new();
    }

    // Compute signed distance: positive = inside (left of the directed edge a->b)
    let edge_sign =
        |p: (f64, f64)| -> f64 { (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0) };

    let line_intersect = |p: (f64, f64), q: (f64, f64)| -> (f64, f64) {
        let d1 = edge_sign(p);
        let d2 = edge_sign(q);
        let denom = d1 - d2;
        if denom.abs() < 1e-15 {
            return p;
        }
        let t = d1 / denom;
        (p.0 + t * (q.0 - p.0), p.1 + t * (q.1 - p.1))
    };

    let n = polygon.len();
    let mut output = Vec::new();

    for i in 0..n {
        let curr = polygon[i];
        let next = polygon[(i + 1) % n];

        let d_curr = edge_sign(curr);
        let d_next = edge_sign(next);

        if d_curr >= 0.0 {
            // curr is inside
            output.push(curr);
            if d_next < 0.0 {
                // crossing to outside
                output.push(line_intersect(curr, next));
            }
        } else if d_next >= 0.0 {
            // entering from outside
            output.push(line_intersect(curr, next));
        }
    }

    output
}

/// Clip polygon `subject` against convex polygon `clip` using Sutherland-Hodgman
pub fn sutherland_hodgman(subject: &[(f64, f64)], clip: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if subject.is_empty() || clip.is_empty() {
        return Vec::new();
    }

    let n_clip = clip.len();
    let mut output = subject.to_vec();

    for i in 0..n_clip {
        if output.is_empty() {
            break;
        }
        let a = clip[i];
        let b = clip[(i + 1) % n_clip];
        output = sh_clip_polygon_by_half_plane(&output, a, b);
    }

    output
}

// ============================================================================
// General polygon boolean operations using the even-odd / winding rule
// ============================================================================

/// Compute the intersection of two polygons (works for convex clip polygon via S-H)
pub fn polygon_intersection(a: &[(f64, f64)], b: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    let clipped = sutherland_hodgman(a, b);
    if clipped.len() < 3 {
        Vec::new()
    } else {
        vec![clipped]
    }
}

/// Compute the union of two simple polygons (A ∪ B)
/// Uses the inclusion-exclusion approach: returns merged areas
pub fn polygon_union(a: &[(f64, f64)], b: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    let intersection = sutherland_hodgman(a, b);

    if intersection.len() < 3 {
        // No intersection: return both polygons separately
        return vec![a.to_vec(), b.to_vec()];
    }

    // Check if one polygon contains the other
    let a_in_b = polygon_signed_area(a).abs() > 0.0 && a.iter().all(|&pt| point_in_polygon(pt, b));
    let b_in_a = polygon_signed_area(b).abs() > 0.0 && b.iter().all(|&pt| point_in_polygon(pt, a));

    if a_in_b {
        return vec![b.to_vec()];
    }
    if b_in_a {
        return vec![a.to_vec()];
    }

    // General case: merge boundary traversal
    // Compute the outer boundary using contour traversal
    let merged = merge_polygon_boundaries(a, b);
    if merged.len() < 3 {
        vec![a.to_vec(), b.to_vec()]
    } else {
        vec![merged]
    }
}

/// Compute the difference A - B (A minus B)
pub fn polygon_difference(a: &[(f64, f64)], b: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    let intersection = sutherland_hodgman(a, b);

    if intersection.len() < 3 {
        // No intersection: A remains unchanged
        return vec![a.to_vec()];
    }

    // Compare intersection area to A's area to determine if A is fully inside B.
    // Using area comparison avoids boundary-vertex failures from ray-casting.
    let area_a = polygon_signed_area(a).abs();
    let area_inter = polygon_signed_area(&intersection).abs();
    if area_a > 0.0 && (area_inter / area_a) > 0.999 {
        return Vec::new(); // A is entirely consumed by B
    }

    // Partial overlap: subtract B from A
    let result = subtract_polygon(a, b);
    if result.len() < 3 {
        Vec::new()
    } else {
        vec![result]
    }
}

/// Compute XOR of two polygons (regions in A or B but not both)
pub fn polygon_xor(a: &[(f64, f64)], b: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    let mut result = Vec::new();
    result.extend(polygon_difference(a, b));
    result.extend(polygon_difference(b, a));
    result
}

// ============================================================================
// High-level boolean operation on sets of polygons
// ============================================================================

/// Perform boolean operations on two sets of polygons.
/// Each set is a list of polygons (each polygon is a list of points).
pub fn boolean(
    operand_a: &[Vec<(f64, f64)>],
    operand_b: &[Vec<(f64, f64)>],
    op: BooleanOp,
) -> Vec<Vec<(f64, f64)>> {
    match op {
        BooleanOp::Or => boolean_union(operand_a, operand_b),
        BooleanOp::And => boolean_intersection(operand_a, operand_b),
        BooleanOp::Not => boolean_difference(operand_a, operand_b),
        BooleanOp::Xor => boolean_xor(operand_a, operand_b),
    }
}

fn boolean_union(a: &[Vec<(f64, f64)>], b: &[Vec<(f64, f64)>]) -> Vec<Vec<(f64, f64)>> {
    if a.is_empty() {
        return b.to_vec();
    }
    if b.is_empty() {
        return a.to_vec();
    }

    let mut result = a.to_vec();
    for poly_b in b {
        let mut merged_any = false;
        let mut new_result = Vec::new();

        for poly_a in &result {
            let inter = sutherland_hodgman(poly_a, poly_b);
            if inter.len() >= 3 {
                let merged = polygon_union(poly_a, poly_b);
                new_result.extend(merged);
                merged_any = true;
            } else {
                new_result.push(poly_a.clone());
            }
        }

        if !merged_any {
            new_result.push(poly_b.clone());
        }

        result = new_result;
    }

    result
}

fn boolean_intersection(a: &[Vec<(f64, f64)>], b: &[Vec<(f64, f64)>]) -> Vec<Vec<(f64, f64)>> {
    let mut result = Vec::new();
    for poly_a in a {
        for poly_b in b {
            result.extend(polygon_intersection(poly_a, poly_b));
        }
    }
    result
}

fn boolean_difference(a: &[Vec<(f64, f64)>], b: &[Vec<(f64, f64)>]) -> Vec<Vec<(f64, f64)>> {
    let mut result = a.to_vec();
    for poly_b in b {
        let mut new_result = Vec::new();
        for poly_a in &result {
            new_result.extend(polygon_difference(poly_a, poly_b));
        }
        result = new_result;
    }
    result
}

fn boolean_xor(a: &[Vec<(f64, f64)>], b: &[Vec<(f64, f64)>]) -> Vec<Vec<(f64, f64)>> {
    let mut result = Vec::new();
    result.extend(boolean_difference(a, b));
    result.extend(boolean_difference(b, a));
    result
}

// ============================================================================
// Polygon slicing
// ============================================================================

/// Slice polygons with a line, returning (below/left, above/right) halves.
/// For Axis::X, `position` is the y coordinate of a horizontal cut.
/// For Axis::Y, `position` is the x coordinate of a vertical cut.
pub fn slice(polygons: &[Vec<(f64, f64)>], position: f64, axis: Axis) -> PolySetPair {
    let mut below = Vec::new();
    let mut above = Vec::new();

    for polygon in polygons {
        let (b, a) = slice_polygon(polygon, position, axis);
        if b.len() >= 3 {
            below.push(b);
        }
        if a.len() >= 3 {
            above.push(a);
        }
    }

    (below, above)
}

fn slice_polygon(polygon: &[(f64, f64)], position: f64, axis: Axis) -> PointPair {
    let mut below = Vec::new();
    let mut above = Vec::new();
    let n = polygon.len();

    for i in 0..n {
        let curr = polygon[i];
        let next = polygon[(i + 1) % n];

        let coord = |p: (f64, f64)| match axis {
            Axis::X => p.1,
            Axis::Y => p.0,
        };

        let c_curr = coord(curr);
        let c_next = coord(next);

        if c_curr <= position {
            below.push(curr);
        } else {
            above.push(curr);
        }

        // Edge crosses the cut line
        if (c_curr < position && c_next > position) || (c_curr > position && c_next < position) {
            let t = (position - c_curr) / (c_next - c_curr);
            let ix = curr.0 + t * (next.0 - curr.0);
            let iy = curr.1 + t * (next.1 - curr.1);
            let intersection = (ix, iy);
            below.push(intersection);
            above.push(intersection);
        }
    }

    (below, above)
}

// ============================================================================
// Polygon offset (Minkowski sum approximation)
// ============================================================================

/// Offset (expand or shrink) polygons by a given distance.
/// Positive distance = expand, negative = shrink.
/// Uses vertex offsetting along inward normals.
pub fn offset(polygons: &[Vec<(f64, f64)>], distance: f64, tolerance: f64) -> Vec<Vec<(f64, f64)>> {
    polygons
        .iter()
        .filter_map(|poly| offset_polygon(poly, distance, tolerance))
        .collect()
}

fn offset_polygon(
    polygon: &[(f64, f64)],
    distance: f64,
    _tolerance: f64,
) -> Option<Vec<(f64, f64)>> {
    let n = polygon.len();
    if n < 3 {
        return None;
    }

    // Ensure polygon is CCW for consistent outward normal direction
    let signed_area = polygon_signed_area(polygon);
    let poly: Vec<(f64, f64)> = if signed_area < 0.0 {
        polygon.iter().rev().cloned().collect()
    } else {
        polygon.to_vec()
    };

    let mut result = Vec::new();

    for i in 0..n {
        let prev = poly[(i + n - 1) % n];
        let curr = poly[i];
        let next = poly[(i + 1) % n];

        // Edge directions (from prev to curr, and from curr to next)
        let e1 = (curr.0 - prev.0, curr.1 - prev.1);
        let e2 = (next.0 - curr.0, next.1 - curr.1);

        let len1 = (e1.0 * e1.0 + e1.1 * e1.1).sqrt();
        let len2 = (e2.0 * e2.0 + e2.1 * e2.1).sqrt();

        if len1 < 1e-12 || len2 < 1e-12 {
            result.push(curr);
            continue;
        }

        // For a CCW polygon, outward normals are the RIGHT perpendiculars of each edge direction
        // Right perp of (dx, dy) is (dy, -dx)
        let n1 = (e1.1 / len1, -e1.0 / len1); // outward normal of incoming edge
        let n2 = (e2.1 / len2, -e2.0 / len2); // outward normal of outgoing edge

        // Miter vector (bisector of outward normals)
        let miter = (n1.0 + n2.0, n1.1 + n2.1);
        let miter_len = (miter.0 * miter.0 + miter.1 * miter.1).sqrt();

        let offset_pt = if miter_len < 1e-12 {
            // Anti-parallel normals (180° turn): use either outward normal
            (curr.0 + n1.0 * distance, curr.1 + n1.1 * distance)
        } else {
            // Project outward distance onto miter direction
            let dot = n1.0 * miter.0 / miter_len + n1.1 * miter.1 / miter_len;
            let scale = if dot.abs() < 1e-10 {
                distance
            } else {
                distance / dot
            };
            // Clamp miter to avoid extremely long spikes (miter limit of 4x distance)
            let clamp_limit = distance.abs() * 4.0;
            let clamped_scale = scale.clamp(-clamp_limit, clamp_limit);
            (
                curr.0 + miter.0 / miter_len * clamped_scale,
                curr.1 + miter.1 / miter_len * clamped_scale,
            )
        };

        result.push(offset_pt);
    }

    if result.len() < 3 {
        None
    } else {
        Some(result)
    }
}

// ============================================================================
// Helper: merge two polygon boundaries (for union)
// This is a simplified convex-hull-based merge for overlapping polygons
// ============================================================================

fn merge_polygon_boundaries(a: &[(f64, f64)], b: &[(f64, f64)]) -> Vec<(f64, f64)> {
    // Combine all points from A outside B and B outside A, plus intersection points
    let mut all_pts: Vec<(f64, f64)> = Vec::new();

    // Points of A outside B
    for &p in a {
        if !point_in_polygon(p, b) {
            all_pts.push(p);
        }
    }
    // Points of B outside A
    for &p in b {
        if !point_in_polygon(p, a) {
            all_pts.push(p);
        }
    }
    // Edge intersection points
    all_pts.extend(edge_intersections(a, b));

    if all_pts.len() < 3 {
        return Vec::new();
    }

    // Return convex hull of all points as an approximation of the union
    convex_hull(&all_pts)
}

fn subtract_polygon(a: &[(f64, f64)], b: &[(f64, f64)]) -> Vec<(f64, f64)> {
    // Clip a, keeping parts outside b
    let mut result: Vec<(f64, f64)> = Vec::new();
    let n = a.len();

    for i in 0..n {
        let curr = a[i];
        let next = a[(i + 1) % n];

        let in_b_curr = point_in_polygon(curr, b);
        let _in_b_next = point_in_polygon(next, b);

        if !in_b_curr {
            result.push(curr);
        }

        // Add intersection points where edges cross boundary of B
        for j in 0..b.len() {
            let bp = b[j];
            let bq = b[(j + 1) % b.len()];
            if let Some(ix) = segment_intersect(curr, next, bp, bq) {
                result.push(ix);
            }
        }
    }

    result
}

fn edge_intersections(a: &[(f64, f64)], b: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut result = Vec::new();
    let na = a.len();
    let nb = b.len();
    for i in 0..na {
        let a0 = a[i];
        let a1 = a[(i + 1) % na];
        for j in 0..nb {
            let b0 = b[j];
            let b1 = b[(j + 1) % nb];
            if let Some(pt) = segment_intersect(a0, a1, b0, b1) {
                result.push(pt);
            }
        }
    }
    result
}

/// Test if two segments (a0,a1) and (b0,b1) intersect, returning the intersection point
fn segment_intersect(
    a0: (f64, f64),
    a1: (f64, f64),
    b0: (f64, f64),
    b1: (f64, f64),
) -> Option<(f64, f64)> {
    let dx_a = a1.0 - a0.0;
    let dy_a = a1.1 - a0.1;
    let dx_b = b1.0 - b0.0;
    let dy_b = b1.1 - b0.1;

    let denom = dx_a * dy_b - dy_a * dx_b;
    if denom.abs() < 1e-15 {
        return None; // Parallel
    }

    let dx_ab = b0.0 - a0.0;
    let dy_ab = b0.1 - a0.1;

    let t = (dx_ab * dy_b - dy_ab * dx_b) / denom;
    let u = (dx_ab * dy_a - dy_ab * dx_a) / denom;

    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some((a0.0 + t * dx_a, a0.1 + t * dy_a))
    } else {
        None
    }
}

// ============================================================================
// Convex hull (Graham scan)
// ============================================================================

/// Compute the convex hull of a set of points using the Graham scan algorithm
pub fn convex_hull(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.to_vec();
    }

    // Find the lowest (then leftmost) point
    let mut pts = points.to_vec();
    let pivot_idx = pts
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a.1.partial_cmp(&b.1)
                .unwrap()
                .then_with(|| a.0.partial_cmp(&b.0).unwrap())
        })
        .map(|(i, _)| i)
        .unwrap();

    pts.swap(0, pivot_idx);
    let pivot = pts[0];

    // Sort remaining points by polar angle relative to pivot
    pts[1..].sort_by(|a, b| {
        let cross = (a.0 - pivot.0) * (b.1 - pivot.1) - (a.1 - pivot.1) * (b.0 - pivot.0);
        if cross.abs() < 1e-12 {
            let da = (a.0 - pivot.0).hypot(a.1 - pivot.1);
            let db = (b.0 - pivot.0).hypot(b.1 - pivot.1);
            da.partial_cmp(&db).unwrap()
        } else if cross > 0.0 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    let mut hull = vec![pts[0], pts[1]];

    for &p in &pts[2..] {
        while hull.len() > 1 {
            let n = hull.len();
            let cross = (hull[n - 1].0 - hull[n - 2].0) * (p.1 - hull[n - 2].1)
                - (hull[n - 1].1 - hull[n - 2].1) * (p.0 - hull[n - 2].0);
            if cross <= 0.0 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(p);
    }

    hull
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, w: f64, h: f64) -> Vec<(f64, f64)> {
        vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
    }

    #[test]
    fn test_sutherland_hodgman_full_overlap() {
        let subject = rect(0.0, 0.0, 10.0, 10.0);
        let clip = rect(2.0, 2.0, 6.0, 6.0);
        let result = sutherland_hodgman(&subject, &clip);
        assert!(result.len() >= 4, "Expected at least 4 vertices");
        // Result should be the clip polygon
        let area: f64 = {
            let n = result.len();
            let mut a = 0.0;
            for i in 0..n {
                let j = (i + 1) % n;
                a += result[i].0 * result[j].1 - result[j].0 * result[i].1;
            }
            (a / 2.0).abs()
        };
        assert!((area - 36.0).abs() < 0.5, "Expected area ~36, got {}", area);
    }

    #[test]
    fn test_sutherland_hodgman_partial_overlap() {
        let subject = rect(0.0, 0.0, 6.0, 6.0);
        let clip = rect(3.0, 3.0, 6.0, 6.0);
        let result = sutherland_hodgman(&subject, &clip);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_sutherland_hodgman_no_overlap() {
        let subject = rect(0.0, 0.0, 2.0, 2.0);
        let clip = rect(5.0, 5.0, 2.0, 2.0);
        let result = sutherland_hodgman(&subject, &clip);
        assert!(result.is_empty());
    }

    #[test]
    fn test_boolean_and() {
        let a = vec![rect(0.0, 0.0, 6.0, 6.0)];
        let b = vec![rect(3.0, 3.0, 6.0, 6.0)];
        let result = boolean(&a, &b, BooleanOp::And);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_boolean_or_disjoint() {
        let a = vec![rect(0.0, 0.0, 2.0, 2.0)];
        let b = vec![rect(5.0, 5.0, 2.0, 2.0)];
        let result = boolean(&a, &b, BooleanOp::Or);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_boolean_not() {
        let a = vec![rect(0.0, 0.0, 10.0, 10.0)];
        let b = vec![rect(0.0, 0.0, 10.0, 10.0)]; // same polygon
        let result = boolean(&a, &b, BooleanOp::Not);
        // A minus itself should be empty or very small
        for poly in &result {
            let area: f64 = {
                let n = poly.len();
                let mut ar = 0.0;
                for i in 0..n {
                    let j = (i + 1) % n;
                    ar += poly[i].0 * poly[j].1 - poly[j].0 * poly[i].1;
                }
                (ar / 2.0).abs()
            };
            assert!(area < 1.0, "Expected tiny area, got {}", area);
        }
    }

    #[test]
    fn test_slice_horizontal() {
        let polys = vec![rect(0.0, 0.0, 10.0, 10.0)];
        let (below, above) = slice(&polys, 5.0, Axis::X);
        assert!(!below.is_empty());
        assert!(!above.is_empty());
    }

    #[test]
    fn test_slice_vertical() {
        let polys = vec![rect(0.0, 0.0, 10.0, 10.0)];
        let (left, right) = slice(&polys, 5.0, Axis::Y);
        assert!(!left.is_empty());
        assert!(!right.is_empty());
    }

    #[test]
    fn test_slice_outside() {
        let polys = vec![rect(0.0, 0.0, 10.0, 10.0)];
        let (below, above) = slice(&polys, 20.0, Axis::X);
        assert!(!below.is_empty());
        assert!(above.is_empty());
    }

    #[test]
    fn test_offset_expand() {
        let polys = vec![rect(0.0, 0.0, 10.0, 10.0)];
        let expanded = offset(&polys, 2.0, 0.01);
        assert!(!expanded.is_empty());
        // Expanded polygon should be larger
        let original_area = 100.0;
        let expanded_area: f64 = {
            let p = &expanded[0];
            let n = p.len();
            let mut a = 0.0;
            for i in 0..n {
                let j = (i + 1) % n;
                a += p[i].0 * p[j].1 - p[j].0 * p[i].1;
            }
            (a / 2.0).abs()
        };
        assert!(
            expanded_area > original_area,
            "Expanded area {} should be > {}",
            expanded_area,
            original_area
        );
    }

    #[test]
    fn test_offset_shrink() {
        let polys = vec![rect(0.0, 0.0, 10.0, 10.0)];
        let shrunk = offset(&polys, -1.0, 0.01);
        assert!(!shrunk.is_empty());
    }

    #[test]
    fn test_convex_hull() {
        let pts = vec![
            (0.0, 0.0),
            (5.0, 0.0),
            (5.0, 5.0),
            (0.0, 5.0),
            (2.5, 2.5), // interior point
        ];
        let hull = convex_hull(&pts);
        assert_eq!(hull.len(), 4); // Interior point should be excluded
    }

    #[test]
    fn test_convex_hull_triangle() {
        let pts = vec![(0.0, 0.0), (4.0, 0.0), (2.0, 3.0)];
        let hull = convex_hull(&pts);
        assert_eq!(hull.len(), 3);
    }

    #[test]
    fn test_segment_intersect() {
        let pt = segment_intersect((0.0, 0.0), (2.0, 2.0), (0.0, 2.0), (2.0, 0.0));
        assert!(pt.is_some());
        let (x, y) = pt.unwrap();
        assert!((x - 1.0).abs() < 1e-10);
        assert!((y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_segment_no_intersect() {
        let pt = segment_intersect((0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0));
        assert!(pt.is_none());
    }

    #[test]
    fn test_polygon_intersection_area() {
        let a = rect(0.0, 0.0, 4.0, 4.0);
        let b = rect(2.0, 2.0, 4.0, 4.0);
        let result = polygon_intersection(&a, &b);
        assert!(!result.is_empty());
        let inter_area: f64 = result
            .iter()
            .map(|p| crate::geometry::polygon_area(p))
            .sum();
        assert!(
            (inter_area - 4.0).abs() < 0.5,
            "Expected area ~4, got {}",
            inter_area
        );
    }

    #[test]
    fn test_boolean_multiple_polygons() {
        let a = vec![rect(0.0, 0.0, 5.0, 5.0), rect(10.0, 0.0, 5.0, 5.0)];
        let b = vec![rect(2.0, 2.0, 8.0, 8.0)];
        let result = boolean(&a, &b, BooleanOp::And);
        assert!(!result.is_empty());
    }
}
