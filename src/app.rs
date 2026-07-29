//! Application module

pub mod app;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiContexts};
use bevy_egui::egui;

#[derive(Resource, Default)]
pub struct CadsdState {
    length: f32,
    top_diameter: f32,
    bottom_diameter: f32,
    segments: f32,
    active_tab: String,
    // Add all necessary state variables here
}

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
        .add_plugins(EguiPlugin::default())
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.2)))
        .add_systems(Startup, setup)
        .add_systems(Update, ui_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3d::default());
}

fn ui_system(mut contexts: EguiContexts, mut state: ResMut<CadsdState>) {
    // ... (implement UI logic here)
}