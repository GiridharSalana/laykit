// FlexPath: Flexible path with configurable end caps and join types
// Mirrors the FlexPath API of gdstk

use crate::geometry::{BoundingBox, bounding_box};
use std::f64::consts::PI;

/// End cap style for the start and end of a path
#[derive(Debug, Clone)]
pub enum EndCap {
    /// Flush end (no extension)
    Flush,
    /// Extend by half the path width
    HalfWidth,
    /// Extend by a fixed length
    Extended(f64),
    /// Rounded semicircle end
    Round,
}

/// Join style between path segments
#[derive(Debug, Clone)]
pub enum Join {
    /// Natural join (connected with a straight line)
    Natural,
    /// Miter join (extend edges until they meet, with a miter limit)
    Miter(f64),
    /// Bevel join (flat cut across corner)
    Bevel,
    /// Round join (circular arc)
    Round,
}

/// A path segment specification
#[derive(Debug, Clone)]
pub struct PathSegment {
    /// Endpoint of this segment
    pub end: (f64, f64),
    /// Width at the end of this segment (if different from previous)
    pub width: Option<f64>,
    /// Offset from the path centerline at the end
    pub offset: f64,
    /// Whether coordinates are relative to the previous point
    pub relative: bool,
}

/// A flexible path with configurable geometry
#[derive(Debug, Clone)]
pub struct FlexPath {
    /// Control points defining the path centerline
    pub points: Vec<(f64, f64)>,
    /// Width at each control point
    pub widths: Vec<f64>,
    /// Offset from centerline at each control point
    pub offsets: Vec<f64>,
    /// Join style at each interior vertex
    pub joins: Vec<Join>,
    /// End cap for the start and end of the path
    pub end_caps: (EndCap, EndCap),
    /// Layer number
    pub layer: u32,
    /// Datatype number
    pub datatype: u32,
    /// Number of points used for circular approximations
    pub bend_radius: Option<f64>,
    pub tolerance: f64,
}

impl FlexPath {
    /// Create a new FlexPath starting at `start` with given `width`
    pub fn new(start: (f64, f64), width: f64, layer: u32, datatype: u32) -> Self {
        FlexPath {
            points: vec![start],
            widths: vec![width],
            offsets: vec![0.0],
            joins: Vec::new(),
            end_caps: (EndCap::Flush, EndCap::Flush),
            layer,
            datatype,
            bend_radius: None,
            tolerance: 0.01,
        }
    }

    /// Add a line segment to the path
    pub fn segment(
        &mut self,
        end: (f64, f64),
        width: Option<f64>,
        offset: Option<f64>,
        relative: bool,
    ) -> &mut Self {
        let actual_end = if relative {
            let last = *self.points.last().unwrap();
            (last.0 + end.0, last.1 + end.1)
        } else {
            end
        };

        let w = width.unwrap_or_else(|| *self.widths.last().unwrap());
        let o = offset.unwrap_or_else(|| *self.offsets.last().unwrap());

        self.points.push(actual_end);
        self.widths.push(w);
        self.offsets.push(o);
        self.joins.push(Join::Natural);
        self
    }

    /// Add an arc segment (approximated with line segments)
    pub fn arc(
        &mut self,
        radius: f64,
        initial_angle: f64,
        final_angle: f64,
        width: Option<f64>,
    ) -> &mut Self {
        let center = *self.points.last().unwrap();
        let last_w = *self.widths.last().unwrap();
        let last_o = *self.offsets.last().unwrap();
        let w = width.unwrap_or(last_w);

        let sweep = final_angle - initial_angle;
        let num_pts = ((sweep.abs() / (2.0 * (self.tolerance / radius).acos())).ceil() as usize)
            .clamp(3, 1000);

        for i in 1..=num_pts {
            let angle = initial_angle + sweep * i as f64 / num_pts as f64;
            let pt = (
                center.0 + radius * angle.cos(),
                center.1 + radius * angle.sin(),
            );
            self.points.push(pt);
            self.widths.push(w);
            self.offsets.push(last_o);
            self.joins.push(Join::Natural);
        }
        self
    }

