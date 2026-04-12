// Geometric operations for EDA IC layout
// Provides bounding boxes, polygon transforms, area/perimeter, and point-in-polygon tests
// Mirrors the geometric API of gdstk

use crate::gdsii::{GDSElement, GDSIIFile};
use crate::oasis::OASISElement;

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
}

impl BoundingBox {
    /// Create a new bounding box
    pub fn new(x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        BoundingBox { x_min, y_min, x_max, y_max }
    }

    /// Bounding box width
    pub fn width(&self) -> f64 {
        self.x_max - self.x_min
    }

    /// Bounding box height
    pub fn height(&self) -> f64 {
        self.y_max - self.y_min
    }

    /// Bounding box area
    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }

    /// Center of the bounding box
    pub fn center(&self) -> (f64, f64) {
        ((self.x_min + self.x_max) / 2.0, (self.y_min + self.y_max) / 2.0)
    }

    /// Compute the union of two bounding boxes
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            x_min: self.x_min.min(other.x_min),
            y_min: self.y_min.min(other.y_min),
            x_max: self.x_max.max(other.x_max),
            y_max: self.y_max.max(other.y_max),
        }
    }

    /// Compute the intersection of two bounding boxes, returns None if they don't overlap
    pub fn intersect(&self, other: &BoundingBox) -> Option<BoundingBox> {
        let x_min = self.x_min.max(other.x_min);
        let y_min = self.y_min.max(other.y_min);
        let x_max = self.x_max.min(other.x_max);
        let y_max = self.y_max.min(other.y_max);
        if x_min <= x_max && y_min <= y_max {
            Some(BoundingBox { x_min, y_min, x_max, y_max })
        } else {
            None
        }
    }

    /// Test if a point is inside the bounding box
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x_min && x <= self.x_max && y >= self.y_min && y <= self.y_max
    }

    /// Test if this bounding box overlaps with another
    pub fn overlaps(&self, other: &BoundingBox) -> bool {
        self.intersect(other).is_some()
    }

    /// Expand the bounding box by a margin on all sides
    pub fn expand(&self, margin: f64) -> BoundingBox {
        BoundingBox {
            x_min: self.x_min - margin,
            y_min: self.y_min - margin,
            x_max: self.x_max + margin,
            y_max: self.y_max + margin,
        }
    }

    /// Build a bounding box from a sequence of points
    pub fn from_points(points: &[(f64, f64)]) -> Option<BoundingBox> {
        if points.is_empty() {
            return None;
        }
        let mut x_min = points[0].0;
        let mut y_min = points[0].1;
        let mut x_max = points[0].0;
        let mut y_max = points[0].1;
        for &(x, y) in &points[1..] {
            x_min = x_min.min(x);
            y_min = y_min.min(y);
            x_max = x_max.max(x);
            y_max = y_max.max(y);
        }
        Some(BoundingBox { x_min, y_min, x_max, y_max })
    }

    /// Convert to polygon (4 corners, closed)
    pub fn to_polygon(&self) -> Vec<(f64, f64)> {
        vec![
            (self.x_min, self.y_min),
            (self.x_max, self.y_min),
            (self.x_max, self.y_max),
            (self.x_min, self.y_max),
            (self.x_min, self.y_min),
        ]
    }
}

// ============================================================================
// Polygon area and perimeter (shoelace formula)
// ============================================================================

/// Compute the signed area of a polygon using the shoelace formula.
/// Positive = counter-clockwise, negative = clockwise.
pub fn polygon_signed_area(points: &[(f64, f64)]) -> f64 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].0 * points[j].1;
        area -= points[j].0 * points[i].1;
    }
    area / 2.0
}

/// Compute the absolute area of a polygon
pub fn polygon_area(points: &[(f64, f64)]) -> f64 {
    polygon_signed_area(points).abs()
}

