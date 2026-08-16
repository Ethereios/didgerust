use bevy_egui::egui;
use crate::app::CadsdState;

pub fn show_settings_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    use egui::Slider;
    
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
    
    ui.heading("📝 Status");
    ui.label(&state.sim_message);
}