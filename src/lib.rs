// Re-export Geo from the accurate crate
pub use cadsd_accurate::geo::Geo;

// Expose wrapper's modules (no re-exports that conflict with accurate crate)
pub mod evo;
pub mod geo;
pub mod loss;
pub mod sim;
pub mod tonehole;
pub mod visualization;
pub mod audio;

// New modules from adfb9d3

// New modules from adfb9d3
pub mod persistence;
pub mod integration;
pub mod export;
pub mod waveguide;

// Expose UI module (only when gui-bevy feature is enabled)
#[cfg(feature = "gui-bevy")]
pub mod app;

#[cfg(feature = "gui-bevy")]
pub mod gui;

// Neural integration module (behind nn-integration feature flag)
#[cfg(feature = "nn-integration")]
pub mod nn;

// Re-export simulation types from accurate crate
pub use cadsd_accurate::sim::{
    acoustical_simulation, 
    get_log_simulation_frequencies, 
    compute_ground_spektrum, 
    get_fundamental,
};

// Re-export local simulation functions
pub use crate::sim::{
    create_segments_from_geo,
    find_peaks,
    find_peaks_with_prominence,
    find_peaks_phase_based,
    DidgeridooSimulator,
    SimulationStrategy,
    Segment,
    Resonance,
};

// Re-export conversion utilities from accurate crate
pub use cadsd_accurate::conv::{
    note_to_freq,
    freq_to_note,
    note_name,
    freq_to_note_and_cent,
    freq_to_wavelength,
    cent_diff,
};

// Re-export analysis helpers from accurate crate
pub use cadsd_accurate::analysis::{
    get_notes,
    plot_bore,
    plot_impedance_spectrum,
};

// Re-export loss function types from accurate crate
pub use cadsd_accurate::loss::DidgeLabLoss;

// Re-export inverse design from accurate crate
pub use cadsd_accurate::inverse_design::{
    InverseDesigner,
    DesignResult,
};

// Re-export simulation and optimization integration types from accurate crate
pub use cadsd_accurate::integration::{
    AcousticSimulator,
    EvolutionaryOptimizer,
    AudioSynthesizer,
    GeometryExporter,
    DefaultSimulator,
    DefaultOptimizer,
};

// Re-export audio synthesis from accurate crate
pub use cadsd_accurate::audio::DefaultSynthesizer;

// Re-export export functionality from accurate crate
pub use cadsd_accurate::export::DefaultExporter;

// Re-export persistence types from accurate crate
pub use cadsd_accurate::persistence::{
    AppSettings,
    ProjectState,
};

// Error type
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

// Configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub sim_fmin: f64,
    pub sim_fmax: f64,
    pub sim_grid_size: f64,
    pub sim_grid: String,
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

pub fn init() -> Result<Config, CadsdError> {
    env_logger::init();
    Ok(Config::default())
}
