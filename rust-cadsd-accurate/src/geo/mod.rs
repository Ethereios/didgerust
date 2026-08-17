//! Geometry representation for didgeridoo bore profiles
//!
//! This module implements the exact same geometry representation as the Python DidgeLab,
//! where a geometry is a list of segments, each (x, diameter) in mm: distance from
//! mouthpiece and bore diameter.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Bore geometry of a didgeridoo as a list of (x, diameter) segments in mm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geo {
    /// List of [x, d] with x = distance from mouthpiece, d = diameter (both in mm)
    pub geo: Vec<[f64; 2]>,
}

impl Geo {
    /// Create a new geometry from segments
    pub fn new(geo: Vec<[f64; 2]>) -> Self {
        let mut clean_geo = Vec::new();
        
        // Remove zero length segments (same implementation as Python)
        for i in 0..geo.len() {
            if i > 0 && geo[i][0] == geo[i-1][0] {
                continue;
            }
            clean_geo.push(geo[i]);
        }
        
        Self { geo: clean_geo }
    }
    
    /// Create from JSON file (same as Python)
    pub fn from_file(infile: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(infile)?;
        let geo: Vec<[f64; 2]> = serde_json::from_str(&content)?;
        Ok(Self::new(geo))
    }
    
    /// Write to text file (same as Python)
    pub fn to_file(&self, outfile: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();
        for segment in &self.geo {
            content.push_str(&format!("{:.10} {:.10}\n", segment[0], segment[1]));
        }
        std::fs::write(outfile, content)?;
        Ok(())
    }
    
    /// Build a conical bore: length (mm), diameters d1 (mouth), d2 (bell), n_segments
    pub fn make_cone(length: f64, d1: f64, d2: f64, n_segments: usize) -> Self {
        let mut shape = vec![[0.0, d1]];
        
        let z = (d2 - d1) / 2.0;
        let angle = (z / length).atan();
        
        for i in 1..n_segments {
            let x = length * (i as f64) / (n_segments as f64);
            let y = 2.0 * x * angle.tan() + d1;
            shape.push([x, y]);
        }
        
        shape.push([length, d2]);
        Self::new(shape)
    }
    
    /// Scale all x (length) by factor; diameters unchanged (same as Python)
    pub fn stretch(&mut self, factor: f64) {
        for i in 0..self.geo.len() {
            self.geo[i][0] *= factor;
        }
    }
    
    /// Scale all x and diameter by factor (e.g. mm to m) (same as Python)
    pub fn scale(&mut self, factor: f64) {
        for i in 0..self.geo.len() {
            self.geo[i][0] *= factor;
            self.geo[i][1] *= factor;
        }
    }
    
    /// Create a copy (same as Python)
    pub fn copy(&self) -> Self {
        Self {
            geo: self.geo.clone(),
        }
    }
    
    /// Insert a bulge (bubble) at position pos with given width and height (same as Python)
    pub fn make_bubble(&mut self, pos: f64, width: f64, height: f64) {
        let mut index = 0;
        for i in 0..self.geo.len() - 1 {
            if self.geo[i + 1][0] > pos {
                index = i;
                break;
            }
        }
        
        let left = self.geo[0..=index].to_vec();
        let right = self.geo[index + 1..].to_vec();
        
        let new_geo = [
            left,
            vec![
                [pos - width / 2.0, self.geo[index][1]],
                [pos, height],
                [pos + width / 2.0, self.geo[index + 1][1]],
            ],
            right,
        ].concat();
        
        self.geo = new_geo;
    }
    
    /// Shift x-coordinates of segments [start..end] by offset (same as Python)
    pub fn move_segments_x(&mut self, start: usize, end: usize, offset: f64) {
        for i in start..=end.min(self.geo.len() - 1) {
            self.geo[i][0] += offset;
        }
    }
    
    /// Total length in mm (x of last segment) (same as Python)
    pub fn length(&self) -> f64 {
        self.geo.last().map(|seg| seg[0]).unwrap_or(0.0)
    }
    
    /// Bell diameter in mm (diameter of last segment) (same as Python)
    pub fn bellsize(&self) -> f64 {
        self.geo.last().map(|seg| seg[1]).unwrap_or(0.0)
    }
    
    /// Sort segments by x (distance from mouthpiece) (same as Python)
    pub fn sort_segments(&mut self) {
        self.geo.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
    }
    
    /// Interpolated bore diameter at position x (mm) (same as Python)
    pub fn diameter_at_x(&self, x: f64) -> f64 {
        if self.geo.is_empty() {
            return 0.0;
        }
        
        if x <= 0.0 {
            return self.geo[0][1];
        }
        
        if x >= self.length() {
            return self.bellsize();
        }
        
        // Find the segment containing x
        for i in 1..self.geo.len() {
            if x < self.geo[i][0] {
                let x1 = self.geo[i-1][0];
                let y1 = self.geo[i-1][1];
                let x2 = self.geo[i][0];
                let y2 = self.geo[i][1];
                
                let ydiff = (y2 - y1) / 2.0;
                let xdiff = x2 - x1;
                
                let angle = (ydiff / xdiff).atan();
                return 2.0 * angle.tan() * (x - x1) + y1;
            }
        }
        
        self.bellsize()
    }
    
