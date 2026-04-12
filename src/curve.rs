// Curve generation for EDA layout
// Provides arc, Bezier, ellipse, and other curve primitives
// Mirrors the Curve API of gdstk

use std::f64::consts::PI;

/// A curve builder for creating complex polygon outlines
#[derive(Debug, Clone)]
pub struct Curve {
    /// All accumulated points on the curve
    pub points: Vec<(f64, f64)>,
    /// Tolerance for arc/curve approximation
    pub tolerance: f64,
}

impl Curve {
    /// Create a new curve starting at the given point
    pub fn new(start: (f64, f64), tolerance: f64) -> Self {
        Curve {
            points: vec![start],
            tolerance: tolerance.max(1e-12),
        }
    }

    /// Add a line segment to the next point
    pub fn line(&mut self, end: (f64, f64), relative: bool) -> &mut Self {
        let pt = if relative {
            let last = *self.points.last().unwrap();
            (last.0 + end.0, last.1 + end.1)
        } else {
            end
        };
        self.points.push(pt);
        self
    }

    /// Add multiple line segments (polyline)
    pub fn polyline(&mut self, pts: &[(f64, f64)], relative: bool) -> &mut Self {
        for &pt in pts {
            self.line(pt, relative);
        }
        self
    }

    /// Add a circular arc from current point
    /// - `center_offset`: offset to arc center from current point
    /// - `angle`: total sweep angle in radians (positive = CCW)
    pub fn arc_center(&mut self, center_offset: (f64, f64), angle: f64) -> &mut Self {
        let start = *self.points.last().unwrap();
        let center = (start.0 + center_offset.0, start.1 + center_offset.1);
        let radius = (center_offset.0 * center_offset.0 + center_offset.1 * center_offset.1).sqrt();

        if radius < 1e-12 || angle.abs() < 1e-12 {
            return self;
        }

        let start_angle = (-center_offset.1).atan2(-center_offset.0);
        let num_pts = ((angle.abs() / (2.0 * (self.tolerance / radius).acos().max(1e-6))).ceil()
            as usize)
            .max(2)
            .min(2000);

        for i in 1..=num_pts {
            let t = i as f64 / num_pts as f64;
            let a = start_angle + t * angle;
            self.points
                .push((center.0 + radius * a.cos(), center.1 + radius * a.sin()));
        }
        self
    }

    /// Add an arc defined by radius and sweep angles
    pub fn arc(&mut self, radius: f64, initial_angle: f64, final_angle: f64) -> &mut Self {
        if radius <= 0.0 {
            return self;
        }

        let sweep = final_angle - initial_angle;
        let num_pts = ((sweep.abs() / (2.0 * (self.tolerance / radius).acos().max(1e-6))).ceil()
            as usize)
            .max(2)
            .min(2000);

        let last = *self.points.last().unwrap();
        let center = (
            last.0 - radius * initial_angle.cos(),
            last.1 - radius * initial_angle.sin(),
        );

        for i in 1..=num_pts {
            let t = i as f64 / num_pts as f64;
            let a = initial_angle + t * sweep;
            self.points
                .push((center.0 + radius * a.cos(), center.1 + radius * a.sin()));
        }
        self
    }

    /// Add a quadratic Bezier curve (one control point)
    pub fn bezier2(
        &mut self,
        ctrl: (f64, f64),
        end: (f64, f64),
        relative: bool,
        num_points: usize,
    ) -> &mut Self {
        let start = *self.points.last().unwrap();
        let (ctrl, end) = if relative {
            (
                (start.0 + ctrl.0, start.1 + ctrl.1),
                (start.0 + end.0, start.1 + end.1),
            )
        } else {
            (ctrl, end)
        };

        let n = num_points.max(2);
        for i in 1..=n {
            let t = i as f64 / n as f64;
            let it = 1.0 - t;
            let pt = (
                it * it * start.0 + 2.0 * it * t * ctrl.0 + t * t * end.0,
                it * it * start.1 + 2.0 * it * t * ctrl.1 + t * t * end.1,
            );
            self.points.push(pt);
        }
        self
    }

