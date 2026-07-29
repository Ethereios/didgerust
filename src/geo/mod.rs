//! Geometry module – representation of bore geometry and helper operations.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Bore geometry of a didgeridoo as a list of (x, diameter) segments in mm
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Geo {
    /// List of [x, d] with x = distance from mouthpiece, d = diameter (both in mm)
    pub geo: Vec<[f64; 2]>,
}

impl Geo {
    pub fn new(geo: Vec<[f64; 2]>) -> Self {
        let mut clean_geo = Vec::new();
        for i in 0..geo.len() {
            if i > 0 && geo[i][0] == geo[i - 1][0] {
                continue;
            }
            clean_geo.push(geo[i]);
        }
        Self { geo: clean_geo }
    }

    pub fn make_cone(length: f64, d1: f64, d2: f64, n_segments: usize) -> Self {
        let mut geo = Vec::new();
        for i in 0..=n_segments {
            let x = (i as f64 / n_segments as f64) * length;
            let d = d1 + (d2 - d1) * (i as f64 / n_segments as f64);
            geo.push([x, d]);
        }
        Self::new(geo)
    }

    pub fn make_cylinder(length: f64, d: f64, n_segments: usize) -> Self {
        Self::make_cone(length, d, d, n_segments)
    }

    pub fn length(&self) -> f64 {
        self.geo.last().map(|p| p[0]).unwrap_or(0.0)
    }

    pub fn bellsize(&self) -> f64 {
        self.geo.last().map(|p| p[1]).unwrap_or(0.0)
    }

    pub fn diameter_at_x(&self, x: f64) -> f64 {
        if self.geo.is_empty() {
            return 0.0;
        }
        if x <= self.geo[0][0] {
            return self.geo[0][1];
        }
        if x >= self.geo.last().unwrap()[0] {
            return self.geo.last().unwrap()[1];
        }
        for i in 1..self.geo.len() {
            if x <= self.geo[i][0] {
                let x0 = self.geo[i - 1][0];
                let d0 = self.geo[i - 1][1];
                let x1 = self.geo[i][0];
                let d1 = self.geo[i][1];
                let t = (x - x0) / (x1 - x0);
                return d0 + t * (d1 - d0);
            }
        }
        self.geo.last().unwrap()[1]
    }

    pub fn compute_volume(&self) -> f64 {
        let mut volume = 0.0;
        for i in 1..self.geo.len() {
            let dx = self.geo[i][0] - self.geo[i - 1][0];
            let d1 = self.geo[i - 1][1];
            let d2 = self.geo[i][1];
            volume += dx * PI * (d1 * d1 + d1 * d2 + d2 * d2) / 12.0;
        }
        volume
    }

    pub fn scale_diameter(&mut self, factor: f64) {
        for pt in &mut self.geo {
            pt[1] *= factor;
        }
    }

    pub fn stretch(&mut self, factor: f64) {
        for pt in &mut self.geo {
            pt[0] *= factor;
        }
    }

    pub fn add_bubble(&mut self, center: f64, width: f64, height: f64) {
        let left = center - width / 2.0;
        let right = center + width / 2.0;
        let d_left = self.diameter_at_x(left);
        let d_center = self.diameter_at_x(center);
        let d_right = self.diameter_at_x(right);

        let mut new_geo = Vec::new();
        for pt in &self.geo {
            if pt[0] < left || pt[0] > right {
                new_geo.push(*pt);
            }
        }
        new_geo.push([left, d_left]);
        new_geo.push([center, d_center + height]);
        new_geo.push([right, d_right]);
        new_geo.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        self.geo = new_geo;
    }

    pub fn get_max_diameter(&self) -> f64 {
        self.geo.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn get_max_d(&self) -> f64 {
        self.get_max_diameter()
    }
}

pub type BoreGeometry = Geo;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_length() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        assert!((geo.length() - 1000.0).abs() < 1.0);
    }

    #[test]
    fn interpolate() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        let d = geo.diameter_at_x(500.0);
        assert!((d - 46.0).abs() < 2.0);
    }

    #[test]
    fn bubble() {
        let mut geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        geo.add_bubble(500.0, 200.0, 70.0);
        let centre_d = geo.diameter_at_x(500.0);
        assert!(centre_d > 20.0);
    }
}