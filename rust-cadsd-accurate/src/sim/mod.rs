//! Acoustical simulation entry point for CADSD
//!
//! This module provides the exact same interface as the Python DidgeLab acoustical_simulation module.
//! It computes acoustic impedance at the given frequencies for a didgeridoo geometry using
//! transmission-line-based acoustical simulation.

use crate::geo::Geo;
use crate::CadsdError;
use num_complex::Complex64;
use std::f64::consts::PI;

/// Physical constants (same as Python implementation)
const AIR_DENSITY: f64 = 1.2929;      // kg/m³
const AIR_VISCOSITY: f64 = 1.708e-5;  // Pa·s
const SPEED_OF_SOUND: f64 = 343.37;   // m/s
const EPS_GEO: f64 = 1e-12;           // Minimum length/diameter to avoid zero division

/// Compute acoustic impedance at the given frequencies for a didgeridoo geometry
///
/// Uses a transmission-line model of the bore. Impedance peaks correspond to
/// resonances (drone, toots). The result has the same length as `frequencies`.
///
/// # Arguments
/// * `geo` - Didgeridoo geometry (bore profile as list of segments)
/// * `frequencies` - 1D array of frequencies in Hz at which to evaluate impedance
/// * `simulation_method` - Which backend to use ("tlm_python" or "tlm_cython")
///
/// # Returns
/// Impedance magnitude at each frequency, in the same order as `frequencies`
pub fn acoustical_simulation(
    geo: &Geo,
    frequencies: &[f64],
    simulation_method: &str,
) -> Result<Vec<f64>, CadsdError> {
    match simulation_method {
        "tlm_python" => {
            let segments = create_segments_from_geo(&geo.geo);
            let impedances: Vec<f64> = frequencies
                .iter()
                .map(|&freq| cadsd_ze(&segments, freq))
                .collect();
            Ok(impedances)
        }
        "tlm_cython" => {
            // In the real implementation, this would call the Cython backend
            // For now, we'll use the same Python implementation
            let segments = create_segments_from_geo(&geo.geo);
            let impedances: Vec<f64> = frequencies
                .iter()
                .map(|&freq| cadsd_ze(&segments, freq))
                .collect();
            Ok(impedances)
        }
        _ => Err(CadsdError::SimulationError(
            format!("Unknown simulation method: {}", simulation_method)
        )),
    }
}

/// Segment structure for transmission line modeling (same as Python Cython)
#[derive(Debug, Clone)]
struct Segment {
    l: f64,       // length (m)
    d0: f64,      // input diameter (m)
    d1: f64,      // output diameter (m)
    a0: f64,      // input area (m²)
    a01: f64,     // intermediate area (m²)
    _a1: f64,      // output area (m²)
    phi: f64,     // cone angle (radians)
    x0: f64,      // x0 coordinate
    x1: f64,      // x1 coordinate
    r0: f64,      // characteristic impedance
}

impl Segment {
    /// Create a new segment (same logic as Python Cython)
    fn new(mut length: f64, mut d0: f64, mut d1: f64) -> Self {
        // Avoid zero division (same as Python)
        if length <= 0.0 {
            length = EPS_GEO;
        }
        if d0 <= 0.0 {
            d0 = EPS_GEO;
        }
        if d1 <= 0.0 {
            d1 = EPS_GEO;
        }
        
        let a0 = PI * d0 * d0 / 4.0;
        let a01 = PI * (d0 + d1) * (d0 + d1) / 16.0;
        let a1 = PI * d1 * d1 / 4.0;
        let phi = ((d1 - d0) / (2.0 * length)).atan();
        
        let (x0, x1, _l_cone) = if (d1 - d0).abs() < 1e-12 {
            // Cylindrical segment
            (0.0, 0.0, f64::NAN)
        } else {
            // Conical segment
            let sin_phi = (2.0 * phi).sin();
            let l_val = if sin_phi.abs() < 1e-12 { 
                f64::NAN 
            } else { 
                (d1 - d0) / (2.0 * sin_phi) 
            };
            let x1_val = if sin_phi.abs() < 1e-12 { 
                f64::NAN 
            } else { 
                d1 / (2.0 * sin_phi) 
            };
            let x0_val = x1_val - l_val;
            (x0_val, x1_val, l_val)
        };
        
        let r0 = AIR_DENSITY * SPEED_OF_SOUND / a0;
        
        Self {
            l: length,
            d0,
            d1,
            a0,
            a01,
            _a1: a1,
            phi,
            x0,
            x1,
            r0,
        }
    }
}

/// Create segments from geometry (list of [x_mm, diameter_mm]); converts mm to m
fn create_segments_from_geo(geo: &[[f64; 2]]) -> Vec<Segment> {
    let mut segments = Vec::new();
    
    // Convert from mm to m (same as Python)
    let shape: Vec<[f64; 2]> = geo.iter()
        .map(|&[x, d]| [x / 1000.0, d / 1000.0])
        .collect();
    
    for i in 1..shape.len() {
        let seg1 = shape[i];
        let seg0 = shape[i-1];
        let length = seg1[0] - seg0[0];
        let d0 = seg0[1];
        let d1 = seg1[1];
        
        segments.push(Segment::new(length, d0, d1));
    }
    
    segments
}

