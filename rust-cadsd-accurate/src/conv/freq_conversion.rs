/// Convert frequency to Hz (identity function, for API consistency)
pub fn freq_to_hz(freq: f64) -> f64 {
    freq
}

/// Convert Hz to frequency (identity function, for API consistency)
pub fn hz_to_freq(hz: f64) -> f64 {
    hz
}

/// Convert cents to frequency ratio
pub fn cent_to_freq(cent: f64) -> f64 {
    2.0_f64.powf(cent / 1200.0)
}

/// Convert frequency ratio to cents
pub fn freq_to_cent(freq: f64) -> f64 {
    1200.0 * freq.log2()
}