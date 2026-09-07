#[cfg(feature = "gui-bevy")]
use bevy::prelude::*;

#[cfg(feature = "gui-bevy")]
use bevy_egui::EguiContexts;
#[cfg(feature = "gui-bevy")]
use bevy_egui::egui;

#[cfg(feature = "gui-bevy")]
#[derive(Resource, Default)]
pub struct CadsdState {
    pub length: f32,
    pub top_diameter: f32,
    pub bottom_diameter: f32,
    pub segments: f32,
    pub active_tab: String,
    pub frequencies: Vec<f64>,
    pub impedances: Vec<f64>,
    pub fundamental_freq: Option<f64>,
    pub sim_message: String,
}

#[cfg(feature = "gui-bevy")]
pub fn setup(mut commands: Commands) {
    commands.spawn(Camera3d::default());
}

#[cfg(feature = "gui-bevy")]
pub fn ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<CadsdState>,
) {
    let ctx = contexts.ctx_mut().expect("egui context");
    crate::ui::apply_visual_theme(ctx);

    // Use existing panel pattern but ensure proper API
    egui::SidePanel::left("settings").show(ctx, |ui| {
        show_settings_panel(ui, &mut state);
    });
    egui::SidePanel::right("export").show(ctx, |ui| {
        show_export_panel(ui, &mut state);
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("CADSD - Didgeridoo Analyzer");
        ui.label("Design your didgeridoo");
        ui.label(&state.sim_message);
    });
}

// Helper functions to avoid using panel methods that don't exist
#[cfg(feature = "gui-bevy")]
fn show_settings_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("Settings");
    ui.add(egui::Slider::new(&mut state.length, 500.0..=3000.0)
        .text("Length (mm)")
        .step_by(10.0));
}

#[cfg(feature = "gui-bevy")]
fn show_export_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("Export");
    if ui.button("JSON Export").clicked() {
        state.sim_message = "Exported JSON".to_string();
    }
    ui.separator();
    if ui.button("OBJ Export").clicked() {
        state.sim_message = "Exported OBJ".to_string();
    }
}