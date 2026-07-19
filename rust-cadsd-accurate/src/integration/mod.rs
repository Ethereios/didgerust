//! Decoupled architecture traits and default service implementations
//!
//! This module defines traits for the core services:
//! - `AcousticSimulator`
//! - `EvolutionaryOptimizer`
//! - `AudioSynthesizer`
//! - `GeometryExporter`

use crate::geo::Geo;
use crate::evo::TargetSound;
use crate::inverse_design::{InverseDesigner, DesignResult};
use crate::CadsdError;
use std::sync::Arc;

/// Service for running acoustical simulation on a didgeridoo geometry.
pub trait AcousticSimulator: Send + Sync {
    /// Computes the impedance spectrum of a geometry across target frequencies.
    fn simulate(&self, geo: &Geo, frequencies: &[f64]) -> Result<Vec<f64>, CadsdError>;
    
    /// Finds the fundamental frequency (Hz) of a geometry.
    fn get_fundamental(&self, geo: &Geo) -> Result<f64, CadsdError>;
}

/// Service for running inverse design optimization from target sounds.
pub trait EvolutionaryOptimizer: Send + Sync {
    /// Optimizes didgeridoo geometry to match a target sound, invoking an optional progress callback.
    fn optimize(
        &self,
        target: TargetSound,
        pop_size: usize,
        generations: usize,
        progress_cb: Option<Arc<dyn Fn(usize, f64) + Send + Sync>>,
    ) -> Result<DesignResult, CadsdError>;
}

/// Service for synthesizing wind instrument/didgeridoo audio.
pub trait AudioSynthesizer: Send + Sync {
    /// Generates audio sample data (PCM float samples) based on the bore acoustics.
    fn synthesize(
        &self,
        geo: &Geo,
        frequencies: &[f64],
        impedances: &[f64],
        duration_secs: f64,
        sample_rate: u32,
    ) -> Vec<f32>;
}

/// Service for exporting bore geometries to 3D mesh formats.
pub trait GeometryExporter: Send + Sync {
    /// Writes the geometry as a standard OBJ mesh to the provided writer.
    fn export_obj(&self, geo: &Geo, writer: &mut impl std::io::Write) -> std::io::Result<()>;
    
    /// Writes the geometry as a GLTF mesh to the provided writer.
    fn export_gltf(&self, geo: &Geo, writer: &mut impl std::io::Write) -> std::io::Result<()>;
}

// === DEFAULT SERVICE IMPLEMENTATIONS ===

/// Standard acoustic simulator using the built-in transmission line model.
pub struct DefaultSimulator;

impl AcousticSimulator for DefaultSimulator {
    fn simulate(&self, geo: &Geo, frequencies: &[f64]) -> Result<Vec<f64>, CadsdError> {
        crate::sim::acoustical_simulation(geo, frequencies, "tlm_python")
    }

    fn get_fundamental(&self, geo: &Geo) -> Result<f64, CadsdError> {
        match crate::sim::get_fundamental(geo, "tlm_python", 20.0) {
            Ok((fund, _)) => Ok(fund),
            Err(e) => Err(e),
        }
    }
}

/// Standard evolutionary optimizer using the existing genetic algorithms.
pub struct DefaultOptimizer;

impl EvolutionaryOptimizer for DefaultOptimizer {
    fn optimize(
        &self,
        target: TargetSound,
        pop_size: usize,
        generations: usize,
        progress_cb: Option<Arc<dyn Fn(usize, f64) + Send + Sync>>,
    ) -> Result<DesignResult, CadsdError> {
        let designer = InverseDesigner::new()
            .with_population_size(pop_size)
            .with_generations(generations)
            .with_verbose(false);
            
        designer.design_with_progress(target, progress_cb)
            .map_err(|e| CadsdError::EvolutionError(e))
    }
}
