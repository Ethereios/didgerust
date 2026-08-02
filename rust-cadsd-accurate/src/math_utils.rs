// src/cadsd-accurate/math_utils.rs
pub mod math_utils {
    pub fn linear_interpolation(x0: f64, y0: f64, x1: f64, y1: f64, x: f64) -> f64 {
        if x1 == x0 { return y0; }
        y0 + (y1 - y0) * (x - x0) / (x1 - x0)
    }
}