    /// Add a cubic Bezier curve (two control points)
    pub fn bezier3(
        &mut self,
        ctrl1: (f64, f64),
        ctrl2: (f64, f64),
        end: (f64, f64),
        relative: bool,
        num_points: usize,
    ) -> &mut Self {
        let start = *self.points.last().unwrap();
        let (ctrl1, ctrl2, end) = if relative {
            (
                (start.0 + ctrl1.0, start.1 + ctrl1.1),
                (start.0 + ctrl2.0, start.1 + ctrl2.1),
                (start.0 + end.0, start.1 + end.1),
            )
        } else {
            (ctrl1, ctrl2, end)
        };

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
        }
        self
    }

    /// Add a smooth cubic Bezier (C1-continuous with previous segment)
    pub fn smooth_bezier(
        &mut self,
        ctrl2: (f64, f64),
        end: (f64, f64),
        relative: bool,
        num_points: usize,
    ) -> &mut Self {
        let n = self.points.len();
        let start = self.points[n - 1];

        // Reflect previous control point for C1 continuity
        let ctrl1 = if n >= 2 {
            let prev = self.points[n - 2];
            (2.0 * start.0 - prev.0, 2.0 * start.1 - prev.1)
        } else {
            start
        };

        self.bezier3(ctrl1, ctrl2, end, relative, num_points)
    }

    /// Add an elliptical arc
    pub fn ellipse_arc(
        &mut self,
        rx: f64,
        ry: f64,
        rotation: f64,
        initial_angle: f64,
        final_angle: f64,
        num_points: usize,
    ) -> &mut Self {
        let last = *self.points.last().unwrap();
        let center_x = last.0
            - rx * (rotation.cos() * initial_angle.cos()
                - ry / rx * rotation.sin() * initial_angle.sin());
        let center_y = last.1
            - rx * (rotation.sin() * initial_angle.cos()
                + ry / rx * rotation.cos() * initial_angle.sin());

        let sweep = final_angle - initial_angle;
        let n = num_points.max(2).min(2000);

        let cos_r = rotation.cos();
        let sin_r = rotation.sin();

        for i in 1..=n {
            let t = i as f64 / n as f64;
            let angle = initial_angle + t * sweep;
            let local_x = rx * angle.cos();
            let local_y = ry * angle.sin();
            self.points.push((
                center_x + cos_r * local_x - sin_r * local_y,
                center_y + sin_r * local_x + cos_r * local_y,
            ));
        }
        self
    }

    /// Interpolate through a sequence of points using a smooth spline
    pub fn interpolate(
        &mut self,
        pts: &[(f64, f64)],
        tension: f64,
        num_per_segment: usize,
    ) -> &mut Self {
        if pts.is_empty() {
            return self;
        }

        let start = *self.points.last().unwrap();
        let all_pts: Vec<(f64, f64)> = std::iter::once(start).chain(pts.iter().cloned()).collect();
        let n = all_pts.len();

        for i in 0..n - 1 {
            let p0 = if i > 0 { all_pts[i - 1] } else { all_pts[i] };
            let p1 = all_pts[i];
            let p2 = all_pts[i + 1];
            let p3 = if i + 2 < n {
                all_pts[i + 2]
            } else {
                all_pts[i + 1]
            };

            let m1 = (
                (1.0 - tension) * (p2.0 - p0.0) / 2.0,
                (1.0 - tension) * (p2.1 - p0.1) / 2.0,
            );
            let m2 = (
                (1.0 - tension) * (p3.0 - p1.0) / 2.0,
                (1.0 - tension) * (p3.1 - p1.1) / 2.0,
            );

            let num_seg = num_per_segment.max(2);
            for step in 1..=num_seg {
                let t = step as f64 / num_seg as f64;
                let t2 = t * t;
                let t3 = t2 * t;

                let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
                let h10 = t3 - 2.0 * t2 + t;
                let h01 = -2.0 * t3 + 3.0 * t2;
                let h11 = t3 - t2;

                let x = h00 * p1.0 + h10 * m1.0 + h01 * p2.0 + h11 * m2.0;
                let y = h00 * p1.1 + h10 * m1.1 + h01 * p2.1 + h11 * m2.1;
                self.points.push((x, y));
            }
        }
        self
    }

    /// Close the curve by connecting back to the start
    pub fn close(&mut self) -> &mut Self {
        if let Some(&first) = self.points.first() {
            if let Some(&last) = self.points.last() {
                let dx = first.0 - last.0;
                let dy = first.1 - last.1;
                if (dx * dx + dy * dy).sqrt() > 1e-12 {
                    self.points.push(first);
                }
            }
        }
        self
    }

    /// Get the accumulated points
    pub fn get_points(&self) -> &[(f64, f64)] {
        &self.points
    }

    /// Compute the total length of the curve
    pub fn length(&self) -> f64 {
        let mut total = 0.0;
        for i in 0..self.points.len().saturating_sub(1) {
            let dx = self.points[i + 1].0 - self.points[i].0;
            let dy = self.points[i + 1].1 - self.points[i].1;
            total += (dx * dx + dy * dy).sqrt();
        }
        total
    }
}

// ============================================================================
// Standalone curve/shape generators
// ============================================================================

