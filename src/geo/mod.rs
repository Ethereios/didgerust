// src/geo/mod.rs
//! Geometry module – representation of bore geometry and helper operations.

use serde::{Deserialize, Serialize};

/// `Geo` holds a piecewise‑linear description of the instrument bore.
/// Each entry is `[x_mm, diameter_mm]`. The points must be ordered by
/// increasing `x`. This mirrors the Python `Geo` class.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Geo {
    /// Vector of `[x, diameter]` points in millimetres.
    pub points: Vec<[f64; 2]>,
}

impl Geo {
    /// Create a new `Geo` from a vector of points. Panics if the vector
    /// has fewer than two points or is not sorted by `x`.
    pub fn new(mut points: Vec<[f64; 2]>) -> Self {
        assert!(points.len() >= 2, "Geo requires at least two points");
        // Ensure sorted by x.
        points.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        Self { points }
    }

    /// Total length of the bore (mm).
    pub fn length(&self) -> f64 {
        let first = self.points.first().unwrap()[0];
        let last = self.points.last().unwrap()[0];
        last - first
    }

    /// Alias for `diameter_at` returning a default of 0.0 if out of range.
    pub fn diameter_at_x(&self, x_mm: f64) -> f64 {
        self.diameter_at(x_mm).unwrap_or(0.0)
    }

    /// Linear interpolation of the diameter at a given longitudinal
    /// position `x_mm`. Returns `None` if `x` lies outside the defined
    /// range.
    pub fn diameter_at(&self, x_mm: f64) -> Option<f64> {
        if x_mm < self.points[0][0] || x_mm > self.points.last().unwrap()[0] {
            return None;
        }
        // Find segment containing x.
        for window in self.points.windows(2) {
            let x0 = window[0][0];
            let d0 = window[0][1];
            let x1 = window[1][0];
            let d1 = window[1][1];
            if (x0..=x1).contains(&x_mm) {
                let t = (x_mm - x0) / (x1 - x0);
                return Some(d0 + t * (d1 - d0));
            }
        }
        // Edge case – exact match on the last point.
        Some(self.points.last().unwrap()[1])
    }

    /// Scale all diameters by `factor` (unitless).
    pub fn scale_diameter(&mut self, factor: f64) {
        for pt in &mut self.points {
            pt[1] *= factor;
        }
    }

    /// Stretch (or compress) the longitudinal axis by `factor`.
    pub fn stretch(&mut self, factor: f64) {
        for pt in &mut self.points {
            pt[0] *= factor;
        }
    }

    /// Add a sinusoidal bubble centered at `center_mm` with `width_mm`
    /// (full width at half‑maximum) and `height_mm` (peak increase in
    /// diameter). The method inserts additional points into the existing
    /// vector to preserve piecewise‑linear continuity.
    pub fn add_bubble(&mut self, center_mm: f64, width_mm: f64, height_mm: f64) {
        // Determine affected region.
        let left = center_mm - width_mm / 2.0;
        let right = center_mm + width_mm / 2.0;
        // Collect points that will stay unchanged.
        let mut new_pts = Vec::new();
        for &pt in &self.points {
            if pt[0] < left || pt[0] > right {
                new_pts.push(pt);
            }
        }
        // Insert three new points: left edge, peak, right edge.
        let d_left = self.diameter_at(left).unwrap_or_else(|| self.points[0][1]);
        let d_right = self.diameter_at(right).unwrap_or_else(|| self.points.last().unwrap()[1]);
        // Simple cosine‑shaped bump.
        let num_steps = 8; // resolution of the bump.
        for i in 0..=num_steps {
            let t = i as f64 / num_steps as f64;
            let x = left + t * width_mm;
            // Cosine bump: height * (0.5 - 0.5*cos(π*t))
            let bump = height_mm * (0.5 - 0.5 * (std::f64::consts::PI * t).cos());
            // Linear interpolation between left and right diameters.
            let d_base = d_left + t * (d_right - d_left);
            new_pts.push([x, d_base + bump]);
        }
        // Sort and replace.
        new_pts.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        self.points = new_pts;
    }
}

impl Geo {
    /// Create a cone‑shaped geometry for quick testing.
    /// `length` in mm, `d0` mouth diameter, `d_bell` bell diameter, `points` number of points.
    pub fn make_cone(length: f64, d0: f64, d_bell: f64, points: usize) -> Self {
        let mut pts = Vec::with_capacity(points);
        for i in 0..points {
            let x = i as f64 * length / (points as f64 - 1.0);
            let d = d0 + (d_bell - d0) * (x / length);
            pts.push([x, d]);
        }
        Self::new(pts)
    }

    /// Return bell (last point) diameter in mm.
    pub fn bellsize(&self) -> f64 {
        self.points.last().map(|p| p[1]).unwrap_or(0.0)
    }

    /// Return maximum diameter in mm.
    pub fn get_max_diameter(&self) -> f64 {
        self.points.iter().map(|p| p[1]).fold(0.0, f64::max)
    }

    /// Approximate volume (mm³) using truncated cone segments.
    pub fn compute_volume(&self) -> f64 {
        if self.points.len() < 2 { return 0.0; }
        let mut vol = 0.0;
        for window in self.points.windows(2) {
            let x0 = window[0][0];
            let d0 = window[0][1];
            let x1 = window[1][0];
            let d1 = window[1][1];
            let dx = x1 - x0;
            let r0 = d0 / 2.0;
            let r1 = d1 / 2.0;
            vol += std::f64::consts::PI * dx * (r0 * r0 + r0 * r1 + r1 * r1) / 3.0;
        }
        vol
    }
}

pub type BoreGeometry = Geo;

// Simple unit tests for the geometry helpers.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_length() {
        let geo = Geo::new(vec![[0.0, 20.0], [100.0, 30.0]]);
        assert_eq!(geo.length(), 100.0);
    }

    #[test]
    fn interpolate() {
        let geo = Geo::new(vec![[0.0, 20.0], [100.0, 40.0]]);
        let d = geo.diameter_at(50.0).unwrap();
        assert!((d - 30.0).abs() < 1e-6);
    }

    #[test]
    fn bubble() {
        let mut geo = Geo::new(vec![[0.0, 20.0], [200.0, 20.0]]);
        geo.add_bubble(100.0, 40.0, 5.0);
        // Ensure the peak exists near the centre.
        let centre_d = geo.diameter_at(100.0).unwrap();
        assert!(centre_d > 20.0);
    }
}