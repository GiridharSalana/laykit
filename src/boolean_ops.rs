// Boolean polygon operations — Clipper2 PolyTree backend (gdstk-compatible).
//
// gdstk uses Clipper PolyTree + `link_holes` in `clipper_tools.cpp`. LayKit uses
// the same via `clipper2c-sys` (`clipper_clipper64_execute_tree_with_open`).

use crate::clipper_polytree::{boolean_polytree, scaling_from_precision, slice_polytree};
use clipper2::{EndType, JoinType, Milli, Paths, inflate, simplify};
use clipper2c_sys::{
    ClipperClipType_DIFFERENCE, ClipperClipType_INTERSECTION, ClipperClipType_UNION,
    ClipperClipType_XOR,
};

/// Polygon set: a list of polygons, each a list of (x, y) points.
type PolySet = Vec<Vec<(f64, f64)>>;
/// A pair of polygon sets (used by slice).
type PolySetPair = (PolySet, PolySet);

/// Default vertex rounding precision (gdstk default).
pub const DEFAULT_PRECISION: f64 = 1e-3;

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

/// Axis for slice operations (`X` = vertical cuts at constant x, like gdstk).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    /// Vertical cuts at constant x (gdstk `axis="x"`).
    X,
    /// Horizontal cuts at constant y (gdstk `axis="y"`).
    Y,
}

/// Perform boolean operations on two sets of polygons (gdstk-compatible, default precision).
pub fn boolean(
    operand_a: &[Vec<(f64, f64)>],
    operand_b: &[Vec<(f64, f64)>],
    op: BooleanOp,
) -> PolySet {
    boolean_with_precision(operand_a, operand_b, op, DEFAULT_PRECISION)
}

/// Perform boolean operations with explicit vertex precision (matches `gdstk.boolean`).
pub fn boolean_with_precision(
    operand_a: &[Vec<(f64, f64)>],
    operand_b: &[Vec<(f64, f64)>],
    op: BooleanOp,
    precision: f64,
) -> PolySet {
    let clip_type = match op {
        BooleanOp::Or => ClipperClipType_UNION,
        BooleanOp::And => ClipperClipType_INTERSECTION,
        BooleanOp::Not => ClipperClipType_DIFFERENCE,
        BooleanOp::Xor => ClipperClipType_XOR,
    };
    boolean_polytree(operand_a, operand_b, clip_type, precision)
}

/// Offset (expand or shrink) polygons — Clipper2 inflate + simplify (gdstk-style).
pub fn offset(polygons: &[Vec<(f64, f64)>], distance: f64, tolerance: f64) -> PolySet {
    offset_with_precision(polygons, distance, tolerance, DEFAULT_PRECISION)
}

/// Offset with gdstk default precision for vertex quantization.
pub fn offset_with_precision(
    polygons: &[Vec<(f64, f64)>],
    distance: f64,
    tolerance: f64,
    precision: f64,
) -> PolySet {
    if polygons.is_empty() {
        return Vec::new();
    }
    let _ = scaling_from_precision(precision);
    let paths: Paths<Milli> = polygons
        .iter()
        .cloned()
        .map(clipper2::Path::<Milli>::from)
        .collect::<Vec<_>>()
        .into();
    let inflated = inflate::<Milli>(paths, distance, JoinType::Round, EndType::Polygon, 2.0);
    let simplified = simplify::<Milli>(inflated, tolerance.max(1e-6), false);
    simplified.into()
}

/// Slice polygons at cut positions (gdstk-compatible PolyTree intersection strips).
pub fn slice(polygons: &[Vec<(f64, f64)>], position: f64, axis: Axis) -> PolySetPair {
    slice_with_precision(polygons, &[position], axis, DEFAULT_PRECISION)
}

/// Slice at multiple positions (gdstk `slice` with a list of cuts).
pub fn slice_at_positions(
    polygons: &[Vec<(f64, f64)>],
    positions: &[f64],
    axis: Axis,
) -> Vec<PolySet> {
    slice_at_positions_with_precision(polygons, positions, axis, DEFAULT_PRECISION)
}

/// Slice at multiple positions with explicit precision.
pub fn slice_at_positions_with_precision(
    polygons: &[Vec<(f64, f64)>],
    positions: &[f64],
    axis: Axis,
    precision: f64,
) -> Vec<PolySet> {
    let x_axis = axis == Axis::X;
    slice_polytree(polygons, positions, x_axis, precision)
}

