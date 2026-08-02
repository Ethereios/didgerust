use bevy_egui::egui;

pub fn apply_visual_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(32, 34, 42);
    visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(200, 205, 220);

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 52, 68);
    visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255);

    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 80, 120);
    visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255);

    visuals.selection.bg_fill = egui::Color32::from_rgb(60, 100, 180);

    ctx.set_visuals(visuals);
}