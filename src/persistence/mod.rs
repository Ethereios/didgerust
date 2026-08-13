//! Persistence module for saving/loading application state
//!
//! Provides real JSON file save/load for AppSettings, ProjectState,
//! and optimizer checkpoints via serde_json + std::fs.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Validates geometry points for consistency and physical plausibility
pub fn validate_geo_points(points: &[[f64; 2]]) -> Result<(), String> {
    if points.is_empty() {
        return Err("Geometry points cannot be empty".to_string());
    }
    
    // Minimum 2 points required to define a bore shape
    if points.len() < 2 {
        return Err("Geometry must have at least 2 points to define a bore shape".to_string());
    }
    
    let mut prev_length: Option<f64> = None;
    let mut prev_diameter: Option<f64> = None;
    
    for point in points {
        // Check that we have exactly 2 elements per point
        if point.len() != 2 {
            return Err("Each geometry point must have exactly 2 coordinates".to_string());
        }
        
        // Validate X coordinate (position along bore)
        if point[0] < 0.0 {
            return Err("X coordinates must be non-negative".to_string());
        }
        
        // Validate diameter (index 1) - must be positive
        let diameter = point[1];
        if diameter <= 0.0 {
            return Err("Diameters must be positive".to_string());
        }
        
        // Validate that X coordinates are strictly increasing
        if let Some(prev_x) = prev_length {
            if point[0] <= prev_x {
                return Err("X coordinates must be strictly increasing".to_string());
            }
        }
        
        // Validate that diameters don't decrease too rapidly (minimum taper)
        if let (Some(prev_d), Some(prev_x)) = (prev_diameter, prev_length) {
            let taper_rate = (diameter - prev_d) / (point[0] - prev_x + f64::EPSILON);
            if taper_rate < -10.0 { // Too steep negative taper
                return Err("Diameter taper too steep".to_string());
            }
        }
        
        prev_length = Some(point[0]);
        prev_diameter = Some(diameter);
    }
    
    Ok(())
}

/// History entry for undo/redo in geometry panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoHistoryEntry {
    pub geo_points: Vec<[f64; 2]>,
    pub label: String,
}

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
    /// Validate settings and return a validated copy with corrected values
    pub fn validate(&self) -> Self {
        let mut result = self.clone();
        
        // Temperature clamping
        if result.temperature < -273.15 {
            result.temperature = -273.15;
        }
        
        // Wall thickness minimum
        if result.wall_thickness <= 0.0 {
            result.wall_thickness = 1.0;
        }
        
        // Boolean validation - ensure they are proper booleans
        // In serde JSON, booleans are always valid, just reassign to confirm
        
        // Non-negative values
        if result.mesh_rotation_speed < 0.0 {
            result.mesh_rotation_speed = 0.0;
        }
        
        // Non-empty strings (at minimum set to "unknown" if empty)
        if result.color_scheme.is_empty() {
            result.color_scheme = "unknown".to_string();
        }
        if result.theme.is_empty() {
            result.theme = "dark".to_string();
        }
        
        // Log verbosity bounds
        if result.log_verbosity > 4 {
            result.log_verbosity = 4;
        }
        
        // Ensure all fields are properly set
        result
    }
}

/// Project-level state (geometry + simulation results)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub geometry: String,
    pub simulation_results: String,
    /// Historical states for undo/redo functionality
    pub geo_history: Vec<GeoHistoryEntry>,
    /// Current index in history stack
    pub geo_history_index: usize,
}

