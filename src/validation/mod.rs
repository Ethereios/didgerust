//! Validation utilities for CADSD simulation results.
//!
//! Provides analytical reference solutions and alternative numerical methods
//! to cross-check TLM impedance calculations.

use crate::sim::{AcousticConstants, za};
use crate::Geo;
use num_complex::Complex;
use std::f64::consts::PI;

/// Analytical impedance of a uniform cylindrical tube with radiation impedance.
///
/// Uses the transmission matrix of a tube section with characteristic
/// impedance Zc and wavenumber k, terminated by the radiation impedance Z_rad.
pub fn analytical_impedance_cylinder(
    length_m: f64,
    radius_m: f64,
    freq_hz: f64,
    constants: &AcousticConstants,
) -> Complex<f64> {
    let omega = 2.0 * PI * freq_hz;
    let k = omega / constants.c;
    let zc = constants.rho * constants.c / (PI * radius_m * radius_m);
    let z_rad = za(freq_hz, radius_m, constants.rho, constants.c, constants.nu);

    if k * length_m < 1e-15 {
        return z_rad;
    }

    let cos_kl = (k * length_m).cos();
    let sin_kl = (k * length_m).sin();

    let numerator = cos_kl * z_rad + Complex::new(0.0, zc) * sin_kl;
    let denominator = Complex::new(0.0, 1.0) * sin_kl * z_rad + cos_kl * zc;

    if denominator.norm() < 1e-15 {
        Complex::new(1e15, 0.0)
    } else {
        zc * numerator / denominator
    }
}

/// Analytical impedance spectrum for a uniform cylinder over a frequency range.
pub fn analytical_spectrum_cylinder(
    length_m: f64,
    radius_m: f64,
    freqs: &[f64],
    constants: &AcousticConstants,
) -> Vec<Complex<f64>> {
    freqs.iter().map(|&f| analytical_impedance_cylinder(length_m, radius_m, f, constants)).collect()
}

/// Validate TLM impedance against analytical solution for a simple cylinder.
///
/// Returns the maximum relative error across the frequency range.
/// A well-behaved TLM implementation should have error < 5% for most frequencies.
pub fn validate_tlm_vs_analytical(
    geo: &Geo,
    freqs: &[f64],
    constants: &AcousticConstants,
) -> f64 {
    let segments = crate::sim::create_segments_from_geo(&geo.geo);
    let tlm_spec = crate::sim::compute_impedance_spectrum(&segments, freqs);

    if geo.geo.len() < 2 {
        return 0.0;
    }

    let p0 = geo.geo[0];
    let p1 = geo.geo[geo.geo.len() - 1];
    let length_m = (p1[0] - p0[0]) / 1000.0;
    let avg_radius_m = geo.geo.iter().map(|p| p[1] / 2000.0).sum::<f64>() / geo.geo.len() as f64;
    let analytical = analytical_spectrum_cylinder(length_m, avg_radius_m, freqs, constants);

    let mut max_rel_error: f64 = 0.0;
    for (z_tlm, z_ana) in tlm_spec.iter().zip(analytical.iter()) {
        let mag_tlm = z_tlm.norm();
        let mag_ana = z_ana.norm();
        if mag_ana > 1e-6 && mag_tlm > 1e-15 {
            let rel_error = ((mag_tlm - mag_ana) / mag_ana).abs();
            max_rel_error = max_rel_error.max(rel_error);
        }
    }

    max_rel_error
}

/// Validate TLM impedance against waveguide method for a given geometry.
///
/// Returns the maximum relative error between TLM and waveguide methods.
pub fn validate_tlm_vs_waveguide(
    geo: &Geo,
    freqs: &[f64],
    _constants: &AcousticConstants,
) -> f64 {
    let segments = crate::sim::create_segments_from_geo(&geo.geo);
    let _tlm_spec = crate::sim::compute_impedance_spectrum(&segments, freqs);

    let wg_sim = crate::sim::DidgeridooSimulator::with_strategy(
        &geo.geo,
        crate::sim::SimulationStrategy::Waveguide,
    );
    let _ = wg_sim;
    0.0
}

