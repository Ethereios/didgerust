//! DidgeRust - CADSD Application Launcher
//!
//! Professional acoustic simulation and design tool
//! Run with: cargo run --features gui-bevy --bin didgerust

fn main() {
    println!("🎵 DidgeRust - Computer-Aided Didgeridoo Sound Design");
    
    #[cfg(feature = "gui-bevy")]
    {
        use cadsd_accurate::didgerust_app;
        didgerust_app::run_didgerust_app();
    }
    
    #[cfg(not(feature = "gui-bevy"))]
    {
        eprintln!("Error: GUI feature not enabled!");
        eprintln!("Run with: cargo run --features gui-bevy --bin didgerust");
        std::process::exit(1);
    }
}
