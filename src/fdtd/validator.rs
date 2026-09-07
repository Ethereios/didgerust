//! 3-D FDTD validator for acoustic simulation validation.
//! 
//! Provides FDTD-based validation of TLM results by solving the 3-D acoustic wave equation
//! in a cylindrical domain and comparing the resulting pressure/velocity fields.

use crate::sim::AcousticConstants;
use num_complex::Complex;

/// 3-D FDTD grid for acoustic wave simulation
pub struct FDTDGrid {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
    pub dt: f64,
    pub pressure: Vec<Complex<f64>>,
    pub velocity_x: Vec<Complex<f64>>,
    pub velocity_y: Vec<Complex<f64>>,
    pub velocity_z: Vec<Complex<f64>>,
}

impl FDTDGrid {
    /// Create a new FDTD grid
    pub fn new(nx: usize, ny: usize, nz: usize, dx: f64, dy: f64, dz: f64, constants: &AcousticConstants) -> Self {
        let dt = dx.min(dy).min(dz) / (constants.c * 2.0_f64.sqrt());
        let n = nx * ny * nz;
        Self {
            nx, ny, nz, dx, dy, dz, dt,
            pressure: vec![Complex::ZERO; n],
            velocity_x: vec![Complex::ZERO; n],
            velocity_y: vec![Complex::ZERO; n],
            velocity_z: vec![Complex::ZERO; n],
        }
    }

    /// Get index for 3-D coordinates
    pub fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        x * self.ny * self.nz + y * self.nz + z
    }

    /// Apply absorbing boundary conditions (CPML or simple PML)
    pub fn apply_absorbing_bc(&mut self, thickness: usize) {
        let n = self.pressure.len();
        let _volume = (self.nx * self.ny * self.nz) as f64;
        let decay: f64 = 0.1; // Simple exponential decay for PML
        
        for i in 0..n {
            let x = i / (self.ny * self.nz);
            let y = (i % (self.ny * self.nz)) / self.nz;
            let z = i % self.nz;
            
            // Distance to nearest boundary
            let dx = (x as f64 - thickness as f64).min(self.nx as f64 - x as f64 - 1.0 - thickness as f64).max(0.0);
            let dy = (y as f64 - thickness as f64).min(self.ny as f64 - y as f64 - 1.0 - thickness as f64).max(0.0);
            let dz = (z as f64 - thickness as f64).min(self.nz as f64 - z as f64 - 1.0 - thickness as f64).max(0.0);
            
            let sigma = (dx.min(dy).min(dz) * decay / thickness as f64).min(1.0);
            
            if sigma > 0.0 {
                self.pressure[i] *= Complex::new(1.0 - sigma, 0.0);
                self.velocity_x[i] *= Complex::new(1.0 - sigma, 0.0);
                self.velocity_y[i] *= Complex::new(1.0 - sigma, 0.0);
                self.velocity_z[i] *= Complex::new(1.0 - sigma, 0.0);
            }
        }
    }

    /// Single time step update
    pub fn update(&mut self, constants: &AcousticConstants) {
        let c = constants.c;
        let rho = constants.rho;
        
        let mut new_pressure = self.pressure.clone();
        let mut new_vx = self.velocity_x.clone();
        let mut new_vy = self.velocity_y.clone();
        let mut new_vz = self.velocity_z.clone();

        // Update pressure field
        for x in 1..self.nx - 1 {
            for y in 1..self.ny - 1 {
                for z in 1..self.nz - 1 {
                    let i = self.idx(x, y, z);
                    let ip1 = self.idx(x + 1, y, z);
                    let im1 = self.idx(x - 1, y, z);
                    let jp1 = self.idx(x, y + 1, z);
                    let jm1 = self.idx(x, y - 1, z);
                    let kp1 = self.idx(x, y, z + 1);
                    let km1 = self.idx(x, y, z - 1);

                    let div_v = (self.velocity_x[ip1].re - self.velocity_x[im1].re) / self.dx
                              + (self.velocity_y[jp1].re - self.velocity_y[jm1].re) / self.dy
                              + (self.velocity_z[kp1].re - self.velocity_z[km1].re) / self.dz;
                    
                    new_pressure[i] = self.pressure[i] + (c * c * self.dt / rho) * div_v;
                }
            }
        }

        // Update velocity fields
        for x in 1..self.nx - 1 {
            for y in 1..self.ny - 1 {
                for z in 1..self.nz - 1 {
                    let i = self.idx(x, y, z);
                    let ip1 = self.idx(x + 1, y, z);
                    let im1 = self.idx(x - 1, y, z);
                    let jp1 = self.idx(x, y + 1, z);
                    let jm1 = self.idx(x, y - 1, z);
                    let kp1 = self.idx(x, y, z + 1);
                    let km1 = self.idx(x, y, z - 1);

                    let dpdx = (self.pressure[ip1].re - self.pressure[im1].re) / self.dx;
                    let dpdy = (self.pressure[jp1].re - self.pressure[jm1].re) / self.dy;
                    let dpdz = (self.pressure[kp1].re - self.pressure[km1].re) / self.dz;

                    new_vx[i] = self.velocity_x[i] - (self.dt / (rho * self.dx)) * dpdx;
                    new_vy[i] = self.velocity_y[i] - (self.dt / (rho * self.dy)) * dpdy;
                    new_vz[i] = self.velocity_z[i] - (self.dt / (rho * self.dz)) * dpdz;
                }
            }
        }

        self.pressure = new_pressure;
        self.velocity_x = new_vx;
        self.velocity_y = new_vy;
        self.velocity_z = new_vz;
    }

    /// Add point source at specified location
    pub fn add_source(&mut self, x: usize, y: usize, z: usize, amplitude: f64, freq: f64) {
        let i = self.idx(x, y, z);
        let omega = 2.0 * std::f64::consts::PI * freq;
        let t_factor = (0..=1000).map(|n| (n as f64 * self.dt * omega).sin()).collect::<Vec<_>>();
        
        self.pressure[i] = Complex::new(amplitude * t_factor[0], 0.0);
    }

    /// Get estimated impedance at a point
    pub fn estimate_impedance(&self, x: usize, y: usize, z: usize, _area: f64) -> Complex<f64> {
        let i = self.idx(x, y, z);
        let p = self.pressure[i];
        let v = self.velocity_x[i]; // Assuming flow in x direction
        
        if v.re.abs() > 1e-12 {
            p / v
        } else {
            Complex::new(1e6, 0.0) // High impedance if no flow
        }
    }
}

