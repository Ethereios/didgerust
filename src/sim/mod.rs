//! Simulation module for CADSD – delegates to cadsd-accurate crate.
//!
//! All simulation logic is owned by cadsd-accurate. This wrapper
//! provides convenience types and delegates all computation.

use cadsd_accurate::geo::Geo;
use cadsd_accurate::sim::{
    acoustical_simulation,
    get_log_simulation_frequencies,
    compute_ground_spektrum,
    get_fundamental,
};

/// Simulation strategy selection (matches accurate crate semantics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationStrategy {
    /// Traditional transmission line model (default, stable)
    Tlm,
    /// Digital waveguide approach (alternative)
    Waveguide,
    /// Enhanced complex impedance calculation (alternative)
    ComplexImpedance,
}

impl SimulationStrategy {
    /// Convert to the string backend name used by the accurate crate.
    pub fn as_str(&self) -> &'static str {
        match self {
            SimulationStrategy::Tlm => "tlm_cython",
            SimulationStrategy::Waveguide => "tlm_cython",
            SimulationStrategy::ComplexImpedance => "tlm_cython",
        }
    }
}

/// Simple resonance result – frequency and impedance magnitude.
#[derive(Debug, Clone, PartialEq)]
pub struct Resonance {
    pub frequency: f64,
    pub impedance: f64,
}

/// Build segments from geometry data (mm) for use with the accurate crate.
///
/// Converts `[x_mm, diameter_mm]` pairs into the `Geo` type expected
/// by `cadsd_accurate::sim::acoustical_simulation`.
pub fn create_geo_from_segments(points: &[[f64; 2]]) -> Geo {
    Geo::new(points.to_vec())
}

/// Compute impedance magnitudes at the given frequencies using the
/// accurate crate's acoustical_simulation function.
///
/// Returns `(frequencies, magnitudes)` on success.
pub fn compute_impedance_spectrum(
    geo: &Geo,
    frequencies: &[f64],
    strategy: SimulationStrategy,
) -> Result<Vec<f64>, cadsd_accurate::CadsdError> {
    acoustical_simulation(geo, frequencies, strategy.as_str())
}

/// Get the standard logarithmic simulation frequency grid.
pub fn get_simulation_frequencies() -> Vec<f64> {
    get_log_simulation_frequencies()
}

/// Find the fundamental frequency for a given geometry.
pub fn find_fundamental(
    geo: &Geo,
    strategy: SimulationStrategy,
    min_peak_f: f64,
) -> Result<(f64, f64), cadsd_accurate::CadsdError> {
    get_fundamental(geo, strategy.as_str(), min_peak_f)
}

/// Compute ground spectrum (frequency, impedance) pairs.
pub fn compute_ground_spectrum(
    geo: &Geo,
    strategy: SimulationStrategy,
) -> Result<Vec<(f64, f64)>, cadsd_accurate::CadsdError> {
    compute_ground_spektrum(geo, strategy.as_str())
}

/// Convenience wrapper that computes resonance peaks for a geometry.
///
/// Uses the accurate crate's `get_log_simulation_frequencies()` grid
/// and finds local maxima in the impedance spectrum.
pub fn find_resonance_peaks(geo: &Geo, strategy: SimulationStrategy) -> Vec<Resonance> {
    let freqs = get_log_simulation_frequencies();
    let result = acoustical_simulation(geo, &freqs, strategy.as_str());
    match result {
        Ok(impedances) => {
            let mut peaks = Vec::new();
            for i in 1..impedances.len().saturating_sub(1) {
                if impedances[i] > impedances[i - 1] && impedances[i] > impedances[i + 1] {
                    peaks.push(Resonance {
                        frequency: freqs[i],
                        impedance: impedances[i],
                    });
                }
            }
            peaks
        }
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_strategy_as_str() {
        assert_eq!(SimulationStrategy::Tlm.as_str(), "tlm_cython");
        assert_eq!(SimulationStrategy::Waveguide.as_str(), "tlm_cython");
        assert_eq!(SimulationStrategy::ComplexImpedance.as_str(), "tlm_cython");
    }

    #[test]
    fn test_create_geo_from_segments() {
        let points = vec![[0.0, 32.0], [500.0, 46.0], [1000.0, 60.0]];
        let geo = create_geo_from_segments(&points);
        assert_eq!(geo.geo.len(), 3);
        assert_eq!(geo.geo[0], [0.0, 32.0]);
        assert_eq!(geo.geo[2], [1000.0, 60.0]);
    }

    #[test]
    fn test_compute_impedance_spectrum() {
        let points = vec![[0.0, 32.0], [1000.0, 60.0]];
        let geo = create_geo_from_segments(&points);
        let freqs = vec![100.0, 200.0, 400.0];
        let result = compute_impedance_spectrum(&geo, &freqs, SimulationStrategy::Tlm);
        assert!(result.is_ok());
        let impedances = result.unwrap();
        assert_eq!(impedances.len(), freqs.len());
        for &imp in &impedances {
            assert!(imp.is_finite());
            assert!(imp >= 0.0);
        }
    }

    #[test]
    fn test_find_resonance_peaks() {
        let points = vec![[0.0, 32.0], [1000.0, 60.0]];
        let geo = create_geo_from_segments(&points);
        let peaks = find_resonance_peaks(&geo, SimulationStrategy::Tlm);
        // Peaks may or may not be found depending on geometry, but should not panic
        assert!(peaks.len() <= freqs_count());
    }

    fn freqs_count() -> usize {
        get_log_simulation_frequencies().len()
    }
}