/// Compute transfer matrix for angular frequency w (same as Python)
fn ap(w: f64, segments: &[Segment]) -> [[Complex64; 2]; 2] {
    let mut x = [[Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
                 [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)]];
    
    for segment in segments {
        let y = ap_segment(w, segment);
        
        // Matrix multiplication (same as Python)
        let z = [
            [
                x[0][0] * y[0][0] + x[0][1] * y[1][0],
                x[0][0] * y[0][1] + x[0][1] * y[1][1]
            ],
            [
                x[1][0] * y[0][0] + x[1][1] * y[1][0],
                x[1][0] * y[0][1] + x[1][1] * y[1][1]
            ]
        ];
        
        x = z;
    }
    
    x
}

/// Compute transfer matrix for single segment (same as Python Cython)
fn ap_segment(w: f64, segment: &Segment) -> [[Complex64; 2]; 2] {
    let l = segment.l;
    let d0 = segment.d0;
    let d1 = segment.d1;
    let _a0 = segment.a0;
    let a01 = segment.a01;
    let x0 = segment.x0;
    let x1 = segment.x1;
    let r0 = segment.r0;
    
    // Viscothermal loss parameters (same as Python)
    let rvw = (AIR_DENSITY * w * a01 / (AIR_VISCOSITY * PI)).sqrt();
    let kw = w / SPEED_OF_SOUND;
    let tw = kw * 1.045 / rvw + (kw * (1.0 + 1.045 / rvw)) * Complex64::i();
    let zcw = r0 * (1.0 + 0.369 / rvw) - Complex64::i() * r0 * 0.369 / rvw;
    
    if (d0 - d1).abs() < 1e-12 {
        // Cylindrical segment (same as Python)
        let ccoshlwl = (tw * l).cosh();
        let csinhlwl = (tw * l).sinh();
        
        [
            [ccoshlwl, zcw * csinhlwl],
            [csinhlwl / zcw, ccoshlwl]
        ]
    } else {
        // Conical segment (same as Python)
        let l_cone = if (d1 - d0).abs() < 1e-12 { 0.0 } else { (d1 - d0) / (2.0 * segment.phi.sin()) };
        let ccoshlwl = (tw * l_cone).cosh();
        let csinhlwl = (tw * l_cone).sinh();
        
        let y00 = x1 / x0 * (ccoshlwl - csinhlwl / (tw * x1));
        let y01 = x0 / x1 * zcw * csinhlwl;
        let y10 = ((x1 / x0 - 1.0 / (tw * tw * x0 * x0)) * csinhlwl + 
                   tw * l_cone / ((tw * x0) * (tw * x0)) * ccoshlwl) / zcw;
        let y11 = x0 / x1 * (ccoshlwl + csinhlwl / (tw * x0));
        
        [[y00, y01], [y10, y11]]
    }
}

/// Radiation impedance at the bell (last segment) for angular frequency w (same as Python)
fn za(w: f64, segments: &[Segment]) -> Complex64 {
    if segments.is_empty() {
        return Complex64::new(0.0, 0.0);
    }
    
    let last_segment = &segments[segments.len() - 1];
    let l = last_segment.l;
    let d1 = last_segment.d1;
    let a01 = last_segment.a01;
    let r0 = last_segment.r0;
    
    // Viscothermal loss parameters (same as Python)
    let rvw = (AIR_DENSITY * w * a01 / (AIR_VISCOSITY * PI)).sqrt();
    let zcw = r0 * (1.0 + 0.369 / rvw) - Complex64::i() * r0 * 0.369 / rvw;
    
    // Radiation impedance (from Geipel, same as Python)
    0.5 * zcw * (w * w * d1 * d1 / (SPEED_OF_SOUND * SPEED_OF_SOUND) + 
                 Complex64::i() * 0.6 * l * w * d1 / SPEED_OF_SOUND)
}

/// Input impedance at mouthpiece (magnitude) for frequency f Hz (same as Python)
fn cadsd_ze(segments: &[Segment], frequency: f64) -> f64 {
    let w = 2.0 * PI * frequency;
    let a = za(w, segments);
    let b = ap(w, segments);
    
    // Input impedance magnitude (same as Python)
    let numerator = a * b[0][0] + b[0][1];
    let denominator = a * b[1][0] + b[1][1];
    
    (numerator / denominator).norm()
}

/// Get logarithmic simulation frequencies (same as Python)
pub fn get_log_simulation_frequencies() -> Vec<f64> {
    get_log_simulation_frequencies_with_params(20.0, 2000.0, 1.0)
}

