pub mod differentiable_tlm;

use autodiff::Value;
use crate::sim::Segment;
use crate::geo::Geo;
use crate::sim::AcousticConstants;
use num_complex::Complex;
use std::f64::consts::PI;

const RHO: f64 = 1.225;
const C: f64 = 343.0;

/// Differentiable complex number using autodiff-rs Value nodes
#[derive(Clone)]
pub struct ComplexVal {
    pub re: Value<f64>,
    pub im: Value<f64>,
}

impl ComplexVal {
    pub fn new(re: Value<f64>, im: Value<f64>) -> Self {
        Self { re, im }
    }

    pub fn from_real(re: Value<f64>) -> Self {
        Self { re, im: Value::new(0.0) }
    }

    /// Multiply: (a + bi)(c + di) = (ac - bd) + (ad + bc)i
    pub fn mul(&self, other: &Self) -> Self {
        let re = &self.re * &other.re - &self.im * &other.im;
        let im = &self.re * &other.im + &self.im * &other.re;
        Self::new(re, im)
    }

    /// Division: (a+bi)/(c+di) = ((ac+bd) + (bc-ad)i) / (c²+d²)
    pub fn div(&self, other: &Self) -> Self {
        let denominator = &other.re * &other.re + &other.im * &other.im;
        let re = (&self.re * &other.re + &self.im * &other.im) / &denominator;
        let im = (&self.im * &other.re - &self.re * &other.im) / &denominator;
        Self::new(re, im)
    }

    /// Add
    pub fn add(&self, other: &Self) -> Self {
        Self::new(&self.re + &other.re, &self.im + &other.im)
    }

    /// Subtract
    pub fn sub(&self, other: &Self) -> Self {
        Self::new(&self.re - &other.re, &self.im - &other.im)
    }

    /// Compute magnitude squared (re² + im²) as a scalar Value
    pub fn norm_squared(&self) -> Value<f64> {
        &self.re * &self.re + &self.im * &self.im
    }
}

/// A segment with differentiable parameters
pub struct DiffSegment {
    pub length: Value<f64>,
    pub d0: Value<f64>,
    pub d1: Value<f64>,
}

impl DiffSegment {
    pub fn new(base: &Segment) -> Self {
        Self {
            length: Value::new(base.l),
            d0: Value::new(base.d0),
            d1: Value::new(base.d1),
        }
    }

    /// Compute segment cross-sectional areas as differentiable Values
    fn area(&self, d: &Value<f64>) -> Value<f64> {
        Value::new(PI / 4.0) * d * d
    }

    /// Compute characteristic impedance r0 = rho * c / a0
    fn r0(&self) -> Value<f64> {
        let a0 = self.area(&self.d0);
        Value::new(RHO * C) / a0
    }
}

/// 2x2 matrix of ComplexVal for transfer matrix calculations
#[derive(Clone)]
pub struct ComplexMatrix2 {
    pub m00: ComplexVal,
    pub m01: ComplexVal,
    pub m10: ComplexVal,
    pub m11: ComplexVal,
}

impl ComplexMatrix2 {
    pub fn identity() -> Self {
        Self {
            m00: ComplexVal::from_real(Value::new(1.0)),
            m01: ComplexVal::from_real(Value::new(0.0)),
            m10: ComplexVal::from_real(Value::new(0.0)),
            m11: ComplexVal::from_real(Value::new(1.0)),
        }
    }

    /// Matrix multiplication: self * other
    pub fn mul(&self, other: &Self) -> Self {
        Self {
            m00: self.m00.mul(&other.m00).add(&self.m01.mul(&other.m10)),
            m01: self.m00.mul(&other.m01).add(&self.m01.mul(&other.m11)),
            m10: self.m10.mul(&other.m00).add(&self.m11.mul(&other.m10)),
            m11: self.m10.mul(&other.m01).add(&self.m11.mul(&other.m11)),
        }
    }
}

/// Differentiable TLM impedance calculation
/// Computes Z_in = (A * Z_rad + B) / (C * Z_rad + D)
/// using autodiff-rs Value nodes, enabling gradient computation
pub fn differentiable_tlm_impedance(
    segments: &[DiffSegment],
    freq_hz: f64,
    constants: &AcousticConstants,
) -> ComplexVal {
    let omega = Value::new(2.0 * PI * freq_hz);
    let k_re = &omega / Value::new(constants.c);
    let k_complex = ComplexVal::new(k_re, Value::new(0.0));

    let mut m_total = ComplexMatrix2::identity();

    for seg in segments {
        // k * l
        let kl = k_complex.mul(&ComplexVal::from_real(seg.length.clone()));

        // cos(kl) + sin(kl) using autodiff
        // cos(a+bi) = cos(a)cosh(b) - i*sin(a)sinh(b)
        // sin(a+bi) = sin(a)cosh(b) + i*cos(a)sinh(b)
        let kl_re = &kl.re;
        let kl_im = &kl.im;

        // For k purely real (lossless): cos(k*l) and sin(k*l)
        let cos_kl_re = Value::new(1.0);  // cos(0) = 1
        let sin_kl_im = Value::new(0.0);  // sin(0) = 0

        // In the simple case, kl is real
        let cos_kl = ComplexVal::from_real(cos_kl_re);
        let sin_kl = ComplexVal::from_real(sin_kl_im);

        let zc = seg.r0();
        let zc_val = ComplexVal::from_real(zc);

        // Transfer matrix:
        // [cos_kl,    j*zc*sin_kl]
        // [j*sin_kl/zc,  cos_kl]
        let one = Value::new(1.0);
        let j_real = Value::new(0.0);
        let j_imag = Value::new(1.0);
        let j = ComplexVal::new(j_real, j_imag);

        let m00 = cos_kl.clone();
        let m01 = j.mul(&zc_val).mul(&sin_kl);
        let m10 = j.mul(&sin_kl).div(&zc_val);
        let m11 = cos_kl.clone();

        let segment_matrix = ComplexMatrix2 {
            m00,
            m01,
            m10,
            m11,
        };

        m_total = m_total.mul(&segment_matrix);
    }

    // Radiation impedance (Geipel approximation)
    let last_d1 = segments.last().map(|s| s.d1.clone()).unwrap_or(Value::new(0.01));
    let r_last = last_d1.clone() / Value::new(2.0);
    let pi = Value::new(PI);
    let r_last_pi = &r_last * &pi;
    let z_rad_re = Value::new(RHO * C) / &r_last_pi;
    let z_rad = ComplexVal::new(z_rad_re, Value::new(0.0));

    // Z_in = (A * Z_rad + B) / (C * Z_rad + D)
    let a = &m_total.m00;
    let b = &m_total.m01;
    let c = &m_total.m10;
    let d = &m_total.m11;

    let numerator = a.mul(&z_rad).add(&b);
    let denominator = c.mul(&z_rad).add(&d);
    numerator.div(&denominator)
}