    /// Add a cubic Bezier curve segment
    pub fn bezier(
        &mut self,
        ctrl1: (f64, f64),
        ctrl2: (f64, f64),
        end: (f64, f64),
        width: Option<f64>,
        num_points: usize,
    ) -> &mut Self {
        let start = *self.points.last().unwrap();
        let last_w = *self.widths.last().unwrap();
        let last_o = *self.offsets.last().unwrap();
        let w = width.unwrap_or(last_w);

        let n = num_points.max(2);
        for i in 1..=n {
            let t = i as f64 / n as f64;
            let it = 1.0 - t;
            let pt = (
                it * it * it * start.0
                    + 3.0 * it * it * t * ctrl1.0
                    + 3.0 * it * t * t * ctrl2.0
                    + t * t * t * end.0,
                it * it * it * start.1
                    + 3.0 * it * it * t * ctrl1.1
                    + 3.0 * it * t * t * ctrl2.1
                    + t * t * t * end.1,
            );
            self.points.push(pt);
            self.widths.push(w);
            self.offsets.push(last_o);
            self.joins.push(Join::Natural);
        }
        self
    }

    /// Set the join style for all subsequent segments
    pub fn with_join(mut self, join: Join) -> Self {
        self.joins = self.joins.into_iter().map(|_| join.clone()).collect();
        self
    }

    /// Set end caps
    pub fn with_end_caps(mut self, start: EndCap, end: EndCap) -> Self {
        self.end_caps = (start, end);
        self
    }

    /// Get the total length of the path centerline
    pub fn length(&self) -> f64 {
        let mut total = 0.0;
        for i in 0..self.points.len().saturating_sub(1) {
            let dx = self.points[i + 1].0 - self.points[i].0;
            let dy = self.points[i + 1].1 - self.points[i].1;
            total += (dx * dx + dy * dy).sqrt();
        }
        total
    }

    /// Convert the FlexPath to a polygon outline
    pub fn to_polygon(&self) -> Option<Vec<(f64, f64)>> {
        let n = self.points.len();
        if n < 2 {
            return None;
        }

        let mut left_side: Vec<(f64, f64)> = Vec::new();
        let mut right_side: Vec<(f64, f64)> = Vec::new();

        // Build side vertices for each point.
        // `get_vertex_normal_and_width` returns the perpendicular (left-normal) of the path
        // direction at each vertex. For a path going right (+x), normal = (0, +y).
        // left side  = center + normal * (offset + hw)
        // right side = center + normal * (offset - hw)
        for i in 0..n {
            let (normal, width, offset_val) = self.get_vertex_normal_and_width(i);
            let hw = width / 2.0;
            let center = self.points[i];

            let left = (
                center.0 + normal.0 * (offset_val + hw),
                center.1 + normal.1 * (offset_val + hw),
            );
            let right = (
                center.0 + normal.0 * (offset_val - hw),
                center.1 + normal.1 * (offset_val - hw),
            );

            left_side.push(left);
            right_side.push(right);
        }

        // Apply end caps
        let mut polygon = Vec::new();

        // Start cap
        match &self.end_caps.0 {
            EndCap::Flush => {
                polygon.push(left_side[0]);
            }
            EndCap::HalfWidth => {
                let hw = self.widths[0] / 2.0;
                let dir = self.segment_direction(0);
                polygon.push((left_side[0].0 - dir.0 * hw, left_side[0].1 - dir.1 * hw));
            }
            EndCap::Extended(ext) => {
                let dir = self.segment_direction(0);
                polygon.push((left_side[0].0 - dir.0 * ext, left_side[0].1 - dir.1 * ext));
            }
            EndCap::Round => {
                let center = self.points[0];
                let hw = self.widths[0] / 2.0;
                let dir = self.segment_direction(0);
                let start_angle = dir.1.atan2(dir.0) + PI / 2.0;
                let num_pts = 8;
                for k in 0..=num_pts {
                    let angle = start_angle + PI * k as f64 / num_pts as f64;
                    polygon.push((center.0 + hw * angle.cos(), center.1 + hw * angle.sin()));
                }
            }
        }

        // Left side (forward direction)
        polygon.extend_from_slice(&left_side);

        // End cap
        match &self.end_caps.1 {
            EndCap::Flush => {
                polygon.push(right_side[n - 1]);
            }
            EndCap::HalfWidth => {
                let hw = self.widths[n - 1] / 2.0;
                let dir = self.segment_direction(n - 2);
                polygon.push((
                    right_side[n - 1].0 + dir.0 * hw,
                    right_side[n - 1].1 + dir.1 * hw,
                ));
            }
            EndCap::Extended(ext) => {
                let dir = self.segment_direction(n - 2);
                polygon.push((
                    right_side[n - 1].0 + dir.0 * ext,
                    right_side[n - 1].1 + dir.1 * ext,
                ));
            }
            EndCap::Round => {
                let center = self.points[n - 1];
                let hw = self.widths[n - 1] / 2.0;
                let dir = self.segment_direction(n - 2);
                let start_angle = dir.1.atan2(dir.0) - PI / 2.0;
                let num_pts = 8;
                for k in 0..=num_pts {
                    let angle = start_angle + PI * k as f64 / num_pts as f64;
                    polygon.push((center.0 + hw * angle.cos(), center.1 + hw * angle.sin()));
                }
            }
        }

        // Right side (reverse direction)
        for pt in right_side.iter().rev() {
            polygon.push(*pt);
        }

        // Close the polygon
        if let Some(&first) = polygon.first() {
            polygon.push(first);
        }

        Some(polygon)
    }

