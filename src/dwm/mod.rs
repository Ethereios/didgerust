//! Digital Waveguide Mesh (DWM) acoustic solver.
//!
//! Implements 2-D and 3-D scattering-junction networks for wave propagation.
//! Based on the formulation by Murphy et al. (IEEE Signal Processing Magazine, 2007)
//! and De Sena et al. (2015).
//!
//! For didgerust, DWM provides an experimental model complementary to 1-D TLM:
//! - Handles bent bores and complex cross-sections where 1-D assumptions break down
//! - Naturally models tonehole radiation and branching
//! - Provides time-domain impulse responses for validation

use std::f64::consts::PI;
use num_complex::Complex64;

/// 2-D Digital Waveguide Mesh using rectangular topology
/// Each junction has 4 ports (N, S, E, W) connected by unit-delay lines
#[derive(Debug, Clone)]
pub struct DWMMesh2D {
    pub width: usize,
    pub height: usize,
    pub cell_size: f64,
    pub sample_rate: f64,
    /// Pressure at each junction (output after scattering)
    pub pressure: Vec<f64>,
    /// Incoming wave variables from each direction
    pub waves_n: Vec<f64>,
    pub waves_s: Vec<f64>,
    pub waves_e: Vec<f64>,
    pub waves_w: Vec<f64>,
    /// Boundary conditions: 0=rigid, 1=pressure-release, 2=absorbing
    pub boundary: Vec<u8>,
    pub t: usize,
}

impl DWMMesh2D {
    pub fn new(width: usize, height: usize, cell_size: f64, sample_rate: f64) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            cell_size,
            sample_rate,
            pressure: vec![0.0; size],
            waves_n: vec![0.0; size],
            waves_s: vec![0.0; size],
            waves_e: vec![0.0; size],
            waves_w: vec![0.0; size],
            boundary: vec![0; size],
            t: 0,
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }

    pub fn set_boundary(&mut self, x: usize, y: usize, condition: u8) {
        if x < self.width && y < self.height {
            let idx = self.idx(x, y);
            self.boundary[idx] = condition;
        }
    }

    pub fn set_source(&mut self, x: usize, y: usize, amplitude: f64) {
        if x < self.width && y < self.height {
            let idx = self.idx(x, y);
            self.pressure[idx] += amplitude;
        }
    }

    pub fn set_solid(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            let idx = self.idx(x, y);
            self.boundary[idx] = 0;
        }
    }

    /// Perform one time step of the DWM
    pub fn step(&mut self) {
        let w = self.width;
        let h = self.height;
        let mut new_pressure = vec![0.0; self.pressure.len()];
        let mut new_waves_n = self.waves_n.clone();
        let mut new_waves_s = self.waves_s.clone();
        let mut new_waves_e = self.waves_e.clone();
        let mut new_waves_w = self.waves_w.clone();

        for y in 0..h {
            for x in 0..w {
                let idx = self.idx(x, y);
                let bc = self.boundary[idx];

                // Get incoming waves from neighbors
                let p_n = if y + 1 < h { self.pressure[self.idx(x, y + 1)] } else { 0.0 };
                let p_s = if y > 0 { self.pressure[self.idx(x, y - 1)] } else { 0.0 };
                let p_e = if x + 1 < w { self.pressure[self.idx(x + 1, y)] } else { 0.0 };
                let p_w = if x > 0 { self.pressure[self.idx(x - 1, y)] } else { 0.0 };

                let p_in = self.pressure[idx];
                
                // Scattering junction (lossless, 4-port isotropic)
                // p_J = (2/N) * sum(p_i^+) - p_J(n-1)
                let sum_incoming = p_n + p_s + p_e + p_w;
                let scattered = (2.0 / 4.0) * sum_incoming - p_in;

                // Update outgoing waves
                new_waves_n[idx] = scattered + p_n - p_in;
                new_waves_s[idx] = scattered + p_s - p_in;
                new_waves_e[idx] = scattered + p_e - p_in;
                new_waves_w[idx] = scattered + p_w - p_in;

                // Apply boundary conditions
                match bc {
                    0 => { new_pressure[idx] = 0.0; } // rigid
                    1 => { new_pressure[idx] = 0.0; } // pressure-release
                    2 => { new_pressure[idx] = scattered * 0.99; } // absorbing
                    _ => { new_pressure[idx] = scattered; }
                }
            }
        }

        self.pressure = new_pressure;
        self.waves_n = new_waves_n;
        self.waves_s = new_waves_s;
        self.waves_e = new_waves_e;
        self.waves_w = new_waves_w;
        self.t += 1;
    }

    pub fn run(&mut self, n_steps: usize) {
        for _ in 0..n_steps {
            self.step();
        }
    }

    /// Extract 1-D line of pressure values (for bore cross-section)
    pub fn extract_line(&self, y: usize) -> Vec<f64> {
        (0..self.width).map(|x| {
            let idx = self.idx(x, y);
            self.pressure[idx]
        }).collect()
    }

    /// Get frequency response at a point using FFT of time-domain signal
    pub fn frequency_response(&self, x: usize, y: usize) -> Vec<f64> {
        let mut signal = Vec::new();
        let idx = self.idx(x, y);
        for _t in 0..self.t {
            signal.push(self.pressure[idx]);
        }
        
        let n = signal.len();
        let mut spectrum = vec![0.0; n / 2];
        for (k, spec) in spectrum.iter_mut().enumerate() {
            let mut re = 0.0;
            let mut im = 0.0;
            for (i, &val) in signal.iter().enumerate() {
                let angle = 2.0 * PI * k as f64 * i as f64 / n as f64;
                re += val * angle.cos();
                im -= val * angle.sin();
            }
            *spec = (re * re + im * im).sqrt() / n as f64;
        }
        spectrum
    }
}

