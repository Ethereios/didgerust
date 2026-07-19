//! CADSD - Computer-Aided Didgeridoo Sound Design
//!
//! A comprehensive Rust implementation of CADSD for didgeridoo and wind instrument modeling,
//! featuring acoustical simulation, evolutionary optimization, and visualization tools.

pub mod sim;
pub mod geo;
pub mod evo;
pub mod loss;
pub mod visualization;

// Re-export commonly used items
pub use geo::{Geo, BoreGeometry};
pub use sim::{DidgeridooSimulator, SimulationParams, Segment};
pub use evo::{EvolutionaryOptimizer, Genome, LossFunction};
pub use loss::{CompositeTairuaLoss, IntegerHarmonicLoss, NearIntegerLoss, StretchedOddLoss, HarmonicSplittingLoss, PeakQuantityLoss, PeakAmplitudeLoss, ScaleTuningLoss, FrequencyTuningLoss, QFactorLoss, ModalDensityLoss, HighInharmonicLoss, TestLossFunction};

/// Main library error type
#[derive(thiserror::Error, Debug)]
pub enum CadsdError {
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),
    #[error("Simulation failed: {0}")]
    SimulationError(String),
    #[error("Evolution error: {0}")]
    EvolutionError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}