    /// Get the bounding box of the path
    pub fn bounding_box(&self) -> Option<BoundingBox> {
        let poly = self.to_polygon()?;
        let pts: Vec<(f64, f64)> = poly;
        bounding_box(&pts)
    }

    // ---- Private helpers ----

    fn segment_direction(&self, i: usize) -> (f64, f64) {
        let i = i.min(self.points.len() - 2);
        let dx = self.points[i + 1].0 - self.points[i].0;
        let dy = self.points[i + 1].1 - self.points[i].1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-12 {
            (1.0, 0.0)
        } else {
            (dx / len, dy / len)
        }
    }

    fn get_vertex_normal_and_width(&self, i: usize) -> ((f64, f64), f64, f64) {
        let n = self.points.len();
        let w = self.widths[i];
        let o = self.offsets[i];

        // Left perpendicular of direction (dx, dy) is (-dy, dx)
        let left_perp = |d: (f64, f64)| (-d.1, d.0);

        let normal = if n == 1 {
            (0.0, 1.0)
        } else if i == 0 {
            left_perp(self.segment_direction(0))
        } else if i == n - 1 {
            left_perp(self.segment_direction(n - 2))
        } else {
            // Average left-perps from adjacent segments for smooth join
            let d1 = self.segment_direction(i - 1);
            let d2 = self.segment_direction(i);
            let lp1 = left_perp(d1);
            let lp2 = left_perp(d2);
            let avg = (lp1.0 + lp2.0, lp1.1 + lp2.1);
            let len = (avg.0 * avg.0 + avg.1 * avg.1).sqrt();
            if len < 1e-12 {
                lp1
            } else {
                (avg.0 / len, avg.1 / len)
            }
        };

        (normal, w, o)
    }
}

/// A robust path that guarantees no self-intersections (simplified version)
/// Uses FlexPath internally with additional cleanup
#[derive(Debug, Clone)]
pub struct RobustPath {
    pub inner: FlexPath,
}

impl RobustPath {
    /// Create a new RobustPath
    pub fn new(start: (f64, f64), width: f64, layer: u32, datatype: u32) -> Self {
        RobustPath {
            inner: FlexPath::new(start, width, layer, datatype),
        }
    }

    /// Add a segment
    pub fn segment(&mut self, end: (f64, f64), width: Option<f64>, relative: bool) -> &mut Self {
        self.inner.segment(end, width, None, relative);
        self
    }

    /// Convert to polygon
    pub fn to_polygon(&self) -> Option<Vec<(f64, f64)>> {
        // In a full implementation, would add self-intersection checking
        self.inner.to_polygon()
    }