/// Compute the perimeter of a polygon
pub fn polygon_perimeter(points: &[(f64, f64)]) -> f64 {
    let n = points.len();
    if n < 2 {
        return 0.0;
    }
    let mut perimeter = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        let dx = points[j].0 - points[i].0;
        let dy = points[j].1 - points[i].1;
        perimeter += (dx * dx + dy * dy).sqrt();
    }
    perimeter
}

/// Compute the centroid of a polygon
pub fn polygon_centroid(points: &[(f64, f64)]) -> (f64, f64) {
    let n = points.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let area = polygon_signed_area(points);
    if area.abs() < 1e-15 {
        // Degenerate polygon, return mean of vertices
        let cx = points.iter().map(|p| p.0).sum::<f64>() / n as f64;
        let cy = points.iter().map(|p| p.1).sum::<f64>() / n as f64;
        return (cx, cy);
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        let cross = points[i].0 * points[j].1 - points[j].0 * points[i].1;
        cx += (points[i].0 + points[j].0) * cross;
        cy += (points[i].1 + points[j].1) * cross;
    }
    let factor = 1.0 / (6.0 * area);
    (cx * factor, cy * factor)
}

// ============================================================================
// Bounding box for polygon
// ============================================================================

/// Compute bounding box of a polygon (f64 coordinates)
pub fn bounding_box(points: &[(f64, f64)]) -> Option<BoundingBox> {
    BoundingBox::from_points(points)
}

/// Compute bounding box of an integer-coordinate polygon (GDSII style)
pub fn bounding_box_i32(points: &[(i32, i32)]) -> Option<BoundingBox> {
    if points.is_empty() {
        return None;
    }
    let pts: Vec<(f64, f64)> = points.iter().map(|&(x, y)| (x as f64, y as f64)).collect();
    bounding_box(&pts)
}

/// Compute bounding box for a GDSII element
pub fn gds_element_bounding_box(element: &GDSElement) -> Option<BoundingBox> {
    match element {
        GDSElement::Boundary(b) => bounding_box_i32(&b.xy),
        GDSElement::Path(p) => {
            let half_w = p.width.unwrap_or(0) as f64 / 2.0;
            let bb = bounding_box_i32(&p.xy)?;
            Some(bb.expand(half_w))
        }
        GDSElement::Text(t) => Some(BoundingBox::new(
            t.xy.0 as f64, t.xy.1 as f64, t.xy.0 as f64, t.xy.1 as f64,
        )),
        GDSElement::StructRef(s) => Some(BoundingBox::new(
            s.xy.0 as f64, s.xy.1 as f64, s.xy.0 as f64, s.xy.1 as f64,
        )),
        GDSElement::ArrayRef(a) => {
            if a.xy.len() >= 3 {
                bounding_box_i32(&a.xy)
            } else {
                None
            }
        }
        GDSElement::Node(n) => bounding_box_i32(&n.xy),
        GDSElement::Box(b) => bounding_box_i32(&b.xy),
    }
}

/// Compute the bounding box of a structure (all elements combined)
pub fn structure_bounding_box(structure: &crate::gdsii::GDSStructure) -> Option<BoundingBox> {
    let mut result: Option<BoundingBox> = None;
    for element in &structure.elements {
        if let Some(bb) = gds_element_bounding_box(element) {
            result = Some(match result {
                None => bb,
                Some(r) => r.union(&bb),
            });
        }
    }
    result
}

/// Compute the bounding box of an entire GDSII library
pub fn library_bounding_box(library: &GDSIIFile) -> Option<BoundingBox> {
    let mut result: Option<BoundingBox> = None;
    for structure in &library.structures {
        if let Some(bb) = structure_bounding_box(structure) {
            result = Some(match result {
                None => bb,
                Some(r) => r.union(&bb),
            });
        }
    }
    result
}