/// Generate a regular polygon (equilateral) inscribed in a circle of given radius
pub fn regular_polygon(
    center: (f64, f64),
    radius: f64,
    sides: usize,
    initial_angle: f64,
) -> Vec<(f64, f64)> {
    if sides < 3 {
        return Vec::new();
    }
    let mut pts = Vec::with_capacity(sides + 1);
    for i in 0..=sides {
        let angle = initial_angle + 2.0 * PI * i as f64 / sides as f64;
        pts.push((
            center.0 + radius * angle.cos(),
            center.1 + radius * angle.sin(),
        ));
    }
    pts
}

/// Generate an ellipse approximated with line segments
pub fn ellipse(
    center: (f64, f64),
    rx: f64,
    ry: f64,
    initial_angle: f64,
    final_angle: f64,
    tolerance: f64,
    num_points: Option<usize>,
) -> Vec<(f64, f64)> {
    let r_max = rx.max(ry);
    let n = num_points
        .unwrap_or_else(|| {
            ((final_angle - initial_angle).abs() / (2.0 * (tolerance / r_max).acos().max(1e-6)))
                .ceil() as usize
        })
        .max(3)
        .min(10000);

    let sweep = final_angle - initial_angle;
    let mut pts = Vec::with_capacity(n + 1);

    for i in 0..=n {
        let t = i as f64 / n as f64;
        let angle = initial_angle + t * sweep;
        pts.push((center.0 + rx * angle.cos(), center.1 + ry * angle.sin()));
    }

    pts
}

/// Generate a rounded rectangle (rectangle with filleted corners)
pub fn rounded_rectangle(
    x_min: f64,
    y_min: f64,
    x_max: f64,
    y_max: f64,
    radius: f64,
    points_per_corner: usize,
) -> Vec<(f64, f64)> {
    let r = radius.min((x_max - x_min) / 2.0).min((y_max - y_min) / 2.0);
    let ppc = points_per_corner.max(2);
    let mut pts = Vec::new();

    // Bottom-left corner (270° to 180° = from -90° to -180°)
    for i in 0..=ppc {
        let a = -PI / 2.0 - PI / 2.0 * i as f64 / ppc as f64;
        pts.push((x_min + r + r * a.cos(), y_min + r + r * a.sin()));
    }
    // Bottom-right corner (180° to 90°)
    for i in 0..=ppc {
        let a = PI - PI / 2.0 * i as f64 / ppc as f64;
        pts.push((x_max - r + r * a.cos(), y_min + r + r * a.sin()));
    }
    // Top-right corner (90° to 0°)
    for i in 0..=ppc {
        let a = PI / 2.0 - PI / 2.0 * i as f64 / ppc as f64;
        pts.push((x_max - r + r * a.cos(), y_max - r + r * a.sin()));
    }
    // Top-left corner (0° to -90°)
    for i in 0..=ppc {
        let a = 0.0 - PI / 2.0 * i as f64 / ppc as f64;
        pts.push((x_min + r + r * a.cos(), y_max - r + r * a.sin()));
    }

    // Close
    if let Some(&first) = pts.first() {
        pts.push(first);
    }
    pts
}

/// Generate a star polygon
pub fn star(
    center: (f64, f64),
    outer_radius: f64,
    inner_radius: f64,
    points: usize,
    initial_angle: f64,
) -> Vec<(f64, f64)> {
    if points < 3 {
        return Vec::new();
    }
    let total = points * 2;
    let mut pts = Vec::with_capacity(total + 1);
    for i in 0..=total {
        let angle = initial_angle + PI * i as f64 / points as f64;
        let r = if i % 2 == 0 {
            outer_radius
        } else {
            inner_radius
        };
        pts.push((center.0 + r * angle.cos(), center.1 + r * angle.sin()));
    }
    pts
}

