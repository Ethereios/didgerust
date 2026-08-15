pub struct Geo {
    pub geo: Vec<[f64; 2]>,
}

impl Geo {
    // Create a simple cone from length and diameters
    pub fn make_cone(length: f64, d0: f64, d1: f64, n_segments: usize) -> Self {
        let mut points = vec![[0.0, d0]];
        let dx = length / n_segments as f64;
        for i in 1..n_segments {
            let d = d0 + (d1 - d0) * (i as f64) / n_segments as f64;
            points.push([i as f64 * dx, d]);
        }
        points.push([length, d1]);
        Self { geo: points }
    }

    /// Create a Geo from a vector of [x_mm, diameter_mm] points
    pub fn new(points: Vec<[f64; 2]>) -> Self {
        Self { geo: points }
    }

    /// Get diameter at a specific x position (mm along the bore)
    pub fn diameter_at_x(&self, x_mm: f64) -> f64 {
        let x_m = x_mm / 1000.0;
        if self.geo.is_empty() {
            return 0.0;
        }
        if x_m <= 0.0 {
            return self.geo[0][1];
        }
        if x_m >= self.geo.last().unwrap()[0] {
            return self.geo.last().unwrap()[1];
        }
        
        // Find the segment containing x
        for window in self.geo.windows(2) {
            let x0 = window[0][0];
            let x1 = window[1][0];
            if x0 <= x_m && x_m <= x1 {
                let t = (x_m - x0) / (x1 - x0);
                let d0 = window[0][1];
                let d1 = window[1][1];
                return d0 + t * (d1 - d0);
            }
        }
        self.geo.last().unwrap()[1]
    }

    /// Get total length of the bore in mm
    pub fn length(&self) -> f64 {
        if self.geo.is_empty() {
            0.0
        } else {
            self.geo.last().unwrap()[0]
        }
    }

    /// Get bell diameter in mm
    pub fn bellsize(&self) -> f64 {
        if self.geo.is_empty() {
            0.0
        } else {
            self.geo.last().unwrap()[1]
        }
    }
}

impl Default for Geo {
    fn default() -> Self {
        Self { geo: Vec::new() }
    }
}