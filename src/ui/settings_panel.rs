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
        play_note(state);
    }
    
    ui.separator();
    
    ui.heading("🎨 Visualization");
    if ui.button("🔄 Refresh Preview").clicked() {
        state.sim_message = "Preview refreshed".to_string();
    }
    
    ComboBox::from_label("Bore Style")
        .selected_text(&state.bore_style)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.bore_style, "cone".to_string(), "Cone");
            ui.selectable_value(&mut state.bore_style, "cylinder".to_string(), "Cylinder");
            ui.selectable_value(&mut state.bore_style, "exponential".to_string(), "Exponential");
        });
    if ui.button("Apply Bore Style").clicked() {
        apply_bore_style(state);
    }

    ui.separator();
    
    ui.heading("📝 Status");
    ui.label(&state.sim_message);
    
    if ui.button("▶️ Run Simulation").clicked() {
        state.sim_message = "Running simulation...".to_string();
        crate::app::compute_spectrum(state);
        state.sim_message = format!("Simulation complete: {} frequency points", state.frequencies.len());
    }
}

fn play_note(state: &mut CadsdState) {
    let geo = crate::app::current_geo(state);
    let config = crate::audio::AudioConfig {
        sample_rate: state.audio_sample_rate,
        gain: state.audio_gain as f64,
        vibrato_depth: state.audio_vibrato_depth as f64,
        vibrato_freq: state.audio_vibrato_freq as f64,
    };
    
    match crate::audio::AudioProcessor::new(&geo, config) {
        Ok(processor) => {
            let freq = state.fundamental_freq.unwrap_or(100.0);
            processor.set_frequency(freq);
            processor.set_amplitude(
                state.audio_gain as f64,
                state.audio_vibrato_depth as f64,
                state.audio_vibrato_freq as f64,
            );
            let _ = processor.start();
            state.sim_message = format!("Playing note at {:.1} Hz", freq);
        }
        Err(e) => {
            state.sim_message = format!("Audio error: {}", e);
        }
    }
}

fn apply_bore_style(state: &mut CadsdState) {
    match state.bore_style.as_str() {
        "cone" => {
            state.top_diameter = 32.0;
            state.bottom_diameter = 65.0;
        }
        "cylinder" => {
            state.top_diameter = 40.0;
            state.bottom_diameter = 40.0;
        }
        "exponential" => {
            state.top_diameter = 20.0;
            state.bottom_diameter = 80.0;
        }
        _ => {}
    }
    state.sim_message = format!("Applied {} bore style", state.bore_style);
}