/// Generate a spiral (Archimedean spiral)
pub fn spiral(
    center: (f64, f64),
    inner_radius: f64,
    outer_radius: f64,
    turns: f64,
    num_points: usize,
) -> Vec<(f64, f64)> {
    let n = num_points.max(4);
    let dr = (outer_radius - inner_radius) / (turns * 2.0 * PI);
    let mut pts = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let angle = t * turns * 2.0 * PI;
        let r = inner_radius + dr * angle;
        pts.push((center.0 + r * angle.cos(), center.1 + r * angle.sin()));
    }
    pts
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curve_line() {
        let mut c = Curve::new((0.0, 0.0), 0.01);
        c.line((5.0, 3.0), false);
        assert_eq!(c.points.len(), 2);
        assert_eq!(c.points[1], (5.0, 3.0));
    }

    #[test]
    fn test_curve_line_relative() {
        let mut c = Curve::new((2.0, 3.0), 0.01);
        c.line((1.0, 1.0), true);
        assert_eq!(c.points.last(), Some(&(3.0, 4.0)));
    }

    #[test]
    fn test_curve_arc() {
        let mut c = Curve::new((5.0, 0.0), 0.01);
        c.arc(5.0, 0.0, PI / 2.0);
        assert!(c.points.len() > 2);
        let last = c.points.last().unwrap();
        // Should end near (0, 5) for arc from angle 0 to π/2 radius 5
        assert!((last.0 - 0.0).abs() < 0.5, "x should be ~0, got {}", last.0);
        assert!((last.1 - 5.0).abs() < 0.5, "y should be ~5, got {}", last.1);
    }

    #[test]
    fn test_curve_arc_full_circle() {
        let mut c = Curve::new((5.0, 0.0), 0.01);
        c.arc(5.0, 0.0, 2.0 * PI);
        let last = c.points.last().unwrap();
        // Should return close to start
        assert!((last.0 - 5.0).abs() < 0.5, "Expected ~5, got {}", last.0);
        assert!((last.1 - 0.0).abs() < 0.5, "Expected ~0, got {}", last.1);
    }

    #[test]
    fn test_curve_bezier2() {
        let mut c = Curve::new((0.0, 0.0), 0.01);
        c.bezier2((5.0, 5.0), (10.0, 0.0), false, 10);
        assert_eq!(c.points.len(), 11);
        assert_eq!(c.points.last(), Some(&(10.0, 0.0)));
    }

    #[test]
    fn test_curve_bezier3() {
        let mut c = Curve::new((0.0, 0.0), 0.01);
        c.bezier3((2.0, 4.0), (8.0, 4.0), (10.0, 0.0), false, 10);
        assert_eq!(c.points.len(), 11);
        let last = c.points.last().unwrap();
        assert!((last.0 - 10.0).abs() < 1e-10);
        assert!((last.1 - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_curve_close() {
        let mut c = Curve::new((0.0, 0.0), 0.01);
        c.line((5.0, 0.0), false);
        c.line((5.0, 5.0), false);
        c.close();
        assert_eq!(c.points.last(), Some(&(0.0, 0.0)));
    }

    #[test]
    fn test_curve_length() {
        let mut c = Curve::new((0.0, 0.0), 0.01);
        c.line((3.0, 4.0), false); // 5 units
        assert!((c.length() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_curve_polyline() {
        let mut c = Curve::new((0.0, 0.0), 0.01);
        c.polyline(&[(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)], false);
        assert_eq!(c.points.len(), 4);
    }

    #[test]
    fn test_regular_polygon() {
        let hex = regular_polygon((0.0, 0.0), 1.0, 6, 0.0);
        assert_eq!(hex.len(), 7); // 6 sides + closing point
                                  // All vertices should be at distance 1 from center
        for &(x, y) in &hex[..6] {
            let d = (x * x + y * y).sqrt();
            assert!((d - 1.0).abs() < 1e-10, "Distance should be 1, got {}", d);
        }
    }

    #[test]
    fn test_ellipse_full() {
        let pts = ellipse((0.0, 0.0), 5.0, 3.0, 0.0, 2.0 * PI, 0.01, None);
        assert!(pts.len() >= 3);
        // First and last should be the same (complete ellipse)
        let first = pts[0];
        let last = *pts.last().unwrap();
        assert!((first.0 - last.0).abs() < 0.1);
        assert!((first.1 - last.1).abs() < 0.1);
    }

    #[test]
    fn test_rounded_rectangle() {
        let rr = rounded_rectangle(0.0, 0.0, 10.0, 5.0, 1.0, 4);
        assert!(rr.len() > 4); // More points than plain rectangle
                               // All points should be within the rectangle bounds
        for &(x, y) in &rr {
            assert!(x >= -0.01 && x <= 10.01, "x={} out of bounds", x);
            assert!(y >= -0.01 && y <= 5.01, "y={} out of bounds", y);
        }
    }

    #[test]
    fn test_star() {
        let s = star((0.0, 0.0), 5.0, 2.0, 5, 0.0);
        assert_eq!(s.len(), 11); // 5*2 = 10 + closing point
    }

    #[test]
    fn test_spiral() {
        let s = spiral((0.0, 0.0), 1.0, 5.0, 2.0, 100);
        assert_eq!(s.len(), 101);
        // First point should be at radius ≈ 1
        let (x, y) = s[0];
        let r = (x * x + y * y).sqrt();
        assert!(
            (r - 1.0).abs() < 0.5,
            "First radius should be ~1, got {}",
            r
        );
    }

    #[test]
    fn test_curve_interpolate() {
        let mut c = Curve::new((0.0, 0.0), 0.01);
        c.interpolate(&[(5.0, 5.0), (10.0, 0.0)], 0.5, 5);
        assert!(c.points.len() > 3);
    }

    #[test]
    fn test_curve_ellipse_arc() {
        let mut c = Curve::new((5.0, 0.0), 0.01);
        c.ellipse_arc(5.0, 3.0, 0.0, 0.0, PI, 20);
        assert!(c.points.len() > 5);
    }
}
