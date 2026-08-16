//! Real-time audio processing module
//!
//! This module provides low-latency audio output using the waveguide simulation
//! Engine. It enables real-time didgeridoo sound generation with amplitude control.
//!
//! NOTE: Atomic types temporarily disabled due to compilation issues.

use std::f64::consts::PI;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::waveguide::WaveguideEngine;

// Simple wrapper for atomic f64 using atomic u64
pub struct AtomicFloat64(AtomicU64);

impl AtomicFloat64 {
    pub fn new(val: f64) -> Self {
        Self(AtomicU64::new(val.to_bits()))
    }

    pub fn store(&self, val: f64, order: Ordering) {
        self.0.store(val.to_bits(), order);
    }

    pub fn load(&self, order: Ordering) -> f64 {
        f64::from_bits(self.0.load(order))
    }
}

impl Clone for AtomicFloat64 {
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.0.load(Ordering::Relaxed)))
    }
}

use std::sync::atomic::AtomicU64;

/// Configuration for real-time audio processing
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Sampling rate in Hz (default: 44100)
    pub sample_rate: u32,
    /// Block size in samples (affects latency vs. stability, default: 256)
    pub block_size: usize,
    /// Number of audio channels (default: 1 for mono)
    pub channels: usize,
    /// Base frequency reference for pitch (Hz, default: 440.0)
    pub reference_frequency: f64,
    /// Enable amplitude scaling
    pub amplitude_enabled: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            block_size: 256,
            channels: 1,
            reference_frequency: 440.0,
            amplitude_enabled: true,
        }
    }
}

/// Parameters for audio amplitude control
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmplitudeParams {
    /// Overall amplitude scaling (0.0 to 1.0, default: 0.5)
    pub gain: f64,
    /// Vibrato depth (0.0 to 1.0, default: 0.0)
    pub vibrato_depth: f64,
    /// Vibrato frequency in Hz (default: 5.0)
    pub vibrato_freq: f64,
    /// Current phase for vibrato modulation
    pub vibrato_phase: f64,
}

impl Default for AmplitudeParams {
    fn default() -> Self {
        Self {
            gain: 0.5,
            vibrato_depth: 0.0,
            vibrato_freq: 5.0,
            vibrato_phase: 0.0,
        }
    }
}

/// Low-latency audio processor using waveguide simulation
pub struct AudioProcessor {
    /// Waveguide engine for sound generation (shared across threads)
    engine: Arc<Mutex<WaveguideEngine>>,
    /// Current amplitude parameters
    amplitude: Arc<Mutex<AmplitudeParams>>,
    /// Running flag for audio thread
    running: Arc<AtomicBool>,
    /// Last computed frequency for tracking
    last_frequency: AtomicFloat64,
    /// Cached sample rate for calculations
    sample_rate: u32,
}

