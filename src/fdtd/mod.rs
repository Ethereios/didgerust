//! Finite-Difference Time-Domain (FDTD) validator for CADSD TLM results.
//!
//! Implements a 1D time-domain transmission line model that can be used to
//! cross-check frequency-domain TLM impedance calculations. The FDTD model
//! uses staggered grids for pressure and volume velocity.

use crate::sim::{AcousticConstants, za};
use num_complex::Complex;
use std::f64::consts::PI;

/// 1D FDTD tube simulator for impedance validation.
///
/// Uses a staggered grid:
/// - Pressure at integer spatial indices (cell centers)
/// - Volume velocity at half-integer spatial indices (cell faces)
///
/// Time stepping uses leapfrog: velocity at n+1 depends on pressure at n,
/// pressure at n+1 depends on velocity at n+1.
pub struct FdtdValidator {
    pub length: f64,
    pub radius: f64,
    pub n_cells: usize,
    pub dx: f64,
    pub dt: f64,
    pub constants: AcousticConstants,
    pub loss_factor: f64,
}

impl FdtdValidator {
    /// Create a new FDTD validator for a cylindrical tube.
    ///
    /// `length` and `radius` in metres. `n_cells` controls spatial resolution.
    /// The time step is chosen automatically for stability (CFL condition).
    pub fn new(length: f64, radius: f64, n_cells: usize, constants: AcousticConstants) -> Self {
        let dx = length / (n_cells as f64);
        let c = constants.c;
        let dt = 0.5 * dx / c; // Conservative CFL-safe time step
        Self {
            length,
            radius,
            n_cells,
            dx,
            dt,
            constants,
            loss_factor: 0.999,
        }
    }

    /// Set the per-step amplitude loss factor (0.0 = fully lossy, 1.0 = lossless).
    pub fn with_loss(mut self, loss_factor: f64) -> Self {
        self.loss_factor = loss_factor.clamp(0.0, 1.0);
        self
    }

    /// Compute input impedance at a single frequency using sinusoidal steady-state.
    ///
    /// The tube is excited with a sinusoidal pressure source at the input (x=0).
    /// After the transient decays, the complex impedance Z(ω) = P(ω) / U(ω)
    /// is computed from the steady-state phasors.
    ///
    /// Returns `None` if the simulation fails to produce valid spectra.
    pub fn impedance_at(&self, freq_hz: f64) -> Option<Complex<f64>> {
        let omega = 2.0 * PI * freq_hz;
        let n_transient = 5000; // steps to reach steady state
        let n_steady = 512;     // steps to average for phasor
        let total_steps = n_transient + n_steady;

        let mut p = vec![0.0; self.n_cells + 1];
        let mut u = vec![0.0; self.n_cells];

        let rho = self.constants.rho;
        let c = self.constants.c;
        let zc = rho * c / (PI * self.radius * self.radius);

        let mut p_cos_sum = 0.0;
        let mut p_sin_sum = 0.0;
        let mut u_cos_sum = 0.0;
        let mut u_sin_sum = 0.0;
        let mut steady_count = 0;

        for step in 0..total_steps {
            let t = step as f64 * self.dt;
            let source = (omega * t).sin() * 1.0; // 1 Pa excitation

            // Update velocity
            for i in 0..self.n_cells {
                let dp_dx = (p[i + 1] - p[i]) / self.dx;
                u[i] = u[i] * self.loss_factor - (self.dt / (rho * self.dx)) * dp_dx;
            }

            // Update pressure
            for i in 1..self.n_cells {
                let du_dx = (u[i] - u[i - 1]) / self.dx;
                p[i] = p[i] * self.loss_factor - (rho * c * c * self.dt / self.dx) * du_dx;
            }

            // Absorbing boundary at open end
            let r_last = self.radius;
            let z_open = za(freq_hz, r_last, rho, c, self.constants.nu);
            let reflection = (z_open.re - zc) / (z_open.re + zc);
            p[self.n_cells] = reflection * p[self.n_cells - 1];

            // Sinusoidal pressure source at input
            p[0] = source;

            // Accumulate steady-state phasors
            if step >= n_transient {
                let phase = omega * t;
                p_cos_sum += p[0] * phase.cos();
                p_sin_sum += p[0] * phase.sin();
                u_cos_sum += u[0] * phase.cos();
                u_sin_sum += u[0] * phase.sin();
                steady_count += 1;
            }
        }

        if steady_count == 0 {
            return None;
        }

        let n = steady_count as f64;
        let p_phasor = Complex::new(p_cos_sum / n, p_sin_sum / n);
        let u_phasor = Complex::new(u_cos_sum / n, u_sin_sum / n);

        if u_phasor.norm() < 1e-15 {
            return None;
        }

        Some(p_phasor / u_phasor)
    }

    /// Compute impedance spectrum over a frequency range.
    pub fn impedance_spectrum(&self, freqs: &[f64]) -> Vec<Complex<f64>> {
        freqs.iter().filter_map(|&f| self.impedance_at(f)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::AcousticConstants;

    #[test]
    fn test_fdtd_construct() {
        let constants = AcousticConstants::for_temperature(20.0);
        let fdtd = FdtdValidator::new(1.5, 0.016, 100, constants);
        assert_eq!(fdtd.n_cells, 100);
        assert!(fdtd.dt > 0.0);
    }
}
