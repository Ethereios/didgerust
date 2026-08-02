use bevy_egui::egui;
use crate::app::CadsdState;

pub fn show_settings_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    use egui::{ComboBox, Slider};
    
    ui.heading("⚙️ Settings");
    ui.separator();
    
    ui.heading("🔧 Geometry");
    ui.add(Slider::new(&mut state.length, 500.0..=3000.0)
        .text("Length (mm)")
        .step_by(10.0));
    ui.add(Slider::new(&mut state.top_diameter, 10.0..=50.0)
        .text("Mouth Diameter (mm)")
        .step_by(0.5));
    ui.add(Slider::new(&mut state.bottom_diameter, 20.0..=100.0)
        .text("Bell Diameter (mm)")
        .step_by(0.5));
    ui.add(Slider::new(&mut state.segments, 5.0..=500.0)
        .text("Segments")
        .step_by(1.0));

    ui.separator();
    
    ui.heading("🎵 Sound");
    if ui.button("🔊 Play Note").clicked() {
        state.sim_message = "Playing note...".to_string();
    }
    
    ui.separator();
    
    ui.heading("🎨 Visualization");
    if ui.button("🔄 Refresh Preview").clicked() {
        state.sim_message = "Refreshing visualization...".to_string();
    }
    
    ComboBox::from_label("Bore Style")
        .selected_text(&state.active_tab)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.active_tab, "cone".to_string(), "Cone");
            ui.selectable_value(&mut state.active_tab, "cylinder".to_string(), "Cylinder");
            ui.selectable_value(&mut state.active_tab, "exponential".to_string(), "Exponential");
        });

    ui.separator();
    
    ui.heading("📝 Status");
    ui.label(&state.sim_message);
    
    if ui.button("▶️ Run Simulation").clicked() {
        state.sim_message = "Running simulation...".to_string();
    }
}