/// Compute bounding box for an OASIS element
pub fn oasis_element_bounding_box(element: &OASISElement) -> Option<BoundingBox> {
    match element {
        OASISElement::Rectangle(r) => Some(BoundingBox::new(
            r.x as f64, r.y as f64,
            r.x as f64 + r.width as f64, r.y as f64 + r.height as f64,
        )),
        OASISElement::Polygon(p) => {
            let pts: Vec<(f64, f64)> = p.points.iter()
                .map(|&(px, py)| (p.x as f64 + px as f64, p.y as f64 + py as f64))
                .collect();
            bounding_box(&pts)
        }
        OASISElement::Path(p) => {
            let pts: Vec<(f64, f64)> = p.points.iter()
                .map(|&(px, py)| (p.x as f64 + px as f64, p.y as f64 + py as f64))
                .collect();
            let bb = bounding_box(&pts)?;
            Some(bb.expand(p.half_width as f64))
        }
        OASISElement::Trapezoid(t) => Some(BoundingBox::new(
            t.x as f64, t.y as f64,
            t.x as f64 + t.width as f64, t.y as f64 + t.height as f64,
        )),
        OASISElement::CTrapezoid(ct) => Some(BoundingBox::new(
            ct.x as f64, ct.y as f64,
            ct.x as f64 + ct.width as f64, ct.y as f64 + ct.height as f64,
        )),
        OASISElement::Circle(c) => Some(BoundingBox::new(
            c.x as f64 - c.radius as f64, c.y as f64 - c.radius as f64,
            c.x as f64 + c.radius as f64, c.y as f64 + c.radius as f64,
        )),
        OASISElement::Text(t) => Some(BoundingBox::new(
            t.x as f64, t.y as f64, t.x as f64, t.y as f64,
        )),
        OASISElement::Placement(p) => Some(BoundingBox::new(
            p.x as f64, p.y as f64, p.x as f64, p.y as f64,
        )),
    }
}

// ============================================================================
// Point-in-polygon using ray casting (even-odd rule)
// ============================================================================