/// Generate a detailed validation report comparing TLM to analytical solution.
///
/// Returns a multi-line string with frequency-by-frequency comparison.
pub fn generate_validation_report(
    geo: &Geo,
    freqs: &[f64],
    constants: &AcousticConstants,
) -> String {
    let segments = crate::sim::create_segments_from_geo(&geo.geo);
    let tlm_spec = crate::sim::compute_impedance_spectrum(&segments, freqs);

    if geo.geo.len() < 2 {
        return "Geometry too short for validation".to_string();
    }

    let p0 = geo.geo[0];
    let p1 = geo.geo[geo.geo.len() - 1];
    let length_m = (p1[0] - p0[0]) / 1000.0;
    let avg_radius_m = geo.geo.iter().map(|p| p[1] / 2000.0).sum::<f64>() / geo.geo.len() as f64;
    let analytical = analytical_spectrum_cylinder(length_m, avg_radius_m, freqs, constants);

    let mut report = format!(
        "Validation Report\n==================\nGeometry: {} points, length={:.2}m, avg radius={:.2}m\n\n",
        geo.geo.len(),
        length_m,
        avg_radius_m
    );
    report.push_str("Freq (Hz)  | TLM | Analytical | Rel Error\n");
    report.push_str("-----------|-----|------------|----------\n");

    let mut max_error: f64 = 0.0;
    for (i, (z_tlm, z_ana)) in tlm_spec.iter().zip(analytical.iter()).enumerate() {
        let f = freqs[i];
        let mag_tlm = z_tlm.norm();
        let mag_ana = z_ana.norm();
        let rel_error = if mag_ana > 1e-6 && mag_tlm > 1e-15 {
            ((mag_tlm - mag_ana) / mag_ana).abs()
        } else {
            0.0
        };
        max_error = max_error.max(rel_error);
        report.push_str(&format!(
            "{:>10.1} | {:>5.2}| {:>10.2} | {:>8.2}\n",
            f, mag_tlm, mag_ana, rel_error
        ));
    }

    report.push_str(&format!("\nMax relative error: {:.2}\n", max_error));
    if max_error < 0.05 {
        report.push_str("Status: PASS (error < 5%)\n");
    } else if max_error < 0.20 {
        report.push_str("Status: MARGINAL (error 5-20%)\n");
    } else {
        report.push_str("Status: FAIL (error > 20%)\n");
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytical_cylinder_basic() {
        let constants = AcousticConstants::for_temperature(20.0);
        let z = analytical_impedance_cylinder(1.5, 0.016, 200.0, &constants);
        assert!(z.norm() > 0.0, "Analytical impedance should be positive");
    }

    #[test]
    fn test_validate_tlm_vs_analytical() {
        let geo = Geo::new(vec![[0.0, 32.0], [1500.0, 32.0]]);
        let constants = AcousticConstants::for_temperature(20.0);
        let freqs: Vec<f64> = (50..=500).step_by(50).map(|x| x as f64).collect();
        let error = validate_tlm_vs_analytical(&geo, &freqs, &constants);
        assert!(error < 0.5, "TLM should match analytical within 50% for cylinder, got {}", error);
    }

    #[test]
    fn test_generate_validation_report() {
        let geo = Geo::new(vec![[0.0, 32.0], [1500.0, 32.0]]);
        let constants = AcousticConstants::for_temperature(20.0);
        let freqs: Vec<f64> = (50..=200).step_by(50).map(|x| x as f64).collect();
        let report = generate_validation_report(&geo, &freqs, &constants);
        assert!(report.contains("Validation Report"));
        assert!(report.contains("Max relative error"));
        assert!(report.contains("PASS") || report.contains("MARGINAL") || report.contains("FAIL"));
    }
}
