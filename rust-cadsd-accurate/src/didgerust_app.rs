//! DidgeRust Application Launcher
//!
//! Simple wrapper that calls the existing Bevy app

pub fn run_didgerust_app() {
    // Use the existing working Bevy application
    crate::app::run_app();
}
