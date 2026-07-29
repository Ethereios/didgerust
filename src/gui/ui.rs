//! UI module for CADSD GUI
// Re-export from app module when gui-bevy feature is enabled

#[cfg(feature = "gui-bevy")]
use bevy::prelude::Resource;

#[cfg(feature = "gui-bevy")]
pub use crate::app::CadsdState;

#[cfg(feature = "gui-bevy")]
pub fn init_ui(state: &mut CadsdState) {
    // Initialize UI components here
    state.active_tab = "forward".to_string();
}