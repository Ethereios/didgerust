use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiContexts, egui};

#[derive(Resource, Default, Clone)]
struct AppState {
    length: f32,
    top_diameter: f32,
    bottom_diameter: f32,
    segments: usize,
    active_tab: String,
    frequencies: Vec<f64>,
    impedances: Vec<f64>,
    fundamental_freq: Option<f64>,
}

impl AppState {
    fn default() -> Self {
        Self {
            length: 1500.0,
            top_diameter: 32.0,
            bottom_diameter: 65.0,
            segments: 30,
            active_tab: "forward".to_string(),
            frequencies: Vec::new(),
            impedances: Vec::new(),
            fundamental_freq: None,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CADSD GUI".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.2)))
        .insert_resource(AppState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, ui_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3d::default());
}

fn ui_system(
    mut contexts: EguiContexts,
    _state: Res<AppState>,
) {
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("CADSD GUI");
    });
}