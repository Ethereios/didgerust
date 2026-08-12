//! GUI module for CADSD GUI
// Re-export from app module when gui-bevy feature is enabled

#[cfg(feature = "gui-bevy")]
pub use crate::app::CadsdState;

#[cfg(feature = "gui-bevy")]
pub fn init_ui(state: &mut CadsdState) {
    crate::app::init_ui(state);
}