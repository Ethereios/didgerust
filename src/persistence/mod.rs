//! Persistence module for saving/loading application state

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn load_from_file(_path: &str) -> Self {
        Self::default()
    }
}

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