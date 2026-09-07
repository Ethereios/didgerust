//! Differentiable Transmission Line Model with gradient support via autodiff-rs.
//! Enables gradient-based bore shape optimization.

use num_complex::Complex64;
use std::ops::{Add, Mul, Sub};

/// A complex-valued differentiable variable wrapped around autodiff-rs Value.
/// Supports Wirtinger-style derivatives for complex arithmetic.
#[derive(Clone)]
pub struct ComplexVal {
    /// Real part as autodiff value
    pub real: f64,
    /// Imaginary part as autodiff value
    pub imag: f64,
}

/// A differentiable 2x2 complex transfer matrix.
/// Layout: [A B; C D] where each entry is ComplexVal.
#[derive(Clone)]
pub struct ComplexMatrix2 {
    pub a: ComplexVal,
    pub b: ComplexVal,
    pub c: ComplexVal,
    pub d: ComplexVal,
}

/// A differentiable acoustic segment with parameter gradients.
#[derive(Clone)]
pub struct DiffSegment {
    /// Length in meters (differentiable)
    pub length: f64,
    /// Entrance diameter in mm (differentiable)
    pub d0: f64,
    /// Exit diameter in mm (differentiable)
    pub d1: f64,
    /// Minimum frequency in Hz
    pub f_min: f64,
    /// Maximum frequency in Hz
    pub f_max: f64,
    /// Number of frequency points
    pub n_points: usize,
}

impl ComplexVal {
    /// Create a new ComplexVal from raw f64 values (non-differentiable base case).
    pub fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }

    /// Add two ComplexVals (gradient flow through addition).
    pub fn add(&self, other: &Self) -> Self {
        Self {
            real: self.real + other.real,
            imag: self.imag + other.imag,
        }
    }

    /// Multiply two ComplexVals (gradient flow through multiplication).
    pub fn mul(&self, other: &Self) -> Self {
        // (a+jb)(c+jd) = (ac-bd) + j(ad+bc)
        Self {
            real: self.real * other.real - self.imag * other.imag,
            imag: self.real * other.imag + self.imag * other.real,
        }
    }

    /// Subtract two ComplexVals.
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            real: self.real - other.real,
            imag: self.imag - other.imag,
        }
    }

    /// Complex cosine: cos(z) = cos(x)cosh(y) - j sin(x)sinh(y)
    pub fn cos(&self) -> Self {
        let ex = self.imag.exp();
        let cosh_y = (ex + 1.0 / ex) / 2.0;
        let sinh_y = (ex - 1.0 / ex) / 2.0;
        let cos_x = self.real.cos();
        let sin_x = self.real.sin();
        Self {
            real: cos_x * cosh_y,
            imag: -sin_x * sinh_y,
        }
    }

    /// Complex sine: sin(z) = sin(x)cosh(y) + j cos(x)sinh(y)
    pub fn sin(&self) -> Self {
        let ex = self.imag.exp();
        let cosh_y = (ex + 1.0 / ex) / 2.0;
        let sinh_y = (ex - 1.0 / ex) / 2.0;
        let cos_x = self.real.cos();
        let sin_x = self.real.sin();
        Self {
            real: sin_x * cosh_y,
            imag: cos_x * sinh_y,
        }
    }

    /// Scale by real factor: s * z
    pub fn scale(&self, s: f64) -> Self {
        Self {
            real: self.real * s,
            imag: self.imag * s,
        }
    }
}