/// Get logarithmic simulation frequencies with parameters (same as Python)
pub fn get_log_simulation_frequencies_with_params(fmin: f64, fmax: f64, grid_size: f64) -> Vec<f64> {
    let mut frequencies = Vec::new();
    let stepsize = grid_size / 1200.0;
    let start_freq = fmin;
    let mut end_freq = start_freq;
    let mut octave = 0;
    
    while end_freq < fmax {
        let notes: Vec<f64> = (0..1200).map(|n| {
            let note_step = n as f64 * stepsize;
            start_freq * 2.0f64.powf(note_step + octave as f64)
        }).collect();
        
        frequencies.extend(notes.into_iter().filter(|&f| f <= fmax));
        end_freq = *frequencies.last().unwrap_or(&fmax);
        octave += 1;
    }
    
    frequencies.into_iter().filter(|&f| f <= fmax).collect()
}

/// Compute ground spectrum (same interface as Python)
pub fn compute_ground_spektrum(geo: &Geo, simulation_method: &str) -> Result<Vec<(f64, f64)>, CadsdError> {
    let freqs = get_log_simulation_frequencies();
    let impedances = acoustical_simulation(geo, &freqs, simulation_method)?;
    
    // Find peaks (simplified version of Python implementation)
    let mut peaks = Vec::new();
    for i in 1..impedances.len() - 1 {
        if impedances[i] > impedances[i-1] && impedances[i] > impedances[i+1] {
            peaks.push((freqs[i], impedances[i]));
        }
    }
    
    Ok(peaks)
}

/// Get fundamental frequency (same interface as Python)
pub fn get_fundamental(geo: &Geo, simulation_method: &str, min_peak_f: f64) -> Result<(f64, f64), CadsdError> {
    let freqs = get_log_simulation_frequencies();
    let impedances = acoustical_simulation(geo, &freqs, simulation_method)?;
    
    // Find peaks
    let mut peaks = Vec::new();
    for i in 1..impedances.len() - 1 {
        if impedances[i] > impedances[i-1] && impedances[i] > impedances[i+1] {
            peaks.push((freqs[i], impedances[i], i));
        }
    }
    
    // Find first peak above minimum frequency
    for (freq, imp, _index) in peaks {
        if freq > min_peak_f {
            return Ok((freq, imp));
        }
    }
    
    Err(CadsdError::SimulationError("No fundamental frequency found".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::Geo;
    use approx::assert_abs_diff_eq;
    
    #[test]
    fn test_acoustical_simulation_basic() {
        let geo = Geo::make_cone(1000.0, 32.0, 50.0, 20);
        let frequencies = vec![50.0, 100.0, 200.0, 400.0];
        
        let result = acoustical_simulation(&geo, &frequencies, "tlm_python");
        assert!(result.is_ok());
        
        let impedances = result.unwrap();
        assert_eq!(impedances.len(), frequencies.len());
        for &impedance in &impedances {
            assert!(impedance.is_finite());
            assert!(impedance >= 0.0);
        }
    }
    
    #[test]
    fn test_frequency_grid_generation() {
        let frequencies = get_log_simulation_frequencies_with_params(50.0, 500.0, 1.0);
        assert!(!frequencies.is_empty());
        
        // Check that frequencies are in ascending order
        for i in 1..frequencies.len() {
            assert!(frequencies[i] > frequencies[i-1]);
        }
        
        // Check bounds
        assert!(frequencies[0] >= 50.0);
        assert!(frequencies[frequencies.len()-1] <= 500.0);
    }
    
    #[test]
    fn test_segment_creation() {
        let geo_data = vec![[0.0, 32.0], [1000.0, 50.0]];
        let segments = create_segments_from_geo(&geo_data);
        
        assert_eq!(segments.len(), 1);
        let segment = &segments[0];
        
        assert!(segment.l > 0.0);
        assert!(segment.d0 > 0.0);
        assert!(segment.d1 > 0.0);
        assert!(segment.a0 > 0.0);
        assert!(segment.r0 > 0.0);
    }
    
    #[test]
    fn test_cylindrical_vs_conical() {
        // Test cylindrical segment (d0 = d1)
        let geo_cyl = vec![[0.0, 32.0], [1000.0, 32.0]];
        let segments_cyl = create_segments_from_geo(&geo_cyl);
        let impedance_cyl = cadsd_ze(&segments_cyl, 100.0);
        
        // Test conical segment (d0 != d1)
        let geo_cone = vec![[0.0, 32.0], [1000.0, 50.0]];
        let segments_cone = create_segments_from_geo(&geo_cone);
        let impedance_cone = cadsd_ze(&segments_cone, 100.0);
        
        // Both should produce valid, finite impedances
        assert!(impedance_cyl.is_finite());
        assert!(impedance_cone.is_finite());
        assert!(impedance_cyl > 0.0);
        assert!(impedance_cone > 0.0);
    }
}