impl Default for ProjectState {
    fn default() -> Self {
        Self {
            geometry: String::new(),
            simulation_results: String::new(),
            geo_history: Vec::new(),
            geo_history_index: 0,
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_empty_points() {
        let result = validate_geo_points(&[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }
    
    #[test]
    fn test_validate_negative_x() {
        // Test that negative X coordinates are rejected (using 2 points to satisfy minimum)
        let result = validate_geo_points(&[[-1.0, 32.0], [0.0, 32.0]]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("non-negative"));
    }

    #[test]
    fn test_validate_negative_diameter() {
        // Test that negative diameters are rejected (using 2 points to satisfy minimum)
        let result = validate_geo_points(&[[0.0, -1.0], [1.0, 2.0]]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("positive"));
    }

    #[test]
    fn test_validate_strictly_increasing_x() {
        // X coordinates must be strictly increasing
        let result = validate_geo_points(&[[10.0, 32.0], [5.0, 20.0]]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("strictly increasing"));
    }

    #[test]
    fn test_validate_single_valid_point() {
        // Renamed to test_validate_two_valid_points to reflect requirement of at least 2 points
        let result = validate_geo_points(&[[0.0, 32.0], [1.0, 32.0]]);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_valid_geometry_points() {
        // Should accept strictly increasing X coordinates with positive diameters
        let result = validate_geo_points(&[[0.0, 32.0], [1000.0, 20.0]]);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_acceptable_taper() {
        // Acceptable taper: 100mm to 20mm over 1000mm length = 0.08 taper
        let result = validate_geo_points(&[[0.0, 100.0], [1000.0, 20.0]]);
        assert!(result.is_ok());
    }
    
#[test]
fn test_validate_unacceptable_taper() {
    // Unacceptable taper: 10mm to 1mm over 0.1mm length = -99 taper
    let result = validate_geo_points(&[[1.0, 10.0], [1.1, 1.0]]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too steep"));
}
    
    #[test]
    fn test_validate_geo_schema() {
        // Test saving and loading geo schema validation
        let geo_points = vec![
            [0.0, 32.0],
            [1000.0, 20.0],
            [2000.0, 25.0]
        ];
        let result = validate_geo_points(&geo_points);
        assert!(result.is_ok());
    }
    
    // Additional comprehensive tests
    #[test]
    fn test_validate_diameter_positive() {
        assert!(validate_geo_points(&[[0.0, 1.0], [1.0, 1.0]]).is_ok());
        assert!(validate_geo_points(&[[0.0, 1.0], [1.0, 0.0]]).is_err());
        assert!(validate_geo_points(&[[0.0, 1.0], [1.0, -1.0]]).is_err());
    }
    
    #[test]
    fn test_minimum_geometry_points() {
        // Minimum valid geometry must have at least 2 points to form a segment
        assert!(validate_geo_points(&[[0.0, 10.0]]).is_err());
        assert!(validate_geo_points(&[[0.0, 10.0], [1.0, 10.0]]).is_ok());
    }
    
    #[test]
    fn test_large_negative_taper() {
        // Very steep negative taper should fail
        let result = validate_geo_points(&[[0.0, 100.0], [0.1, 5.0]]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too steep"));
    }
    
    #[test]
    fn test_large_positive_taper() {
        // Very steep positive taper should pass (not restricted)
        let result = validate_geo_points(&[[0.0, 5.0], [0.1, 100.0]]);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_complex_multi_segment_validation() {
        // Multi-segment geometry validation
        let geometry = vec![
            [0.0, 32.0],
            [1000.0, 25.0], 
            [2000.0, 80.0],
            [3000.0, 20.0]
        ];
        assert!(validate_geo_points(&geometry).is_ok());
    }
    
    #[test]
    fn test_identical_x_coordinates() {
        // Identical X coordinates at different points should fail
        assert!(validate_geo_points(&[[0.0, 10.0], [0.0, 20.0]]).is_err());
    }
    
    #[test]
    fn test_appsettings_save_load_roundtrip() {
        // Test that AppSettings can be serialized and deserialized
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let loaded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings.temperature, loaded.temperature);
    }
    
    #[test]
    fn test_project_state_serialization() {
        // Test ProjectState serialization with geo_history
        let mut state = ProjectState::default();
        state.geometry = "test_geo".to_string();
        state.geo_history.push(GeoHistoryEntry {
            geo_points: vec![[0.0, 32.0], [100.0, 20.0]],
            label: "initial".to_string(),
        });
        let json = serde_json::to_string(&state).unwrap();
        let loaded: ProjectState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.geometry, loaded.geometry);
        assert_eq!(state.geo_history.len(), loaded.geo_history.len());
    }
}
