//! GUI binary entrypoint for DidgeRust CADSD
//!
//! This binary launches the Bevy + egui application defined in `cadsd::app`.
//!
//! To run: cargo run --bin gui --features gui-bevy

#[cfg(feature = "gui-bevy")]
use bevy::prelude::*;
#[cfg(feature = "gui-bevy")]
use cadsd::app::{CadsdState, ui_system, setup, poll_optimizer_progress};
#[cfg(feature = "gui-bevy")]
use cadsd::waveguide::WaveguideEngine;

#[cfg(feature = "gui-bevy")]
fn draw_bore_gizmos(
    mut gizmos: ResMut<DebugGs>,
    state: Res<CadsdState>,
) {
    if state.active_tab != "geometry" {
        return;
    }
    
    let geo = cadsd::geo::Geo::make_cone(state.length as f64, state.top_diameter as f64, state.bottom_diameter as f64, state.segments.max(5) as usize);
    let engine = WaveguideEngine::from_geo(&geo);
    
    let start = Vec3::new(0.0, 0.0, 0.0);
    let end = Vec3::new(engine.total_length * 1000.0 as f32, 0.0, 0.0);
    
    gizmos.line(start, end, Color::WHITE);
    
    for (i, cell) in engine.cells.iter().enumerate().take(10) {
        let pos = start + (cell.length as f32 * 1000.0) * Vec3::X;
        let radius = (cell.d1 / 1000.0) as f32;
        gizmos.sphere(pos, radius, Color::GREEN.with_alpha(0.3));
    }
}

#[cfg(feature = "gui-bevy")]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "DidgeRust - CADSD GUI".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(CadsdState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (poll_optimizer_progress, ui_system))
        .run();
}

#[cfg(not(feature = "gui-bevy"))]
fn main() {
    println!("GUI feature is not enabled.");
    println!("Run with: cargo run --bin gui --features gui-bevy");
}
