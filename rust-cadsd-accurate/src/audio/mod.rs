//! Didgeridoo audio synthesis and WAV file writing module

use crate::geo::Geo;
use crate::integration::AudioSynthesizer;
use std::io::Write;

/// Default synthesizer implementing additive synthesis based on impedance spectrum.
pub struct DefaultSynthesizer;

impl AudioSynthesizer for DefaultSynthesizer {
    fn synthesize(
        &self,
        _geo: &Geo,
        frequencies: &[f64],
        impedances: &[f64],
        duration_secs: f64,
        sample_rate: u32,
    ) -> Vec<f32> {
        if frequencies.is_empty() || impedances.is_empty() {
            return vec![0.0; (duration_secs * sample_rate as f64) as usize];
        }

        // 1. Identify fundamental frequency from the first impedance peak, or default to ~65Hz
        let f0 = detect_fundamental_from_spectrum(frequencies, impedances);
        
        let num_samples = (duration_secs * sample_rate as f64) as usize;
        let mut samples = vec![0.0; num_samples];

        // 2. Generate harmonics up to 2500 Hz
        let mut harmonics = Vec::new();
        let mut k = 1.0;
        while k * f0 < 2500.0 {
            let freq = k * f0;
            let z = interpolate_impedance(freq, frequencies, impedances);
            // Apply a natural roll-off for high frequencies so it sounds warm and buzzy
            let amp = z / k.powf(0.8);
            harmonics.push((freq, amp));
            k += 1.0;
        }

        // 3. Synthesize the waveform with small organic modulations (vibrato/tremolo)
        for i in 0..num_samples {
            let t = i as f64 / sample_rate as f64;
            
            // Subtle frequency fluctuation to mimic lip pressure variations
            let lip_vibrato = 1.0 + 0.012 * (2.0 * std::f64::consts::PI * 6.1 * t).sin();
            let mut sample_val = 0.0;

            for &(freq, amp) in &harmonics {
                // Additive synthesis sum
                sample_val += amp * (2.0 * std::f64::consts::PI * freq * lip_vibrato * t).sin();
            }

            samples[i] = sample_val as f32;
        }

        // 4. Normalize the samples to prevent clipping and keep volume consistent
        normalize_audio(&mut samples);
        samples
    }
}

/// Detects the fundamental frequency (Hz) by finding the first significant impedance peak.
fn detect_fundamental_from_spectrum(frequencies: &[f64], impedances: &[f64]) -> f64 {
    // Look for local maximum
    let mut peak_freq = 65.4; // Default D1
    let mut max_z = 0.0;
    
    // Simple peak detection in the lower region (20 to 150 Hz)
    for i in 1..(frequencies.len() - 1) {
        let f = frequencies[i];
        if f > 180.0 {
            break;
        }
        let z = impedances[i];
        if z > impedances[i - 1] && z > impedances[i + 1] && z > max_z {
            max_z = z;
            peak_freq = f;
        }
    }
    
    peak_freq
}

/// Interpolates the impedance value for any arbitrary frequency from the computed grid.
fn interpolate_impedance(target_freq: f64, freqs: &[f64], imps: &[f64]) -> f64 {
    if freqs.is_empty() {
        return 1.0;
    }
    if target_freq <= freqs[0] {
        return imps[0];
    }
    if target_freq >= freqs[freqs.len() - 1] {
        return imps[imps.len() - 1];
    }
    
    match freqs.binary_search_by(|f| f.partial_cmp(&target_freq).unwrap()) {
        Ok(idx) => imps[idx],
        Err(idx) => {
            let f0 = freqs[idx - 1];
            let f1 = freqs[idx];
            let z0 = imps[idx - 1];
            let z1 = imps[idx];
            let t = (target_freq - f0) / (f1 - f0);
            z0 + t * (z1 - z0)
        }
    }
}

/// Normalizes audio samples to a peak of 0.8 to avoid clipping.
fn normalize_audio(samples: &mut [f32]) {
    let mut max_val = 0.0_f32;
    for &s in samples.iter() {
        let abs_s = s.abs();
        if abs_s > max_val {
            max_val = abs_s;
        }
    }
    if max_val > 0.0 {
        let gain = 0.8 / max_val;
        for s in samples.iter_mut() {
            *s *= gain;
        }
    }
}

/// Writes a 16-bit mono PCM WAV file containing the synthesized samples.
pub fn write_wav_file(samples: &[f32], sample_rate: u32, writer: &mut impl Write) -> std::io::Result<()> {
    let num_samples = samples.len();
    let subchunk2_size = num_samples * 2; // 16-bit = 2 bytes per sample
    let chunk_size = 36 + subchunk2_size;
    
    // RIFF Header
    writer.write_all(b"RIFF")?;
    writer.write_all(&(chunk_size as u32).to_le_bytes())?;
    writer.write_all(b"WAVE")?;
    
    // fmt Subchunk
    writer.write_all(b"fmt ")?;
    writer.write_all(&16_u32.to_le_bytes())?; // Subchunk1Size (16 for PCM)
    writer.write_all(&1_u16.to_le_bytes())?;  // AudioFormat (1 for PCM)
    writer.write_all(&1_u16.to_le_bytes())?;  // NumChannels (1 for mono)
    writer.write_all(&sample_rate.to_le_bytes())?;
    writer.write_all(&(sample_rate * 2).to_le_bytes())?; // ByteRate (sample_rate * block_align)
    writer.write_all(&2_u16.to_le_bytes())?;  // BlockAlign (num_channels * bits_per_sample / 8)
    writer.write_all(&16_u16.to_le_bytes())?; // BitsPerSample (16 bits)
    
    // data Subchunk
    writer.write_all(b"data")?;
    writer.write_all(&(subchunk2_size as u32).to_le_bytes())?;
    
    // Write audio samples as i16
    for &sample in samples {
        let pcm_sample = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        writer.write_all(&pcm_sample.to_le_bytes())?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::Geo;

    #[test]
    fn test_synthesis_runs() {
        let geo = Geo::make_cone(1500.0, 32.0, 65.0, 20);
        let freqs = vec![50.0, 100.0, 150.0, 200.0];
        let imps = vec![1e5, 2e5, 5e4, 1e4];
        
        let synth = DefaultSynthesizer;
        let samples = synth.synthesize(&geo, &freqs, &imps, 0.5, 44100);
        
        assert_eq!(samples.len(), 22050);
        // Ensure normalization works (max element should be ~0.8)
        let max_sample = samples.iter().fold(0.0_f32, |m, &s| m.max(s.abs()));
        assert!((max_sample - 0.8).abs() < 1e-4);
    }
}