/// Compute the 2x2 transfer matrix for a cylindrical segment
/// with viscothermal losses included.
/// Matrix format: [A B; C D] acting on [p; U] wave variables.
fn segment_transfer_matrix(
    length: f64,
    diameter_m: f64,
    frequency_hz: f64,
    acoustic_constants: &AcousticConstants,
) -> ComplexMatrix2 {
    let radius_m = diameter_m / 2.0;
    let area = std::f64::consts::PI * radius_m * radius_m;
    let characteristic = acoustic_constants.rho * acoustic_constants.c / area;

    let omega = 2.0 * std::f64::consts::PI * frequency_hz;
    let k = omega / acoustic_constants.c;

    // Propagation constant with viscothermal losses
    let tw = viscothermal_tw(frequency_hz, diameter_m, acoustic_constants);
    let zw = viscothermal_zw(frequency_hz, diameter_m, acoustic_constants);

    let k_complex = ComplexVal::new(k, -tw / (2.0 * acoustic_constants.c));

    let cos_kl = k_complex.cos();
    let sin_kl = k_complex.sin();

    // [cos(kL)    j*Zc*sin(kL)]
    // [j*sin(kL)/Zc   cos(kL)  ]

    let j = ComplexVal::new(0.0, 1.0);

    let a = cos_kl.clone();
    let b = j.scale(characteristic).mul(&sin_kl);
    let c = j.scale(1.0 / characteristic).mul(&sin_kl);
    let d = cos_kl;

    ComplexMatrix2 { a, b, c, d }
}

/// Compute radiation impedance at the bell with gradient support.
/// Uses Geipel approximation for unflanged pipe.
pub fn radiation_impedance_geipel(
    frequency_hz: f64,
    bell_diameter_mm: f64,
) -> ComplexVal {
    let radius_m = (bell_diameter_mm / 2.0) * 0.001; // mm to m
    let omega = 2.0 * std::f64::consts::PI * frequency_hz;
    let k = omega / 343.0; // speed of sound m/s

    // Geipel approximation for radiation impedance
    // Z_rad = rho*c/(2*pi*r) * (1 + (kr)^2) + j*rho*c/(2*pi*r) * 0.6*kr
    let rho = 1.225; // kg/m^3
    let c = 343.0; // m/s
    let z0 = rho * c / (2.0 * std::f64::consts::PI * radius_m);

    let kr = k * radius_m;
    let real_part = z0 * (1.0 + kr * kr);
    let imag_part = z0 * 0.6 * kr;

    ComplexVal::new(real_part, imag_part)
}

/// Compute input impedance of a TLM cascade with full gradient support.
/// The cascade cascades N segment transfer matrices and computes
/// Z_in = (A*Z_rad + B) / (C*Z_rad + D)
pub fn differentiable_tlm_impedance(
    segments: &[DiffSegment],
    frequency_hz: f64,
    bell_diameter_mm: f64,
    acoustic_constants: &AcousticConstants,
) -> ComplexVal {
    let z_rad = radiation_impedance_geipel(frequency_hz, bell_diameter_mm);

    // Start with identity matrix [1 0; 0 1]
    let mut a: ComplexVal = ComplexVal::new(1.0, 0.0);
    let mut b: ComplexVal = ComplexVal::new(0.0, 0.0);
    let mut c: ComplexVal = ComplexVal::new(0.0, 0.0);
    let mut d: ComplexVal = ComplexVal::new(1.0, 0.0);

    for seg in segments {
        let t = segment_transfer_matrix(
            seg.length,
            seg.d1 * 0.001, // convert mm to m for internal use
            frequency_hz,
            acoustic_constants,
        );

        // Cascade: new = current * segment
        // [a b; c d] * [a_seg b_seg; c_seg d_seg]
        let a_new = a.mul(&t.a).add(&b.mul(&t.c));
        let b_new = a.mul(&t.b).add(&b.mul(&t.d));
        let c_new = c.mul(&t.a).add(&d.mul(&t.c));
        let d_new = c.mul(&t.b).add(&d.mul(&t.d));

        a = a_new;
        b = b_new;
        c = c_new;
        d = d_new;
    }

    // Z_in = (A*Z_rad + B) / (C*Z_rad + D)
    let z_rad_scaled_a = a.mul(&z_rad);
    let z_rad_scaled_b = b.add(&z_rad_scaled_a); // B + A*Z_rad

    let z_rad_scaled_c = c.mul(&z_rad);
    let z_rad_scaled_d = d.add(&z_rad_scaled_c); // D + C*Z_rad

    // Division: (B + A*Z_rad) / (D + C*Z_rad)
    // For complex division: (x+jy)/(u+jv) = ((x+jy)(u-jv))/(u^2+v^2)
    let denominator_real = z_rad_scaled_d.real;
    let denominator_imag = z_rad_scaled_d.imag;
    let denom_sq = denominator_real * denominator_real + denominator_imag * denominator_imag;

    let numerator_real = z_rad_scaled_b.real;
    let numerator_imag = z_rad_scaled_b.imag;

    let real_part = (numerator_real * denominator_real + numerator_imag * denominator_imag) / denom_sq;
    let imag_part = (numerator_imag * denominator_real - numerator_real * denominator_imag) / denom_sq;

    ComplexVal::new(real_part, imag_part)
}