/// Slice with explicit precision (matches `gdstk.slice`).
pub fn slice_with_precision(
    polygons: &[Vec<(f64, f64)>],
    positions: &[f64],
    axis: Axis,
    precision: f64,
) -> PolySetPair {
    let x_axis = axis == Axis::X;
    let strips = slice_polytree(polygons, positions, x_axis, precision);
    if strips.len() == 2 {
        return (strips[0].clone(), strips[1].clone());
    }
    if strips.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let below = strips[0].clone();
    let above = strips[strips.len() - 1].clone();
    (below, above)
}

/// Sutherland–Hodgman clip (retained for tests and simple two-polygon clip).
pub fn sutherland_hodgman(subject: &[(f64, f64)], clip: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if subject.is_empty() || clip.is_empty() {
        return Vec::new();
    }
    let result = boolean(&[subject.to_vec()], &[clip.to_vec()], BooleanOp::And);
    result.into_iter().next().unwrap_or_default()
}

pub fn polygon_intersection(a: &[(f64, f64)], b: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    boolean(&[a.to_vec()], &[b.to_vec()], BooleanOp::And)
}

pub fn polygon_union(a: &[(f64, f64)], b: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    boolean(&[a.to_vec()], &[b.to_vec()], BooleanOp::Or)
}

pub fn polygon_difference(a: &[(f64, f64)], b: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    boolean(&[a.to_vec()], &[b.to_vec()], BooleanOp::Not)
}

pub fn polygon_xor(a: &[(f64, f64)], b: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    boolean(&[a.to_vec()], &[b.to_vec()], BooleanOp::Xor)
}

/// Graham scan convex hull.
pub fn convex_hull(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut pts: Vec<(f64, f64)> = points.to_vec();
    pts.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap()
            .then(a.1.partial_cmp(&b.1).unwrap())
    });
    pts.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-12 && (a.1 - b.1).abs() < 1e-12);

    if pts.len() < 3 {
        return pts;
    }

    let cross = |o: (f64, f64), a: (f64, f64), b: (f64, f64)| {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    };

    let mut lower = Vec::new();
    for p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], *p) <= 0.0 {
            lower.pop();
        }
        lower.push(*p);
    }

    let mut upper = Vec::new();
    for p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], *p) <= 0.0 {
            upper.pop();
        }
        upper.push(*p);
    }

    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

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
        assert!(result.len() >= 4);
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
        let b = vec![rect(0.0, 0.0, 10.0, 10.0)];
        let result = boolean(&a, &b, BooleanOp::Not);
        let total_area: f64 = result
            .iter()
            .map(|p| crate::geometry::polygon_area(p))
            .sum();
        assert!(
            total_area < 1.0,
            "Expected empty difference, area {}",
            total_area
        );
    }

    #[test]
    fn test_boolean_not_hole() {
        let a = vec![rect(0.0, 0.0, 10.0, 10.0)];
        let b = vec![rect(2.0, 2.0, 6.0, 6.0)];
        let result = boolean(&a, &b, BooleanOp::Not);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 11);
        let area: f64 = crate::geometry::polygon_area(&result[0]);
        assert!((area - 64.0).abs() < 0.5, "expected area 64, got {area}");
    }

    #[test]
    fn test_slice_vertical() {
        let polys = vec![rect(0.0, 0.0, 10.0, 10.0)];
        let strips = slice_at_positions(&polys, &[5.0], Axis::X);
        assert_eq!(strips.len(), 2);
        assert!(!strips[0].is_empty());
        assert!(!strips[1].is_empty());
    }

    #[test]
    fn test_offset_expand() {
        let polys = vec![rect(0.0, 0.0, 10.0, 10.0)];
        let expanded = offset(&polys, 2.0, 0.01);
        assert!(!expanded.is_empty());
        let expanded_area: f64 = expanded
            .iter()
            .map(|p| crate::geometry::polygon_area(p))
            .sum();
        assert!(expanded_area > 100.0);
    }

    #[test]
    fn test_convex_hull() {
        let pts = vec![(0.0, 0.0), (5.0, 0.0), (5.0, 5.0), (0.0, 5.0), (2.5, 2.5)];
        let hull = convex_hull(&pts);
        assert_eq!(hull.len(), 4);
    }
}