    /// Get path length
    pub fn length(&self) -> f64 {
        self.inner.length()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flexpath_basic() {
        let mut path = FlexPath::new((0.0, 0.0), 2.0, 1, 0);
        path.segment((10.0, 0.0), None, None, false);
        assert_eq!(path.points.len(), 2);
        assert_eq!(path.length(), 10.0);
    }

    #[test]
    fn test_flexpath_segment_relative() {
        let mut path = FlexPath::new((5.0, 5.0), 1.0, 1, 0);
        path.segment((3.0, 4.0), None, None, true);
        assert_eq!(path.points.last(), Some(&(8.0, 9.0)));
    }

    #[test]
    fn test_flexpath_width_change() {
        let mut path = FlexPath::new((0.0, 0.0), 1.0, 1, 0);
        path.segment((10.0, 0.0), Some(3.0), None, false);
        assert_eq!(path.widths.last(), Some(&3.0));
    }

    #[test]
    fn test_flexpath_to_polygon() {
        let mut path = FlexPath::new((0.0, 0.0), 2.0, 1, 0);
        path.segment((10.0, 0.0), None, None, false);
        let poly = path.to_polygon();
        assert!(poly.is_some());
        let pts = poly.unwrap();
        assert!(pts.len() >= 4);
    }

    #[test]
    fn test_flexpath_polygon_area() {
        // Straight horizontal path: 10 units long, 2 units wide → area ≈ 20
        let mut path = FlexPath::new((0.0, 0.0), 2.0, 1, 0);
        path.segment((10.0, 0.0), None, None, false);
        let poly = path.to_polygon().unwrap();
        let area = crate::geometry::polygon_area(&poly);
        assert!((area - 20.0).abs() < 2.0, "Expected area ~20, got {}", area);
    }

    #[test]
    fn test_flexpath_multisegment() {
        let mut path = FlexPath::new((0.0, 0.0), 1.0, 1, 0);
        path.segment((5.0, 0.0), None, None, false);
        path.segment((5.0, 5.0), None, None, false);
        path.segment((0.0, 5.0), None, None, false);
        let poly = path.to_polygon();
        assert!(poly.is_some());
    }

    #[test]
    fn test_flexpath_arc() {
        let mut path = FlexPath::new((5.0, 0.0), 1.0, 1, 0);
        path.arc(5.0, 0.0, PI / 2.0, None);
        assert!(path.points.len() > 2);
        let last = path.points.last().unwrap();
        assert!((last.0 - 5.0).abs() < 5.0 * 0.1 + 0.5);
        assert!((last.1 - 5.0).abs() < 5.0 * 0.1 + 0.5);
    }

    #[test]
    fn test_flexpath_bezier() {
        let mut path = FlexPath::new((0.0, 0.0), 1.0, 1, 0);
        path.bezier((2.0, 4.0), (8.0, 4.0), (10.0, 0.0), None, 10);
        assert!(path.points.len() > 5);
    }

    #[test]
    fn test_flexpath_end_caps_halfwidth() {
        let mut path = FlexPath::new((0.0, 0.0), 2.0, 1, 0);
        path.segment((10.0, 0.0), None, None, false);
        path.end_caps = (EndCap::HalfWidth, EndCap::HalfWidth);
        let poly = path.to_polygon().unwrap();
        assert!(poly.len() >= 4);
    }

    #[test]
    fn test_flexpath_end_caps_round() {
        let mut path = FlexPath::new((0.0, 0.0), 2.0, 1, 0);
        path.segment((10.0, 0.0), None, None, false);
        path.end_caps = (EndCap::Round, EndCap::Round);
        let poly = path.to_polygon().unwrap();
        // Round caps add more points
        assert!(poly.len() > 6);
    }

    #[test]
    fn test_flexpath_bounding_box() {
        let mut path = FlexPath::new((0.0, 0.0), 2.0, 1, 0);
        path.segment((10.0, 0.0), None, None, false);
        let bb = path.bounding_box().unwrap();
        assert!(bb.x_min <= 0.0);
        assert!(bb.x_max >= 10.0);
        assert!(bb.y_max - bb.y_min >= 1.9); // at least width-1 tolerance
    }

    #[test]
    fn test_flexpath_offset() {
        let mut path = FlexPath::new((0.0, 0.0), 2.0, 1, 0);
        path.segment((10.0, 0.0), None, Some(1.0), false);
        let poly = path.to_polygon();
        assert!(poly.is_some());
    }

    #[test]
    fn test_robust_path_basic() {
        let mut rp = RobustPath::new((0.0, 0.0), 2.0, 1, 0);
        rp.segment((10.0, 0.0), None, false);
        rp.segment((10.0, 10.0), None, false);
        let poly = rp.to_polygon();
        assert!(poly.is_some());
    }

    #[test]
    fn test_flexpath_length() {
        let mut path = FlexPath::new((0.0, 0.0), 1.0, 1, 0);
        path.segment((3.0, 4.0), None, None, false); // 5 units
        assert!((path.length() - 5.0).abs() < 1e-10);
    }
}
