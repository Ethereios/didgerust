pub mod geo;
pub mod sim;
pub mod evo;
pub mod loss;
pub mod conv;
pub mod analysis;
pub mod inverse_design;
pub mod integration;
pub mod audio;
pub mod export;
pub mod persistence;

#[cfg(feature = "gui-bevy")]
pub mod ui;

pub use geo::Geo;

pub use conv::{note_to_freq, freq_to_note, note_name, freq_to_note_and_cent, freq_to_wavelength, cent_diff};
pub use sim::{acoustical_simulation, get_log_simulation_frequencies, compute_ground_spektrum, get_fundamental};
pub use analysis::{get_notes, plot_bore, plot_impedance_spectrum};
pub use loss::DidgeLabLoss;
pub use inverse_design::{InverseDesigner, DesignResult};
pub use integration::{AcousticSimulator, EvolutionaryOptimizer, AudioSynthesizer, GeometryExporter, DefaultSimulator, DefaultOptimizer};
pub use audio::DefaultSynthesizer;
pub use export::DefaultExporter;
pub use persistence::{AppSettings, ProjectState};

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