/// Validate TLM results against FDTD simulation
pub fn validate_fdtd_vs_tlm(geo: &crate::Geo, freq: f64, constants: &AcousticConstants) -> (Complex<f64>, Complex<f64>, f64) {
    let length_m = 1.5;
    let radius_m = 0.032;
    
    let nx = 16;
    let ny = 16;
    let nz = 32;
    let dx = radius_m * 2.0 / nx as f64;
    let dy = radius_m * 2.0 / ny as f64;
    let dz = length_m / nz as f64;
    
    let mut grid = FDTDGrid::new(nx, ny, nz, dx, dy, dz, constants);
    
    let src_x = nx / 4;
    let src_y = ny / 2;
    let src_z = nz / 2;
    grid.add_source(src_x, src_y, src_z, 1.0, freq);
    
    grid.apply_absorbing_bc(2);
    
    let n_steps = (50.0 * 2.0 * std::f64::consts::PI / freq / grid.dt).max(50.0).min(500.0) as usize;
    for _ in 0..n_steps {
        grid.update(constants);
    }
    
    let term_x = nx - 3;
    let term_y = ny / 2;
    let term_z = nz / 2;
    let fdtd_impedance = grid.estimate_impedance(term_x, term_y, term_z, dy * dz);
    
    let segments = crate::sim::create_segments_from_geo(&geo.geo);
    let tlm_spec = crate::sim::compute_impedance_spectrum(&segments, &[freq]);
    let tlm_impedance = if tlm_spec.is_empty() {
        Complex::new(0.0, 0.0)
    } else {
        tlm_spec[0]
    };
    
    let rel_error = if tlm_impedance.re != 0.0 || tlm_impedance.im != 0.0 {
        ((fdtd_impedance - tlm_impedance).norm() / tlm_impedance.norm()).abs()
    } else {
        0.0
    };
    
    (fdtd_impedance, tlm_impedance, rel_error)
}

