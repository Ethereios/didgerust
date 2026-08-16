//! GUI binary entrypoint for DidgeRust CADSD
//!
//! This binary launches the Bevy + egui application defined in `cadsd::app`.
//!
//! To run: cargo run --bin gui --features gui-bevy

#[cfg(feature = "gui-bevy")]
use bevy::prelude::*;
#[cfg(feature = "gui-bevy")]
use cadsd::app::{CadsdState, draw_bore_gizmos, ui_system, setup, poll_optimizer_progress};

#[cfg(feature = "gui-bevy")]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "DidgeRust - CADSD GUI".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bevy_gizmos::GizmoPlugin)
        .insert_resource(CadsdState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, poll_optimizer_progress)
        .add_systems(Update, ui_system)
        .add_systems(Update, draw_bore_gizmos)
        .run();
}

#[cfg(not(feature = "gui-bevy"))]
fn main() {
    println!("GUI feature is not enabled.");
    println!("Run with: cargo run --bin gui --features gui-bevy");
}