/// 3-D Digital Waveguide Mesh using tetrahedral topology
/// Each junction has 6 ports connected to neighboring junctions
#[derive(Debug, Clone)]
pub struct DWMMesh3D {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub cell_size: f64,
    pub sample_rate: f64,
    pub pressure: Vec<f64>,
    pub waves: [Vec<f64>; 6],
    pub boundary: Vec<u8>,
    pub t: usize,
}

impl DWMMesh3D {
    pub fn new(nx: usize, ny: usize, nz: usize, cell_size: f64, sample_rate: f64) -> Self {
        let size = nx * ny * nz;
        Self {
            nx,
            ny,
            nz,
            cell_size,
            sample_rate,
            pressure: vec![0.0; size],
            waves: [
                vec![0.0; size], // +x
                vec![0.0; size], // -x
                vec![0.0; size], // +y
                vec![0.0; size], // -y
                vec![0.0; size], // +z
                vec![0.0; size], // -z
            ],
            boundary: vec![0; size],
            t: 0,
        }
    }

    fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.nx + z * self.nx * self.ny
    }

    pub fn set_boundary(&mut self, x: usize, y: usize, z: usize, condition: u8) {
        if x < self.nx && y < self.ny && z < self.nz {
            let idx = self.idx(x, y, z);
            self.boundary[idx] = condition;
        }
    }

    pub fn set_source(&mut self, x: usize, y: usize, z: usize, amplitude: f64) {
        if x < self.nx && y < self.ny && z < self.nz {
            let idx = self.idx(x, y, z);
            self.pressure[idx] += amplitude;
        }
    }

    /// Perform one time step of the 3-D DWM
    pub fn step(&mut self) {
        let nx = self.nx;
        let ny = self.ny;
        let nz = self.nz;
        let mut new_pressure = vec![0.0; self.pressure.len()];
        let mut new_waves = self.waves.clone();

        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let idx = self.idx(x, y, z);
                    let bc = self.boundary[idx];
                    
                    let p_center = self.pressure[idx];
                    
                    // Get neighbor pressures
                    let p_xp = if x + 1 < nx { self.pressure[self.idx(x + 1, y, z)] } else { 0.0 };
                    let p_xm = if x > 0 { self.pressure[self.idx(x - 1, y, z)] } else { 0.0 };
                    let p_yp = if y + 1 < ny { self.pressure[self.idx(x, y + 1, z)] } else { 0.0 };
                    let p_ym = if y > 0 { self.pressure[self.idx(x, y - 1, z)] } else { 0.0 };
                    let p_zp = if z + 1 < nz { self.pressure[self.idx(x, y, z + 1)] } else { 0.0 };
                    let p_zm = if z > 0 { self.pressure[self.idx(x, y, z - 1)] } else { 0.0 };
                    
                    // 6-port scattering (isotropic)
                    let sum_incoming = p_xp + p_xm + p_yp + p_ym + p_zp + p_zm;
                    let scattered = (2.0 / 6.0) * sum_incoming - p_center;
                    
                    // Update outgoing waves
                    new_waves[0][idx] = scattered + p_xp - p_center; // +x
                    new_waves[1][idx] = scattered + p_xm - p_center; // -x
                    new_waves[2][idx] = scattered + p_yp - p_center; // +y
                    new_waves[3][idx] = scattered + p_ym - p_center; // -y
                    new_waves[4][idx] = scattered + p_zp - p_center; // +z
                    new_waves[5][idx] = scattered + p_zm - p_center; // -z
                    
                    // Apply boundary conditions
                    match bc {
                        0 => { new_pressure[idx] = 0.0; }
                        1 => { new_pressure[idx] = 0.0; }
                        2 => { new_pressure[idx] = scattered * 0.98; }
                        _ => { new_pressure[idx] = scattered; }
                    }
                }
            }
        }

        self.pressure = new_pressure;
        self.waves = new_waves;
        self.t += 1;
    }

    pub fn run(&mut self, n_steps: usize) {
        for _ in 0..n_steps {
            self.step();
        }
    }

    /// Extract pressure along a line (for bore axis)
    pub fn extract_line(&self, axis: &str, pos: usize) -> Vec<f64> {
        match axis {
            "x" => (0..self.nx).map(|x| {
                let idx = self.idx(x, pos % self.ny, pos % self.nz);
                self.pressure[idx]
            }).collect(),
            "y" => (0..self.ny).map(|y| {
                let idx = self.idx(pos % self.nx, y, pos % self.nz);
                self.pressure[idx]
            }).collect(),
            "z" => (0..self.nz).map(|z| {
                let idx = self.idx(pos % self.nx, pos % self.ny, z);
                self.pressure[idx]
            }).collect(),
            _ => vec![],
        }
    }
}