impl AudioProcessor {
    /// Create a new audio processor with specified geometry
    pub fn new(geo: &crate::Geo, config: AudioConfig) -> Result<Self, String> {
        let engine = WaveguideEngine::from_geo(geo);
        
        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
            amplitude: Arc::new(Mutex::new(AmplitudeParams::default())),
            running: Arc::new(AtomicBool::new(false)),
            last_frequency: AtomicFloat64::new(440.0),
            sample_rate: config.sample_rate,
        })
    }

    /// Set the bore frequency/resonance target
    pub fn set_frequency(&self, freq_hz: f64) {
        self.last_frequency.store(freq_hz, Ordering::Relaxed);
    }

    /// Set amplitude parameters
    pub fn set_amplitude(&self, gain: f64, vibrato_depth: f64, vibrato_freq: f64) {
        let mut amp = self.amplitude.lock().unwrap();
        amp.gain = gain.clamp(0.0, 1.0);
        amp.vibrato_depth = vibrato_depth.clamp(0.0, 1.0);
        amp.vibrato_freq = vibrato_freq.max(0.001);
        amp.vibrato_phase = 0.0;
    }

    /// Get current amplitude parameters
    pub fn get_amplitude(&self) -> AmplitudeParams {
        *self.amplitude.lock().unwrap()
    }

    /// Generate audio samples for the specified number of frames
    pub fn generate_samples(&self, frames: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(frames);
        let dt = 1.0 / (self.sample_rate as f64);
        let mut phase: f32 = 0.0;

        // Precompute static parameters
        let amp_guard = self.amplitude.lock().unwrap();
        let v_freq: f32 = (2.0 * PI * amp_guard.vibrato_freq) as f32;
        let v_phase_inc: f32 = v_freq * dt as f32;
        let freq = self.last_frequency.load(Ordering::Relaxed);
        let gain = amp_guard.gain;
        let vibrato_depth = amp_guard.vibrato_depth;
        drop(amp_guard);

        for _ in 0..frames {
            phase += v_phase_inc;
            let vibrato: f32 = phase.sin() * vibrato_depth as f32;
            let target_freq: f64 = freq * (1.0 + vibrato as f64);

            let z = self.engine.lock().unwrap().transfer_function(target_freq);
            let sample: f32 = (z.re * gain) as f32;
            samples.push(sample.clamp(-1.0, 1.0));
        }

        samples
    }

    /// Start real-time audio output
    pub fn start(&self) -> Result<(), String> {
        let running = Arc::clone(&self.running);
        running.store(true, Ordering::Relaxed);
        
        #[cfg(feature = "cpal-integration")]
        {
            use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
            let host = cpal::default_host();
            let device = host.default_output_device()
                .ok_or_else(|| "No output device found".to_string())?;
            
            let config = device.default_output_config()
                .map_err(|e| format!("Output config error: {}", e))?;
            
            let sample_rate = config.sample_rate() as u64;
            let channels = config.channels() as usize;
            
            let engine = Arc::clone(&self.engine);
            let amplitude = Arc::clone(&self.amplitude);
            let running_flag = Arc::clone(&self.running);
            let freq = Arc::new(self.last_frequency.clone());
            
            let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
            
            let stream = device.build_output_stream(
                config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    if !running_flag.load(Ordering::Relaxed) {
                        for sample in data.iter_mut() {
                            *sample = 0.0;
                        }
                        return;
                    }
                    
                    // Lock resources once for the frame
                    let mut phase: f32 = 0.0;
                    let dt: f32 = (1.0 / sample_rate as f64) as f32;
                    
                    {
                        let engine_guard = engine.lock().unwrap();
                        let amp_guard = amplitude.lock().unwrap();
                        
                        let v_freq: f32 = (2.0 * PI * amp_guard.vibrato_freq) as f32;
                        let v_phase_inc: f32 = v_freq * dt;
                        let target_freq = freq.load(Ordering::Relaxed);
                        let gain = amp_guard.gain;
                        let vibrato_depth = amp_guard.vibrato_depth;
                        
                        for sample in data.iter_mut() {
                            phase += v_phase_inc;
                            let vibrato: f32 = phase.sin() * vibrato_depth as f32;
                            let eff_freq = target_freq * (1.0 + vibrato as f64);
                            
                            let z = engine_guard.transfer_function(eff_freq);
                            let val = z.re * gain;
                            *sample = val.clamp(-1.0, 1.0) as f32;
                        }
                    }
                },
                err_fn,
                None,
            ).map_err(|e| format!("Stream build error: {}", e))?;
            
            stream.play().map_err(|e| format!("Stream play error: {}", e))?;
            println!("Audio processor started with cpal at {}Hz, {} channels", sample_rate, channels);
        }
        
        #[cfg(not(feature = "cpal-integration"))]
        {
            println!("Audio processor started (cpal integration disabled)");
        }
        
        Ok(())
    }

    /// Stop real-time audio output
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Check if audio is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Geo;

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 44100);
    }

    #[test]
    fn test_amplitude_default() {
        let amp = AmplitudeParams::default();
        assert_eq!(amp.gain, 0.5);
        assert_eq!(amp.vibrato_depth, 0.0);
    }

    #[test]
    fn test_audio_processor_creation() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        let config = AudioConfig::default();
        let processor = AudioProcessor::new(&geo, config);
        match processor {
            Ok(_) => {},
            Err(e) => panic!("Failed to create audio processor: {}", e),
        }
    }

    #[test]
    fn test_atomic_float64() {
        let a = AtomicFloat64::new(1.5);
        assert_eq!(a.load(Ordering::SeqCst), 1.5);
        a.store(2.5, Ordering::SeqCst);
        assert_eq!(a.load(Ordering::SeqCst), 2.5);
    }
}