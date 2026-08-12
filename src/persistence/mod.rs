//! Persistence module for saving/loading application state
//!
//! Provides real JSON file save/load for AppSettings, ProjectState,
//! and optimizer checkpoints via serde_json + std::fs.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Application-wide settings (persisted to disk)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub temperature: f32,
    pub wall_thickness: f32,
    pub show_3d: bool,
    pub mesh_rotation_enabled: bool,
    pub mesh_rotation_speed: f32,
    pub color_scheme: String,
    /// Theme selection: "dark" or "light"
    pub theme: String,
    /// Logging verbosity level (0=error, 1=warn, 2=info, 3=debug, 4=trace)
    pub log_verbosity: u8,
    /// Default simulation strategy name
    pub default_strategy: String,
    /// Default mutation strategy name
    pub default_mutation: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            temperature: 20.0,
            wall_thickness: 4.0,
            show_3d: true,
            mesh_rotation_enabled: false,
            mesh_rotation_speed: 30.0,
            color_scheme: "wood".to_string(),
            theme: "dark".to_string(),
            log_verbosity: 2,
            default_strategy: "Tlm".to_string(),
            default_mutation: "Gaussian".to_string(),
        }
    }
}

impl AppSettings {
    /// Load settings from a JSON file. Returns default if file not found or invalid.
    pub fn load_from_file(path: &str) -> Self {
        if !Path::new(path).exists() {
            log::warn!("Settings file not found: {}, using defaults", path);
            return Self::default();
        }
        match fs::read_to_string(path) {
            Ok(contents) => {
                match serde_json::from_str::<AppSettings>(&contents) {
                    Ok(settings) => {
                        log::info!("Settings loaded from {}", path);
                        settings
                    }
                    Err(e) => {
                        log::error!("Failed to parse settings file {}: {}, using defaults", path, e);
                        Self::default()
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read settings file {}: {}, using defaults", path, e);
                Self::default()
            }
        }
    }

    /// Save settings to a JSON file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_str = serde_json::to_string_pretty(self)?;
        fs::write(path, json_str)?;
        log::info!("Settings saved to {}", path);
        Ok(())
    }
}

/// Project-level state (geometry + simulation results)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub geometry: String,
    pub simulation_results: String,
}

impl Default for ProjectState {
    fn default() -> Self {
        Self {
            geometry: String::new(),
            simulation_results: String::new(),
        }
    }
}

impl ProjectState {
    /// Load project state from a JSON file
    pub fn load_from_file(path: &str) -> Option<Self> {
        if !Path::new(path).exists() {
            log::warn!("Project file not found: {}", path);
            return None;
        }
        match fs::read_to_string(path) {
            Ok(contents) => {
                match serde_json::from_str::<ProjectState>(&contents) {
                    Ok(state) => {
                        log::info!("Project state loaded from {}", path);
                        Some(state)
                    }
                    Err(e) => {
                        log::error!("Failed to parse project file {}: {}", path, e);
                        None
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read project file {}: {}", path, e);
                None
            }
        }
    }

    /// Save project state to a JSON file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_str = serde_json::to_string_pretty(self)?;
        fs::write(path, json_str)?;
        log::info!("Project state saved to {}", path);
        Ok(())
    }
}

/// Checkpoint data for the evolutionary optimizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerCheckpoint {
    pub timestamp: String,
    pub population_size: usize,
    pub num_generations: usize,
    pub current_generation: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elite_size: usize,
    pub best_loss: Option<f64>,
    pub generation_progress: f64,
    pub mutation_strategy: String,
    pub simulation_strategy: String,
    pub geometry: OptimizerGeoState,
    pub loss_component_weights: Vec<(String, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerGeoState {
    pub length: f64,
    pub top_diameter: f64,
    pub bottom_diameter: f64,
    pub segments: usize,
}

impl OptimizerCheckpoint {
    /// Load checkpoint from a JSON file
    pub fn load_from_file(path: &str) -> Option<Self> {
        if !Path::new(path).exists() {
            log::warn!("Checkpoint file not found: {}", path);
            return None;
        }
        match fs::read_to_string(path) {
            Ok(contents) => {
                match serde_json::from_str::<OptimizerCheckpoint>(&contents) {
                    Ok(cp) => {
                        log::info!("Checkpoint loaded from {}", path);
                        Some(cp)
                    }
                    Err(e) => {
                        log::error!("Failed to parse checkpoint file {}: {}", path, e);
                        None
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read checkpoint file {}: {}", path, e);
                None
            }
        }
    }

    /// Save checkpoint to a JSON file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_str = serde_json::to_string_pretty(self)?;
        fs::write(path, json_str)?;
        log::info!("Checkpoint saved to {}", path);
        Ok(())
    }
}