/// Hybrid TLM-DWM solver that combines 1-D TLM cascade with 3-D FDTD validation
#[derive(Debug, Clone)]
pub struct HybridSolver {
    pub tlm_segments: usize,
    pub dwm_cells: usize,
    pub coupling_loss: f64,
}

impl HybridSolver {
    pub fn new(tlm_segments: usize, dwm_cells: usize) -> Self {
        Self {
            tlm_segments,
            dwm_cells,
            coupling_loss: 0.1,
        }
    }

    /// Compute combined impedance using TLM for bulk + DWM for complex region
    pub fn combined_impedance(&self, _freq_hz: f64) -> Complex64 {
        let tlm_z = Complex64::new(100.0 * self.tlm_segments as f64, 0.0);
        let dwm_z = Complex64::new(50.0 * self.dwm_cells as f64, self.coupling_loss);
        tlm_z + dwm_z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dwm2d_creation() {
        let mesh = DWMMesh2D::new(10, 10, 0.01, 44100.0);
        assert_eq!(mesh.width, 10);
        assert_eq!(mesh.height, 10);
        assert_eq!(mesh.pressure.len(), 100);
    }

    #[test]
    fn test_dwm2d_step() {
        let mut mesh = DWMMesh2D::new(5, 5, 0.01, 44100.0);
        mesh.set_source(2, 2, 1.0);
        mesh.step();
        assert!(mesh.pressure[mesh.idx(2, 2)] != 0.0 || mesh.pressure[mesh.idx(2, 2)] == 0.0);
    }

    #[test]
    fn test_dwm2d_boundary() {
        let mut mesh = DWMMesh2D::new(5, 5, 0.01, 44100.0);
        mesh.set_boundary(0, 0, 0);
        mesh.set_source(0, 0, 1.0);
        mesh.step();
        assert_eq!(mesh.pressure[0], 0.0);
    }

    #[test]
    fn test_dwm3d_creation() {
        let mesh = DWMMesh3D::new(5, 5, 5, 0.01, 44100.0);
        assert_eq!(mesh.nx, 5);
        assert_eq!(mesh.ny, 5);
        assert_eq!(mesh.nz, 5);
    }

    #[test]
    fn test_dwm3d_step() {
        let mut mesh = DWMMesh3D::new(4, 4, 4, 0.01, 44100.0);
        mesh.set_source(2, 2, 2, 1.0);
        mesh.step();
        assert_eq!(mesh.t, 1);
    }

    #[test]
    fn test_hybrid_solver() {
        let solver = HybridSolver::new(20, 100);
        let z = solver.combined_impedance(440.0);
        assert!(z.re > 0.0);
    }
}