/// Test if a point is inside a polygon using the ray casting algorithm
pub fn point_in_polygon(point: (f64, f64), polygon: &[(f64, f64)]) -> bool {
    let (px, py) = point;
    let n = polygon.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Test if a point is inside any polygon in a list
pub fn point_in_any_polygon(point: (f64, f64), polygons: &[Vec<(f64, f64)>]) -> bool {
    polygons.iter().any(|poly| point_in_polygon(point, poly))
}

/// Test which of a set of points are inside a set of polygons
pub fn inside(points: &[(f64, f64)], polygons: &[Vec<(f64, f64)>]) -> Vec<bool> {
    points.iter().map(|&pt| point_in_any_polygon(pt, polygons)).collect()
}

// ============================================================================
// Polygon transformations
// ============================================================================

/// Translate polygon points by (dx, dy)
pub fn translate(points: &[(f64, f64)], dx: f64, dy: f64) -> Vec<(f64, f64)> {
    points.iter().map(|&(x, y)| (x + dx, y + dy)).collect()
}

/// Rotate polygon points by `angle` radians around center (cx, cy)
pub fn rotate(points: &[(f64, f64)], angle: f64, cx: f64, cy: f64) -> Vec<(f64, f64)> {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    points.iter().map(|&(x, y)| {
        let dx = x - cx;
        let dy = y - cy;
        (cx + dx * cos_a - dy * sin_a, cy + dx * sin_a + dy * cos_a)
    }).collect()
}

/// Scale polygon points by (sx, sy) around center (cx, cy)
pub fn scale(points: &[(f64, f64)], sx: f64, sy: f64, cx: f64, cy: f64) -> Vec<(f64, f64)> {
    points.iter().map(|&(x, y)| {
        (cx + (x - cx) * sx, cy + (y - cy) * sy)
    }).collect()
}

/// Mirror polygon points about a horizontal axis y = axis_y
pub fn mirror_x(points: &[(f64, f64)], axis_y: f64) -> Vec<(f64, f64)> {
    points.iter().map(|&(x, y)| (x, 2.0 * axis_y - y)).collect()
}

/// Mirror polygon points about a vertical axis x = axis_x
pub fn mirror_y(points: &[(f64, f64)], axis_x: f64) -> Vec<(f64, f64)> {
    points.iter().map(|&(x, y)| (2.0 * axis_x - x, y)).collect()
}

/// Apply a full affine transform: translate, rotate, scale, and optionally mirror
pub fn affine_transform(
    points: &[(f64, f64)],
    translation: (f64, f64),
    rotation: f64,
    magnification: f64,
    x_reflection: bool,
) -> Vec<(f64, f64)> {
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();
    points.iter().map(|&(x, y)| {
        // Apply x-reflection first (flip x)
        let (rx, ry) = if x_reflection { (-x, y) } else { (x, y) };
        // Scale
        let (sx, sy) = (rx * magnification, ry * magnification);
        // Rotate
        let tx = sx * cos_r - sy * sin_r + translation.0;
        let ty = sx * sin_r + sy * cos_r + translation.1;
        (tx, ty)
    }).collect()
}

// ============================================================================
// Winding number and orientation
// ============================================================================

/// Test if a polygon is counter-clockwise (positive area)
pub fn is_counter_clockwise(points: &[(f64, f64)]) -> bool {
    polygon_signed_area(points) > 0.0
}

/// Ensure a polygon is counter-clockwise, reversing if necessary
pub fn ensure_counter_clockwise(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if is_counter_clockwise(points) {
        points.to_vec()
    } else {
        points.iter().rev().cloned().collect()
    }
}

/// Ensure a polygon is clockwise, reversing if necessary
pub fn ensure_clockwise(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if !is_counter_clockwise(points) {
        points.to_vec()
    } else {
        points.iter().rev().cloned().collect()
    }
}

// ============================================================================
// Polygon utility functions
// ============================================================================

/// Remove duplicate consecutive vertices from a polygon
pub fn remove_duplicates(points: &[(f64, f64)], tolerance: f64) -> Vec<(f64, f64)> {
    if points.is_empty() {
        return Vec::new();
    }
    let tol2 = tolerance * tolerance;
    let mut result = vec![points[0]];
    for &p in &points[1..] {
        let last = *result.last().unwrap();
        let dx = p.0 - last.0;
        let dy = p.1 - last.1;
        if dx * dx + dy * dy > tol2 {
            result.push(p);
        }
    }
    result
}

/// Close a polygon by ensuring first and last points are equal
pub fn close_polygon(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut result = points.to_vec();
    if (result[0].0 - result.last().unwrap().0).abs() > 1e-12
        || (result[0].1 - result.last().unwrap().1).abs() > 1e-12
    {
        result.push(result[0]);
    }
    result
}

/// Compute the distance between two points
pub fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    (dx * dx + dy * dy).sqrt()
}

/// Fracture a polygon into rectangles (horizontal scan line decomposition)
/// Returns a list of rectangle polygons (as 4-point closed polygons)
pub fn fracture_to_rectangles(points: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    // Collect all unique y-coordinates and sort them
    let mut ys: Vec<f64> = points.iter().map(|p| p.1).collect();
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 1e-10);

    let mut rects = Vec::new();

    // For each horizontal band between consecutive y values
    for i in 0..ys.len().saturating_sub(1) {
        let y_lo = ys[i];
        let y_hi = ys[i + 1];
        let y_mid = (y_lo + y_hi) / 2.0;

        // Find x intersections of polygon edges with the midpoint horizontal line
        let mut x_crossings = Vec::new();
        let n = points.len();
        for j in 0..n {
            let k = (j + 1) % n;
            let (x1, y1) = points[j];
            let (x2, y2) = points[k];
            if (y1 <= y_mid && y2 > y_mid) || (y2 <= y_mid && y1 > y_mid) {
                let t = (y_mid - y1) / (y2 - y1);
                x_crossings.push(x1 + t * (x2 - x1));
            }
        }
        x_crossings.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Pair up the crossings to get horizontal slabs
        let mut idx = 0;
        while idx + 1 < x_crossings.len() {
            let x_lo = x_crossings[idx];
            let x_hi = x_crossings[idx + 1];
            if (x_hi - x_lo).abs() > 1e-10 {
                rects.push(vec![
                    (x_lo, y_lo),
                    (x_hi, y_lo),
                    (x_hi, y_hi),
                    (x_lo, y_hi),
                    (x_lo, y_lo),
                ]);
            }
            idx += 2;
        }
    }

    rects
}

