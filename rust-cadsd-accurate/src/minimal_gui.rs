//! Minimal Working GUI - Stripped down version that actually works
//! This is a comprehensive fix for the white screen issue

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};

#[derive(Resource)]
struct AppState {
    counter: i32,
    message: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: 0,
            message: "GUI is working!".to_string(),
        }
    }
}

pub fn run_minimal_gui() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.2)))
        .insert_resource(AppState::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Update, ui_system)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn ui_system(mut contexts: EguiContexts, mut state: ResMut<AppState>) {
    use bevy_egui::egui;
    
    let ctx = contexts.ctx_mut().unwrap();
    
    // This is the key - we MUST use egui's CentralPanel
    egui::CentralPanel::default()
        .frame(egui::Frame {
            fill: egui::Color32::from_rgb(40, 40, 60),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.heading("✅ CADSD GUI Test");
            ui.separator();
            
            ui.label(&state.message);
            ui.label(format!("Counter: {}", state.counter));
            
            ui.separator();
            
            if ui.button("Click Me!").clicked() {
                state.counter += 1;
                state.message = format!("Button clicked {} times!", state.counter);
            }
            
            if ui.button("Reset").clicked() {
                state.counter = 0;
                state.message = "GUI is working!".to_string();
            }
            
            ui.separator();
            ui.label("If you can see this, the GUI is working!");
        });
}
