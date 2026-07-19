//! CADSD - Computer-Aided Didgeridoo Sound Design
//!
//! Accurate Rust implementation of the CADSD methodology for didgeridoo and 
//! wind instrument modeling, based on Frank Geipel's work and the DidgeLab toolkit.
//!
//! # Core Functionality
//!
//! This library provides two main capabilities:
//!
//! 1. **Forward Design**: Given a geometry, predict its acoustic properties
//! 2. **Inverse Design**: Given target acoustic properties, find a matching geometry
//!
//! # Key Features
//!
//! - Acoustical simulation using transmission line modeling
//! - Computational evolution for shape optimization
//! - Parametric shape generation (Kigali, Mbeya styles)
//! - Modular loss functions for multi-objective optimization
//! - Note/frequency conversion utilities
//! - Spectrum analysis tools
//!
//! # Usage Example
//!
//! ```rust
//! use cadsd_accurate::geo::Geo;
//! use cadsd_accurate::acoustical_simulation::acoustical_simulation;
//! use cadsd_accurate::conv::{note_to_freq, get_log_simulation_frequencies};
//!
//! // Create a conical didgeridoo
//! let geo = Geo::make_cone(1500.0, 32.0, 65.0, 30);
//!
//! // Compute impedance spectrum
//! let frequencies = get_log_simulation_frequencies();
//! let impedances = acoustical_simulation(&geo, &frequencies, "tlm_cython")?;
//!
//! // Find resonance peaks
//! let peaks = get_notes(&frequencies, &impedances);
//! ```

pub mod geo;
pub mod sim;
pub mod evo;
pub mod loss;
pub mod conv;
pub mod analysis;
#[cfg(feature = "gui-bevy")]
pub mod app;
#[cfg(feature = "gui-bevy")]
pub mod minimal_gui;
pub mod inverse_design;
#[cfg(feature = "gui-bevy")]
pub mod didgerust_app;
pub mod integration;
pub mod audio;
pub mod export;
pub mod persistence;

// Re-export main public API
pub use geo::Geo;
pub use sim::{acoustical_simulation, get_log_simulation_frequencies, compute_ground_spektrum, get_fundamental};
pub use conv::{note_to_freq, freq_to_note, note_name, freq_to_note_and_cent, freq_to_wavelength, cent_diff};
pub use analysis::{get_notes, vis_didge, plot_bore, plot_impedance_spectrum};
pub use evo::{Nuevolution, GeoGenome, LossFunctionType as LossFunction, MutationOperator, CrossoverOperator, TargetSound, BoreShapePreference};
pub use loss::DidgeLabLoss;
pub use inverse_design::{InverseDesigner, DesignResult};
pub use integration::{AcousticSimulator, EvolutionaryOptimizer, AudioSynthesizer, GeometryExporter, DefaultSimulator, DefaultOptimizer};
pub use audio::DefaultSynthesizer;
pub use export::DefaultExporter;
pub use persistence::{AppSettings, ProjectState};

/// Main library error type
#[derive(thiserror::Error, Debug)]
pub enum CadsdError {
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),
    #[error("Simulation failed: {0}")]
    SimulationError(String),
    #[error("Evolution error: {0}")]
    EvolutionError(String),
    #[error("Invalid frequency range: {0}")]
    InvalidFrequencyRange(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Configuration for the CADSD system
#[derive(Debug, Clone)]
pub struct Config {
    /// Minimum frequency for simulation (Hz)
    pub sim_fmin: f64,
    /// Maximum frequency for simulation (Hz)
    pub sim_fmax: f64,
    /// Frequency grid size
    pub sim_grid_size: f64,
    /// Grid type ("even" or "log")
    pub sim_grid: String,
    /// Simulation backend ("tlm_python" or "tlm_cython")
    pub sim_backend: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sim_fmin: 20.0,
            sim_fmax: 2000.0,
            sim_grid_size: 1.0,
            sim_grid: "log".to_string(),
            sim_backend: "tlm_cython".to_string(),
        }
    }
}

/// Initialize the CADSD system
pub fn init() -> Result<Config, CadsdError> {
    env_logger::init();
    Ok(Config::default())
}