/// Fillet (round) the corners of a polygon
/// `radius` is the fillet radius, `points_per_arc` is how many points per 90° arc
pub fn fillet(points: &[(f64, f64)], radius: f64, points_per_arc: usize) -> Vec<(f64, f64)> {
    let n = points.len();
    if n < 3 || radius <= 0.0 {
        return points.to_vec();
    }

    let ppa = points_per_arc.max(1);
    let mut result = Vec::new();

    for i in 0..n {
        let prev = points[(i + n - 1) % n];
        let curr = points[i];
        let next = points[(i + 1) % n];

        let v1 = (prev.0 - curr.0, prev.1 - curr.1);
        let v2 = (next.0 - curr.0, next.1 - curr.1);

        let len1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
        let len2 = (v2.0 * v2.0 + v2.1 * v2.1).sqrt();

        if len1 < 1e-10 || len2 < 1e-10 {
            result.push(curr);
            continue;
        }

        let u1 = (v1.0 / len1, v1.1 / len1);
        let u2 = (v2.0 / len2, v2.1 / len2);

        // Half-angle between vectors
        let dot = (u1.0 * u2.0 + u1.1 * u2.1).clamp(-1.0, 1.0);
        let half_angle = dot.acos() / 2.0;

        if half_angle.abs() < 1e-10 || (std::f64::consts::PI - half_angle * 2.0).abs() < 1e-10 {
            result.push(curr);
            continue;
        }

        let tan_half = half_angle.tan();
        let t = radius / tan_half.abs();

        let t1 = t.min(len1 / 2.0);
        let t2 = t.min(len2 / 2.0);

        let p1 = (curr.0 + u1.0 * t1, curr.1 + u1.1 * t1);
        let p2 = (curr.0 + u2.0 * t2, curr.1 + u2.1 * t2);

        // Arc center
        let cross = u1.0 * u2.1 - u1.1 * u2.0;
        let r_actual = t1 * half_angle.tan();

        // Normal pointing toward arc center
        let perp1 = if cross > 0.0 {
            (-u1.1, u1.0)
        } else {
            (u1.1, -u1.0)
        };

        let center = (p1.0 + perp1.0 * r_actual, p1.1 + perp1.1 * r_actual);

        // Compute start and end angles
        let a_start = (p1.1 - center.1).atan2(p1.0 - center.0);
        let a_end = (p2.1 - center.1).atan2(p2.0 - center.0);

        // Determine sweep direction
        let mut sweep = a_end - a_start;
        if cross > 0.0 && sweep > 0.0 {
            sweep -= 2.0 * std::f64::consts::PI;
        } else if cross < 0.0 && sweep < 0.0 {
            sweep += 2.0 * std::f64::consts::PI;
        }

        let angle_span = sweep.abs();
        let num_steps = ((angle_span / std::f64::consts::PI) * ppa as f64).ceil() as usize;
        let num_steps = num_steps.max(1);

        for step in 0..=num_steps {
            let frac = step as f64 / num_steps as f64;
            let angle = a_start + frac * sweep;
            result.push((center.0 + r_actual * angle.cos(), center.1 + r_actual * angle.sin()));
        }
    }

    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box_basic() {
        let bb = BoundingBox::new(0.0, 0.0, 10.0, 5.0);
        assert_eq!(bb.width(), 10.0);
        assert_eq!(bb.height(), 5.0);
        assert_eq!(bb.area(), 50.0);
        assert_eq!(bb.center(), (5.0, 2.5));
    }

    #[test]
    fn test_bounding_box_union() {
        let a = BoundingBox::new(0.0, 0.0, 5.0, 5.0);
        let b = BoundingBox::new(3.0, 3.0, 10.0, 10.0);
        let u = a.union(&b);
        assert_eq!(u.x_min, 0.0);
        assert_eq!(u.y_min, 0.0);
        assert_eq!(u.x_max, 10.0);
        assert_eq!(u.y_max, 10.0);
    }

    #[test]
    fn test_bounding_box_intersect() {
        let a = BoundingBox::new(0.0, 0.0, 6.0, 6.0);
        let b = BoundingBox::new(3.0, 3.0, 10.0, 10.0);
        let i = a.intersect(&b).unwrap();
        assert_eq!(i.x_min, 3.0);
        assert_eq!(i.y_min, 3.0);
        assert_eq!(i.x_max, 6.0);
        assert_eq!(i.y_max, 6.0);
    }

    #[test]
    fn test_bounding_box_no_intersect() {
        let a = BoundingBox::new(0.0, 0.0, 2.0, 2.0);
        let b = BoundingBox::new(5.0, 5.0, 10.0, 10.0);
        assert!(a.intersect(&b).is_none());
    }

    #[test]
    fn test_bounding_box_from_points() {
        let pts = vec![(1.0, 2.0), (5.0, -1.0), (3.0, 7.0)];
        let bb = BoundingBox::from_points(&pts).unwrap();
        assert_eq!(bb.x_min, 1.0);
        assert_eq!(bb.y_min, -1.0);
        assert_eq!(bb.x_max, 5.0);
        assert_eq!(bb.y_max, 7.0);
    }

    #[test]
    fn test_polygon_area() {
        // Unit square
        let square = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        assert!((polygon_area(&square) - 1.0).abs() < 1e-10);

        // Triangle with known area
        let triangle = vec![(0.0, 0.0), (4.0, 0.0), (2.0, 3.0)];
        assert!((polygon_area(&triangle) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_polygon_signed_area_orientation() {
        let ccw = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let cw: Vec<_> = ccw.iter().rev().cloned().collect();
        assert!(polygon_signed_area(&ccw) > 0.0);
        assert!(polygon_signed_area(&cw) < 0.0);
    }

    #[test]
    fn test_polygon_perimeter() {
        let square = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        assert!((polygon_perimeter(&square) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_in_polygon() {
        let square = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        assert!(point_in_polygon((5.0, 5.0), &square));
        assert!(!point_in_polygon((15.0, 5.0), &square));
        assert!(!point_in_polygon((-1.0, 5.0), &square));
    }

    #[test]
    fn test_point_in_polygon_concave() {
        // L-shape
        let l_shape = vec![
            (0.0, 0.0), (10.0, 0.0), (10.0, 5.0),
            (5.0, 5.0), (5.0, 10.0), (0.0, 10.0),
        ];
        assert!(point_in_polygon((2.0, 2.0), &l_shape));
        assert!(point_in_polygon((2.0, 7.0), &l_shape));
        assert!(!point_in_polygon((7.0, 7.0), &l_shape));
    }

    #[test]
    fn test_inside() {
        let square = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let points = vec![(5.0, 5.0), (15.0, 5.0), (2.0, 2.0)];
        let result = inside(&points, &[square]);
        assert_eq!(result, vec![true, false, true]);
    }

    #[test]
    fn test_translate() {
        let pts = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)];
        let translated = translate(&pts, 5.0, 3.0);
        assert_eq!(translated[0], (5.0, 3.0));
        assert_eq!(translated[1], (6.0, 3.0));
        assert_eq!(translated[2], (6.0, 4.0));
    }

    #[test]
    fn test_rotate() {
        use std::f64::consts::PI;
        let pts = vec![(1.0, 0.0)];
        let rotated = rotate(&pts, PI / 2.0, 0.0, 0.0);
        assert!((rotated[0].0 - 0.0).abs() < 1e-10);
        assert!((rotated[0].1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotate_180() {
        use std::f64::consts::PI;
        let pts = vec![(1.0, 0.0), (0.0, 1.0)];
        let rotated = rotate(&pts, PI, 0.0, 0.0);
        assert!((rotated[0].0 - (-1.0)).abs() < 1e-10);
        assert!((rotated[0].1 - 0.0).abs() < 1e-10);
        assert!((rotated[1].0 - 0.0).abs() < 1e-10);
        assert!((rotated[1].1 - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_scale() {
        let pts = vec![(1.0, 2.0), (3.0, 4.0)];
        let scaled = scale(&pts, 2.0, 3.0, 0.0, 0.0);
        assert_eq!(scaled[0], (2.0, 6.0));
        assert_eq!(scaled[1], (6.0, 12.0));
    }

    #[test]
    fn test_mirror_x() {
        let pts = vec![(0.0, 3.0), (1.0, 5.0)];
        let mirrored = mirror_x(&pts, 0.0);
        assert_eq!(mirrored[0], (0.0, -3.0));
        assert_eq!(mirrored[1], (1.0, -5.0));
    }

    #[test]
    fn test_mirror_y() {
        let pts = vec![(3.0, 0.0), (5.0, 1.0)];
        let mirrored = mirror_y(&pts, 0.0);
        assert_eq!(mirrored[0], (-3.0, 0.0));
        assert_eq!(mirrored[1], (-5.0, 1.0));
    }

    #[test]
    fn test_ensure_counter_clockwise() {
        let cw = vec![(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0)];
        let ccw = ensure_counter_clockwise(&cw);
        assert!(is_counter_clockwise(&ccw));
    }

    #[test]
    fn test_affine_transform_translation() {
        let pts = vec![(0.0, 0.0), (1.0, 0.0)];
        let result = affine_transform(&pts, (5.0, 3.0), 0.0, 1.0, false);
        assert!((result[0].0 - 5.0).abs() < 1e-10);
        assert!((result[0].1 - 3.0).abs() < 1e-10);
        assert!((result[1].0 - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_affine_transform_magnification() {
        let pts = vec![(1.0, 1.0)];
        let result = affine_transform(&pts, (0.0, 0.0), 0.0, 2.0, false);
        assert!((result[0].0 - 2.0).abs() < 1e-10);
        assert!((result[0].1 - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_affine_transform_reflection() {
        let pts = vec![(1.0, 1.0)];
        let result = affine_transform(&pts, (0.0, 0.0), 0.0, 1.0, true);
        assert!((result[0].0 - (-1.0)).abs() < 1e-10);
        assert!((result[0].1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_gds_element_bounding_box_boundary() {
        use crate::gdsii::Boundary;
        let boundary = GDSElement::Boundary(Boundary {
            layer: 1,
            datatype: 0,
            xy: vec![(0, 0), (100, 0), (100, 50), (0, 50), (0, 0)],
            elflags: None,
            plex: None,
            properties: Vec::new(),
        });
        let bb = gds_element_bounding_box(&boundary).unwrap();
        assert_eq!(bb.x_min, 0.0);
        assert_eq!(bb.y_min, 0.0);
        assert_eq!(bb.x_max, 100.0);
        assert_eq!(bb.y_max, 50.0);
    }

    #[test]
    fn test_gds_element_bounding_box_path() {
        use crate::gdsii::GPath;
        let path = GDSElement::Path(GPath {
            layer: 1,
            datatype: 0,
            pathtype: 0,
            width: Some(10),
            bgnextn: None,
            endextn: None,
            xy: vec![(0, 0), (100, 0)],
            elflags: None,
            plex: None,
            properties: Vec::new(),
        });
        let bb = gds_element_bounding_box(&path).unwrap();
        assert_eq!(bb.x_min, -5.0); // half_width = 5
        assert_eq!(bb.x_max, 105.0);
    }

    #[test]
    fn test_structure_bounding_box() {
        use crate::gdsii::{Boundary, GDSStructure, GDSTime};
        let structure = GDSStructure {
            name: "TEST".to_string(),
            creation_time: GDSTime::now(),
            modification_time: GDSTime::now(),
            strclass: None,
            elements: vec![
                GDSElement::Boundary(Boundary {
                    layer: 1,
                    datatype: 0,
                    xy: vec![(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)],
                    elflags: None, plex: None, properties: Vec::new(),
                }),
                GDSElement::Boundary(Boundary {
                    layer: 2,
                    datatype: 0,
                    xy: vec![(-50, -50), (50, -50), (50, 50), (-50, 50), (-50, -50)],
                    elflags: None, plex: None, properties: Vec::new(),
                }),
            ],
        };
        let bb = structure_bounding_box(&structure).unwrap();
        assert_eq!(bb.x_min, -50.0);
        assert_eq!(bb.y_min, -50.0);
        assert_eq!(bb.x_max, 100.0);
        assert_eq!(bb.y_max, 100.0);
    }

    #[test]
    fn test_fracture_to_rectangles() {
        // Simple rectangle should come back as itself
        let rect = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 5.0), (0.0, 5.0)];
        let rects = fracture_to_rectangles(&rect);
        assert!(!rects.is_empty());
        // Total area should equal input area
        let total_area: f64 = rects.iter().map(|r| polygon_area(r)).sum();
        let input_area = polygon_area(&rect);
        assert!((total_area - input_area).abs() < 1e-6);
    }

    #[test]
    fn test_remove_duplicates() {
        let pts = vec![(0.0, 0.0), (0.0, 0.0), (1.0, 0.0), (1.0, 1.0)];
        let cleaned = remove_duplicates(&pts, 1e-10);
        assert_eq!(cleaned.len(), 3);
    }

    #[test]
    fn test_close_polygon() {
        let pts = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)];
        let closed = close_polygon(&pts);
        assert_eq!(closed.len(), 4);
        assert_eq!(closed[0], closed[closed.len() - 1]);
    }

    #[test]
    fn test_polygon_centroid() {
        let square = vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)];
        let c = polygon_centroid(&square);
        assert!((c.0 - 1.0).abs() < 1e-10);
        assert!((c.1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_oasis_element_bounding_box_rectangle() {
        use crate::oasis::Rectangle;
        let rect = OASISElement::Rectangle(Rectangle {
            layer: 1, datatype: 0,
            x: 10, y: 20, width: 50, height: 30,
            repetition: None, properties: Vec::new(),
        });
        let bb = oasis_element_bounding_box(&rect).unwrap();
        assert_eq!(bb.x_min, 10.0);
        assert_eq!(bb.y_min, 20.0);
        assert_eq!(bb.x_max, 60.0);
        assert_eq!(bb.y_max, 50.0);
    }

    #[test]
    fn test_oasis_element_bounding_box_circle() {
        use crate::oasis::Circle;
        let circle = OASISElement::Circle(Circle {
            layer: 1, datatype: 0,
            x: 0, y: 0, radius: 10,
            repetition: None, properties: Vec::new(),
        });
        let bb = oasis_element_bounding_box(&circle).unwrap();
        assert_eq!(bb.x_min, -10.0);
        assert_eq!(bb.y_min, -10.0);
        assert_eq!(bb.x_max, 10.0);
        assert_eq!(bb.y_max, 10.0);
    }

    #[test]
    fn test_fillet_basic() {
        let square = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let filleted = fillet(&square, 2.0, 4);
        // Filleted polygon should have more points than original
        assert!(filleted.len() > square.len());
        // Area should be less (corners removed)
        let original_area = polygon_area(&square);
        let filleted_area = polygon_area(&filleted);
        assert!(filleted_area < original_area + 0.1);
    }
}
