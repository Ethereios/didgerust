//! Project state and application settings persistence module

use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

/// Application-wide configuration settings.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub temperature: f32,
    pub wall_thickness: f32,
    pub show_3d: bool,
    pub mesh_rotation_enabled: bool,
    pub mesh_rotation_speed: f32,
    pub color_scheme: String,
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
        }
    }
}

impl AppSettings {
    /// Saves the settings to a JSON file.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(path, json)
            .map_err(|e| format!("IO error: {}", e))?;
        Ok(())
    }

    /// Loads the settings from a JSON file, returning Default if loading fails.
    pub fn load_from_file(path: impl AsRef<Path>) -> Self {
        if !path.as_ref().exists() {
            return Self::default();
        }
        
        fs::read_to_string(path)
            .and_then(|content| {
                serde_json::from_str::<AppSettings>(&content)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })
            .unwrap_or_else(|_| Self::default())
    }
}

/// Project state containing instrument geometry and optimization parameters.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectState {
    // Geometry parameters
    pub length: f32,
    pub top_diameter: f32,
    pub bottom_diameter: f32,
    pub segments: usize,
    pub style_type: String,
    pub bore_curve: f32,
    
    // Mouthpiece parameters
    pub enable_mouthpiece: bool,
    pub mouthpiece_type: String,
    pub mouthpiece_length: f32,
    
    // Finger hole parameters
    pub enable_holes: bool,
    pub hole_count: usize,
    pub hole_positions: Vec<f32>,
    pub hole_diameters: Vec<f32>,
    
    // Advanced parameters
    pub wall_thickness: f32,
    pub temperature: f32,
    pub target_frequency: f64,
    
    // Optimization configuration
    pub opt_bore_shape: String,
    pub opt_population_size: usize,
    pub opt_generations: usize,
    pub opt_min_length: f32,
    pub opt_max_length: f32,
    pub opt_min_bell: f32,
    pub opt_max_bell: f32,
    pub opt_toots_input: String,
}

impl ProjectState {
    /// Saves the project state to a file.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(path, json)
            .map_err(|e| format!("IO error: {}", e))?;
        Ok(())
    }

    /// Loads the project state from a file.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Could not read file: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Invalid project data: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_save_load() {
        let temp_dir = std::env::temp_dir();
        let filepath = temp_dir.join("cadsd_settings_test.json");
        
        let settings = AppSettings {
            temperature: 25.0,
            wall_thickness: 5.5,
            show_3d: false,
            mesh_rotation_enabled: true,
            mesh_rotation_speed: 45.0,
            color_scheme: "metal".to_string(),
        };
        
        assert!(settings.save_to_file(&filepath).is_ok());
        
        let loaded = AppSettings::load_from_file(&filepath);
        assert_eq!(loaded.temperature, 25.0);
        assert_eq!(loaded.wall_thickness, 5.5);
        assert_eq!(loaded.show_3d, false);
        assert_eq!(loaded.mesh_rotation_enabled, true);
        assert_eq!(loaded.mesh_rotation_speed, 45.0);
        assert_eq!(loaded.color_scheme, "metal");
        
        let _ = fs::remove_file(&filepath);
    }
}