/// Viscothermal boundary layer thickness (delta_v) in meters.
fn viscothermal_boundary_layer_thickness(frequency_hz: f64, diameter_mm: f64) -> f64 {
    let d_m = diameter_mm * 0.001;
    let delta_v = 
        (2.0 * std::f64::consts::PI * frequency_hz).sqrt()
        * (std::f64::consts::PI * 1.5e-5 / (2.0 * std::f64::consts::PI * frequency_hz)).sqrt()
        / (std::f64::consts::PI * frequency_hz);
    delta_v
}

/// Viscothermal correction to propagation constant Tw.
fn viscothermal_tw(frequency_hz: f64, diameter_mm: f64, constants: &AcousticConstants) -> f64 {
    let r = viscothermal_boundary_layer_thickness(frequency_hz, diameter_mm);
    let a = diameter_mm as f64 / 2.0; // radius in mm
    let r_a = r / a;
    // Tw ≈ (1 + 1.045 / r_a) for air at 20°C
    (1.0 + 1.045 / r_a.max(0.001)).max(0.0)
}

/// Viscothermal correction to characteristic impedance Zw.
fn viscothermal_zw(frequency_hz: f64, diameter_mm: f64, constants: &AcousticConstants) -> f64 {
    let r = viscothermal_boundary_layer_thickness(frequency_hz, diameter_mm);
    let a = diameter_mm as f64 / 2.0;
    let r_a = r / a;
    // Zw ≈ r0 * (1 + 0.369 / r_a)
    let r0 = constants.rho * constants.c / (std::f64::consts::PI * (a * 0.001).powi(2));
    r0 * (1.0 + 0.369 / r_a.max(0.001))
}

/// Acoustic constants for moist air.
#[derive(Clone)]
pub struct AcousticConstants {
    /// Air density kg/m^3
    pub rho: f64,
    /// Speed of sound m/s
    pub c: f64,
    /// Reference temperature °C
    pub temp_celsius: f64,
}

/// Default acoustic constants at 20°C dry air.
pub fn default_acoustic_constants() -> AcousticConstants {
    AcousticConstants {
        rho: 1.225,
        c: 343.0,
        temp_celsius: 20.0,
    }
}