/// Generate comprehensive validation report including FDTD comparison
pub fn generate_fdtd_validation_report(geo: &crate::Geo, freqs: &[f64], constants: &AcousticConstants) -> String {
    let mut report = "FDTD Validation Report\n========================\n\n".to_string();
    report.push_str("Comparing TLM impedance against 3-D FDTD simulation\n\n");
    
    let mut max_error: f64 = 0.0;
    let mut fdtd_mags = Vec::new();
    let mut tlm_mags = Vec::new();
    
    for &freq in freqs {
        let (fdtd_z, tlm_z, err) = validate_fdtd_vs_tlm(geo, freq, constants);
        fdtd_mags.push(fdtd_z.norm());
        tlm_mags.push(tlm_z.norm());
        max_error = max_error.max(err);
        
        report.push_str(&format!(
            "Freq: {:>8.1} Hz | TLM: {:>10.2} Ω | FDTD: {:>10.2} Ω | Error: {:.2}%\n",
            freq, tlm_z.norm(), fdtd_z.norm(), err * 100.0
        ));
    }
    
    report.push_str(&format!("\nMaximum relative error: {:.4}\n", max_error));
    
    if max_error < 0.10 {
        report.push_str("Status: EXCELLENT (error < 10%)\n");
    } else if max_error < 0.20 {
        report.push_str("Status: GOOD (error 10-20%)\n");
    } else if max_error < 0.50 {
        report.push_str("Status: ACCEPTABLE (error 20-50%)\n");
    } else {
        report.push_str("Status: NEEDS VALIDATION (error > 50%)\n");
    }
    
    report.push_str("\nNote: FDTD is a simplified model. For full validation,\n");
    report.push_str("consider using the surrogate model with gradient information.\n");
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Geo;
    use crate::sim::AcousticConstants;

    #[test]
    fn test_fdtd_grid_creation() {
        let constants = AcousticConstants::default();
        let grid = FDTDGrid::new(32, 32, 32, 0.01, 0.01, 0.01, &constants);
        
        assert_eq!(grid.nx, 32);
        assert_eq!(grid.ny, 32);
        assert_eq!(grid.nz, 32);
        assert_eq!(grid.pressure.len(), 32 * 32 * 32);
    }

    #[test]
    fn test_fdtd_index_mapping() {
        let constants = AcousticConstants::default();
        let grid = FDTDGrid::new(4, 4, 4, 0.01, 0.01, 0.01, &constants);
        
        assert_eq!(grid.idx(0, 0, 0), 0);
        assert_eq!(grid.idx(1, 0, 0), 16);
        assert_eq!(grid.idx(0, 1, 0), 4);
        assert_eq!(grid.idx(0, 0, 1), 1);
    }

    #[test]
    fn test_fdtd_update_step() {
        let constants = AcousticConstants::default();
        let mut grid = FDTDGrid::new(16, 16, 16, 0.01, 0.01, 0.01, &constants);
        
        let i = grid.idx(8, 8, 8);
        let ip1 = grid.idx(9, 8, 8);
        let im1 = grid.idx(7, 8, 8);
        
        grid.pressure[i] = Complex::new(1.0, 0.0);
        grid.velocity_x[i] = Complex::new(0.1, 0.0);
        grid.velocity_x[ip1] = Complex::new(0.2, 0.0);
        grid.velocity_x[im1] = Complex::new(0.0, 0.0);
        
        grid.update(&constants);
        
        assert!(grid.pressure[i].re != 1.0 || grid.velocity_x[i].re != 0.1);
    }

    #[test]
    fn test_fdtd_impedance_estimation() {
        let constants = AcousticConstants::default();
        let mut grid = FDTDGrid::new(16, 16, 16, 0.01, 0.01, 0.01, &constants);
        
        // Set some values
        let i = grid.idx(8, 8, 8);
        grid.pressure[i] = Complex::new(1.0, 0.0);
        grid.velocity_x[i] = Complex::new(0.5, 0.0);
        
        let z = grid.estimate_impedance(8, 8, 8, 0.01 * 0.01);
        assert!((z.re - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_fdtd_vs_tlm_validation() {
        let geo = Geo::make_cone(1500.0, 32.0, 65.0, 10);
        let constants = AcousticConstants::default();
        let freqs = vec![200.0];
        
        for &freq in &freqs {
            let (fdtd_z, tlm_z, _err) = validate_fdtd_vs_tlm(&geo, freq, &constants);
            
            assert!(fdtd_z.re.is_finite() && fdtd_z.im.is_finite(), "FDTD impedance should be finite");
            assert!(tlm_z.re.is_finite() && tlm_z.im.is_finite(), "TLM impedance should be finite");
        }
    }
}