    /// Approximate internal volume in mm³ (trapezoidal rule along bore) (same as Python)
    pub fn compute_volume(&self) -> f64 {
        let mut volume = 0.0;
        
        for i in 1..self.geo.len() {
            let length = self.geo[i][0] - self.geo[i-1][0];
            let d1 = self.geo[i-1][1];
            let d2 = self.geo[i][1];
            
            // Trapezoidal rule for volume calculation
            volume += length * PI * (d1 * d1 + d1 * d2 + d2 * d2) / 12.0;
        }
        
        volume
    }
    
    /// Scale geometry length to max_length (factor applied to x only) (same as Python)
    pub fn scale_length(&mut self, max_length: f64) {
        let factor = max_length / self.length();
        for i in 0..self.geo.len() {
            self.geo[i][0] *= factor;
        }
    }
    
    /// Maximum bore diameter in geometry (mm) (same as Python)
    pub fn get_max_d(&self) -> f64 {
        self.geo.iter().map(|seg| seg[1]).fold(0.0, f64::max)
    }
    
    /// Scale diameters so maximum diameter becomes max_d (same as Python)
    pub fn scale_diameter(&mut self, max_d: f64) {
        let current_max = self.get_max_d();
        let factor = max_d / current_max;
        for i in 0..self.geo.len() {
            self.geo[i][1] *= factor;
        }
    }
    
    /// Format a short text summary of the geometry (same as Python)
    pub fn print_summary(&self, peaks: Option<&Vec<(f64, f64)>>, loss: Option<f64>) -> String {
        let mut s = format!("length:\t\t{:.2}\n", self.length());
        s.push_str(&format!("bell size:\t{:.2}\n", self.bellsize()));
        s.push_str(&format!("num segments:\t{}\n", self.geo.len()));
        
        if let Some(peak_list) = peaks {
            s.push_str(&format!("num peaks:\t{}\n", peak_list.len()));
        }
        
        if let Some(loss_val) = loss {
            s.push_str(&format!("loss:\t\t{:.2}\n", loss_val));
        }
        
        if let Some(peak_list) = peaks {
            s.push_str(&format!("{:?}", peak_list));
        }
        
        s
    }

    /// Create a Kigali-style parametric shape
    pub fn make_kigali(length: f64, top_diameter: f64, bottom_diameter: f64, power: f64, n_segments: usize) -> Self {
        let mut shape = vec![[0.0, top_diameter]];
        
        for i in 1..n_segments {
            let x = length * (i as f64) / (n_segments as f64);
            let position_ratio = x / length;
            // Power-law taper: y = d0 + (d1 - d0) * (x/L)^power
            let y = top_diameter + (bottom_diameter - top_diameter) * position_ratio.powf(power.abs());
            shape.push([x, y]);
        }
        
        shape.push([length, bottom_diameter]);
        Self::new(shape)
    }

    /// Create an Mbeya-style parametric shape
    pub fn make_mbeya(length: f64, top_diameter: f64, bottom_diameter: f64, power: f64, n_segments: usize) -> Self {
        let mut shape = vec![[0.0, top_diameter]];
        
        // Define sections: straight, opening, bell
        let straight_length = length * 0.3;  // First 30% is straight
        let opening_length = length * 0.5;   // Middle 50% is opening
        let bell_length = length * 0.2;      // Last 20% is bell
        
        // Straight section
        let straight_segments = (n_segments as f64 * 0.3) as usize;
        for i in 1..straight_segments {
            let x = (i as f64) * straight_length / (straight_segments as f64);
            shape.push([x, top_diameter]); // Constant diameter in straight section
        }
        
        // Opening section (transition zone)
        let opening_start = straight_length;
        let opening_segments = (n_segments as f64 * 0.5) as usize;
        let mid_diameter = (top_diameter + bottom_diameter) / 2.0;
        
        for i in 1..opening_segments {
            let x = opening_start + (i as f64) * opening_length / (opening_segments as f64);
            let position_ratio = (i as f64) / (opening_segments as f64);
            let y = top_diameter + (mid_diameter - top_diameter) * position_ratio.powf(power.abs());
            shape.push([x, y]);
        }
        
        // Bell section
        let bell_start = straight_length + opening_length;
        let bell_segments = n_segments - straight_segments - opening_segments;
        
        for i in 1..=bell_segments {
            let x = bell_start + (i as f64) * bell_length / (bell_segments as f64);
            let position_ratio = (i as f64) / (bell_segments as f64);
            let y = mid_diameter + (bottom_diameter - mid_diameter) * position_ratio.powf(power.abs());
            shape.push([x, y]);
        }
        
        // Make sure we end at the exact length
        if shape.last().map(|s| s[0]) != Some(length) {
            shape.push([length, bottom_diameter]);
        }
        
        Self::new(shape)
    }
}

