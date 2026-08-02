#[cfg(feature = "gui-bevy")]
use bevy::prelude::*;

#[cfg(feature = "gui-bevy")]
use bevy_egui::EguiContexts;

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
    use crate::ui::{show_export_panel, show_settings_panel};
    use bevy_egui::egui;

    let ctx = contexts.ctx_mut().expect("egui context");
    crate::ui::apply_visual_theme(ctx);

    ctx.left_panel("settings", |ui| {
        show_settings_panel(ui, &mut state);
    });

    ctx.right_panel("export", |ui| {
        show_export_panel(ui, &mut state);
    });

    ctx.central_panel(|ui| {
        ui.heading("CADSD - Didgeridoo Analyzer");
        ui.label("Design your didgeridoo");
        ui.label(&state.sim_message);
    });
}