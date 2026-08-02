// This module re-exports note conversion and frequency conversion functions
// from the Python DidgeLab conv module.

pub mod note_conversion;
pub mod freq_conversion;

pub use note_conversion::{note_to_freq, freq_to_note, note_name, freq_to_note_and_cent, freq_to_wavelength, cent_diff};
pub use freq_conversion::{freq_to_hz, hz_to_freq, cent_to_freq, freq_to_cent};