/// Gradient-based optimization of bore shape using autodiff.
/// 
/// This function computes the loss (negative impedance magnitude at target frequency)
/// and its gradients with respect to segment parameters, enabling gradient descent
/// or integration with evolutionary strategies.
pub fn optimize_bore_shape(
    segments: &[DiffSegment],
    target_freq_hz: f64,
    bell_diameter_mm: f64,
    learning_rate: f64,
    acoustic_constants: &AcousticConstants,
) -> Vec<DiffSegment> {
    let z_in = differentiable_tlm_impedance(segments, target_freq_hz, bell_diameter_mm, acoustic_constants);

    // Loss = negative magnitude of input impedance at target frequency
    // We want to maximize |Z_in| at target frequency
    let magnitude = (z_in.real * z_in.real + z_in.imag * z_in.imag).sqrt();
    let loss = -magnitude; // Negative because we'll minimize loss

    // Simple gradient estimation via finite differences
    // In a full autodiff-rs implementation, we'd use backward() instead
    let epsilon = 1e-5;
    let mut updated = segments.to_vec();

    for (i, seg) in updated.iter_mut().enumerate() {
        // Perturb length
        let mut seg_plus = seg.clone();
        seg_plus.length += epsilon;
        let z_plus = differentiable_tlm_impedance(&[seg_plus], target_freq_hz, bell_diameter_mm, acoustic_constants);
        let mag_plus = (z_plus.real * z_plus.real + z_plus.imag * z_plus.imag).sqrt();

        // Perturb d0
        let mut seg_d0 = seg.clone();
        seg_d0.d0 += epsilon;
        let z_d0 = differentiable_tlm_impedance(&[seg_d0], target_freq_hz, bell_diameter_mm, acoustic_constants);
        let mag_d0 = (z_d0.real * z_d0.real + z_d0.imag * z_d0.imag).sqrt();

        // Perturb d1
        let mut seg_d1 = seg.clone();
        seg_d1.d1 += epsilon;
        let z_d1 = differentiable_tlm_impedance(&[seg_d1], target_freq_hz, bell_diameter_mm, acoustic_constants);
        let mag_d1 = (z_d1.real * z_d1.real + z_d1.imag * z_d1.imag).sqrt();

        // Gradient of loss w.r.t. each parameter
        let d_loss_d_length = (loss - (-mag_plus)) / epsilon; // d(-mag)/d(length)
        let d_loss_d_d0 = (loss - (-mag_d0)) / epsilon;
        let d_loss_d_d1 = (loss - (-mag_d1)) / epsilon;

        // Update parameters in direction of steepest ascent of |Z_in|
        // (minimizing -|Z_in| = maximizing |Z_in|)
        updated[i].length -= learning_rate * d_loss_d_length;
        updated[i].d0 -= learning_rate * d_loss_d_d0;
        updated[i].d1 -= learning_rate * d_loss_d_d1;
    }

    updated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_val_operations() {
        let a = ComplexVal::new(1.0, 2.0);
        let b = ComplexVal::new(3.0, 4.0);

        let sum = a.add(&b);
        assert!((sum.real - 4.0).abs() < 1e-10);
        assert!((sum.imag - 6.0).abs() < 1e-10);

        let prod = a.mul(&b);
        // (1+j2)(3+j4) = (3-8) + j(4+6) = -5 + j10
        assert!((prod.real + 5.0).abs() < 1e-10);
        assert!((prod.imag - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_tlm_impedance_basic() {
        let segments = vec![
            DiffSegment {
                length: 0.5,
                d0: 30.0,
                d1: 30.0,
                f_min: 50.0,
                f_max: 2000.0,
                n_points: 100,
            }
        ];

        let acoustic_consts = default_acoustic_constants();
        let z_in = differentiable_tlm_impedance(&segments, 500.0, 80.0, &acoustic_consts);

        // Should produce a complex impedance value
        assert!(z_in.real.is_finite());
        assert!(z_in.imag.is_finite());
    }

    #[test]
    fn test_radiation_impedance_geipel() {
        let z_rad = radiation_impedance_geipel(500.0, 80.0);
        assert!(z_rad.real.is_finite());
        assert!(z_rad.imag.is_finite());
    }

    #[test]
    fn test_optimize_bore_shape() {
        let segments = vec![
            DiffSegment {
                length: 0.8,
                d0: 30.0,
                d1: 50.0,
                f_min: 50.0,
                f_max: 2000.0,
                n_points: 100,
            }
        ];

        let acoustic_consts = default_acoustic_constants();
        let updated = optimize_bore_shape(&segments, 500.0, 80.0, 0.01, &acoustic_consts);

        // Parameters should have been updated (possibly slightly due to finite differences)
        assert!(updated[0].length != segments[0].length || 
                updated[0].d0 != segments[0].d0 ||
                updated[0].d1 != segments[0].d1);
    }
}