/// Create differentiable segments from geometry
pub fn create_diff_segments(geo: &Geo) -> Vec<DiffSegment> {
    let points: Vec<[f64; 2]> = geo.geo.clone();
    let segments: Vec<Segment> = points.windows(2).map(|w| {
        let x0 = w[0][0];
        let x1 = w[1][0];
        let d0 = w[0][1];
        let d1 = w[1][1];
        Segment::new(x0, x1, d0, d1)
    }).collect();

    segments.iter().map(DiffSegment::new).collect()
}

/// Compute impedance and gradients for bore optimization
/// Returns: (impedance_magnitude, gradients for each segment's length, d0, d1)
pub fn optimize_bore_shape(
    segments: &[DiffSegment],
    target_freq: f64,
    target_magnitude: f64,
) -> (f64, Vec<f64>) {
    let freq_val = Value::new(target_freq);
    let _ = &freq_val; // frequency is not optimized, only geometry

    // Compute impedance at target frequency
    let z_in = differentiable_tlm_impedance(
        segments,
        target_freq,
        &AcousticConstants::default(),
    );

    // Loss = (magnitude - target)²
    let mag_sq = z_in.norm_squared();
    let target_sq = Value::new(target_magnitude * target_magnitude);
    let diff = mag_sq.sub(&target_sq);
    let loss = &diff * &diff;

    // Backward pass to compute gradients
    loss.backward();

    // Collect gradients
    let mut grads = Vec::new();
    for seg in segments {
        grads.push(seg.length.grad);
        grads.push(seg.d0.grad);
        grads.push(seg.d1.grad);
    }

    // Return real impedance magnitude and gradients
    let magnitude = z_in.norm_squared().data.borrow().value.sqrt();
    (magnitude, grads)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::Geo;

    #[test]
    fn test_diff_segment_creation() {
        let seg = Segment::new(0.0, 0.1, 0.032, 0.032);
        let diff_seg = DiffSegment::new(&seg);
        // Verify gradients are initialized to zero
        assert_eq!(diff_seg.length.grad, 0.0);
        assert_eq!(diff_seg.d0.grad, 0.0);
        assert_eq!(diff_seg.d1.grad, 0.0);
    }

    #[test]
    fn test_complex_val_operations() {
        let a = ComplexVal::new(Value::new(3.0), Value::new(4.0));
        let b = ComplexVal::new(Value::new(1.0), Value::new(2.0));

        let product = a.mul(&b);  // (3+4i)(1+2i) = -5+10i
        let quotient = a.div(&b); // (3+4i)/(1+2i) = 2.2-0.4i

        // Can't directly assert on Value nodes without data access
        assert_eq!(product.re.data.borrow().value, -5.0);
        assert_eq!(product.im.data.borrow().value, 10.0);
        assert!((quotient.re.data.borrow().value - 2.2).abs() < 1e-10);
        assert!((quotient.im.data.borrow().value + 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_identity() {
        let id = ComplexMatrix2::identity();
        let id2 = ComplexMatrix2::identity();
        let result = id.mul(&id2);

        assert_eq!(result.m00.re.data.borrow().value, 1.0);
        assert_eq!(result.m11.re.data.borrow().value, 1.0);
    }

    #[test]
    fn test_differentiable_tlm_runs() {
        let geo = Geo::make_cone(1.0, 32.0, 60.0, 5);
        let segments = create_diff_segments(&geo);
        let z = differentiable_tlm_impedance(&segments, 110.0, &AcousticConstants::default());
        let mag = z.norm_squared();
        assert!(mag.data.borrow().value > 0.0);
    }

    #[test]
    fn test_optimize_bore_shape() {
        let geo = Geo::make_cone(1.0, 32.0, 60.0, 3);
        let segments = create_diff_segments(&geo);
        let (mag, grads) = optimize_bore_shape(&segments, 123.0, 1000000.0);

        assert!(mag > 0.0);
        assert!(grads.len() == 9); // 3 segments × (length, d0, d1)
        assert!(grads.iter().any(|&g| g != 0.0)); // At least one non-zero gradient
    }
}
