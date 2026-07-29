use bevy_egui::egui;

pub fn apply_visual_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(18, 18, 22);
    visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(180, 185, 200);
    
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(32, 34, 42);
    visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(220, 225, 240);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);

    
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 52, 68);
    visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);

    
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 80, 120);
    visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);

    
    visuals.selection.bg_fill = egui::Color32::from_rgb(60, 100, 180);
    // `Visuals::window_rounding` doesn't exist in the egui version used here.
    // Keep defaults for window rounding.

    
    ctx.set_visuals(visuals);
}
