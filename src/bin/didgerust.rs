//! Binary entry point that launches the Bevy + egui UI

fn main() {
    // Forward to the UI implementation in the accurate crate
    cadsd_accurate::app::run_app();
}