impl Default for Geo {
    fn default() -> Self {
        Self::new(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    
    #[test]
    fn test_cone_creation() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 10);
        assert_eq!(geo.geo.len(), 11); // 10 segments + 1 endpoint
        assert_eq!(geo.geo[0], [0.0, 32.0]);
        assert_eq!(geo.geo[10], [1000.0, 60.0]);
        assert_eq!(geo.length(), 1000.0);
        assert_eq!(geo.bellsize(), 60.0);
    }
    
    #[test]
    fn test_zero_length_removal() {
        let geo_data = vec![
            [0.0, 32.0],
            [0.0, 35.0], // zero length segment
            [100.0, 40.0],
            [100.0, 45.0], // zero length segment
            [1000.0, 60.0],
        ];
        
        let geo = Geo::new(geo_data);
        // Should have removed the zero-length segments
        assert_eq!(geo.geo.len(), 3);
        assert_eq!(geo.geo[0], [0.0, 32.0]);
        assert_eq!(geo.geo[1], [100.0, 40.0]);
        assert_eq!(geo.geo[2], [1000.0, 60.0]);
    }
    
    #[test]
    fn test_diameter_interpolation() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 5);
        let diameter_at_start = geo.diameter_at_x(0.0);
        let diameter_at_middle = geo.diameter_at_x(500.0);
        let diameter_at_end = geo.diameter_at_x(1000.0);
        
        assert_abs_diff_eq!(diameter_at_start, 32.0, epsilon = 1e-10);
        assert!(diameter_at_middle > 32.0);
        assert!(diameter_at_middle < 60.0);
        assert_abs_diff_eq!(diameter_at_end, 60.0, epsilon = 1e-10);
    }
    
    #[test]
    fn test_volume_calculation() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 10);
        let volume = geo.compute_volume();
        assert!(volume > 0.0);
        // Expected volume for conical frustum: π * h * (r1² + r1*r2 + r2²) / 3
        let expected = PI * 1000.0 * (16.0 * 16.0 + 16.0 * 30.0 + 30.0 * 30.0) / 3.0;
        assert_abs_diff_eq!(volume, expected, epsilon = 1.0); // Allow some numerical error
    }

    #[test]
    fn test_scale_length() {
        let mut geo = Geo::make_cone(1000.0, 32.0, 60.0, 10);
        geo.scale_length(2000.0);
        assert_abs_diff_eq!(geo.length(), 2000.0, epsilon = 1e-10);
        assert_abs_diff_eq!(geo.bellsize(), 60.0, epsilon = 1e-10);
    }

    #[test]
    fn test_scale_diameter() {
        let mut geo = Geo::make_cone(1000.0, 32.0, 60.0, 10);
        geo.scale_diameter(100.0);
        assert_abs_diff_eq!(geo.get_max_d(), 100.0, epsilon = 1e-10);
        assert_abs_diff_eq!(geo.bellsize(), 100.0, epsilon = 1e-10);
    }

    #[test]
    fn test_get_max_d() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 10);
        assert_abs_diff_eq!(geo.get_max_d(), 60.0, epsilon = 1e-10);
    }

    #[test]
    fn test_make_bubble() {
        let mut geo = Geo::make_cone(1000.0, 32.0, 60.0, 10);
        geo.make_bubble(500.0, 100.0, 20.0);
        assert!(geo.geo.len() > 10);
    }

    #[test]
    fn test_diameter_at_x() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 10);
        let d_start = geo.diameter_at_x(0.0);
        let d_middle = geo.diameter_at_x(500.0);
        let d_end = geo.diameter_at_x(1000.0);
        assert!(d_start > 0.0);
        assert!(d_middle > d_start);
        assert!(d_end > d_middle);
    }
}

impl Geo {
    /// Calculate the taper ratio of the didgeridoo
    pub fn taper_ratio(&self) -> f64 {
        if self.geo.is_empty() {
            return 0.0;
        }
        
        // Taper ratio is the ratio of the largest diameter to the smallest diameter
        let max_diameter = self.geo.iter().map(|[_x, d]| *d).fold(0.0_f64, |a, b| a.max(b));
        let min_diameter = self.geo.iter().map(|[_x, d]| *d).fold(f64::MAX, |a, b| a.min(b));
        
        if min_diameter > 0.0 {
            max_diameter / min_diameter
        } else {
            1.0 // Avoid division by zero
        }
    }
}