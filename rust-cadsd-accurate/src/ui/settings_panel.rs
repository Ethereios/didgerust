use bevy_egui::egui;
use rfd::FileDialog;
use crate::app::CadsdState;
use crate::persistence::{AppSettings, ProjectState};

pub fn show_settings_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    let old_settings = AppSettings {
        temperature: state.temperature,
        wall_thickness: state.wall_thickness,
        show_3d: state.show_3d,
        mesh_rotation_enabled: state.mesh_rotation_enabled,
        mesh_rotation_speed: state.mesh_rotation_speed,
        color_scheme: state.color_scheme.clone(),
    };
    
    ui.heading("Settings");
    ui.separator();
    
    // ----- Project Persistence -----
    ui.heading("💾 Project");
    ui.horizontal(|ui| {
        if ui.button("Save Project").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("DidgeRust Project", &["dproj"])
                .set_file_name("my_didgeridoo.dproj")
                .save_file()
            {
                let project = ProjectState {
                    length: state.length,
                    top_diameter: state.top_diameter,
                    bottom_diameter: state.bottom_diameter,
                    segments: state.segments,
                    style_type: state.style_type.clone(),
                    bore_curve: state.bore_curve,
                    enable_mouthpiece: state.enable_mouthpiece,
                    mouthpiece_type: state.mouthpiece_type.clone(),
                    mouthpiece_length: state.mouthpiece_length,
                    enable_holes: state.enable_holes,
                    hole_count: state.hole_count,
                    hole_positions: state.hole_positions.clone(),
                    hole_diameters: state.hole_diameters.clone(),
                    wall_thickness: state.wall_thickness,
                    temperature: state.temperature,
                    target_frequency: state.target_frequency,
                    opt_bore_shape: state.opt_bore_shape.clone(),
                    opt_population_size: state.opt_population_size,
                    opt_generations: state.opt_generations,
                    opt_min_length: state.opt_min_length,
                    opt_max_length: state.opt_max_length,
                    opt_min_bell: state.opt_min_bell,
                    opt_max_bell: state.opt_max_bell,
                    opt_toots_input: state.opt_toots_input.clone(),
                };
                if let Err(e) = project.save_to_file(&path) {
                    state.last_error = Some(format!("Save failed: {}", e));
                } else {
                    state.simulation_message = format!("✓ Project saved to {}", path.display());
                }
            }
        }
        if ui.button("Load Project").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("DidgeRust Project", &["dproj"])
                .pick_file()
            {
                match ProjectState::load_from_file(&path) {
                    Ok(project) => {
                        state.length = project.length;
                        state.top_diameter = project.top_diameter;
                        state.bottom_diameter = project.bottom_diameter;
                        state.segments = project.segments;
                        state.style_type = project.style_type.clone();
                        state.bore_curve = project.bore_curve;
                        state.enable_mouthpiece = project.enable_mouthpiece;
                        state.mouthpiece_type = project.mouthpiece_type.clone();
                        state.mouthpiece_length = project.mouthpiece_length;
                        state.enable_holes = project.enable_holes;
                        state.hole_count = project.hole_count;
                        state.hole_positions = project.hole_positions.clone();
                        state.hole_diameters = project.hole_diameters.clone();
                        state.wall_thickness = project.wall_thickness;
                        state.temperature = project.temperature;
                        state.target_frequency = project.target_frequency;
                        state.opt_bore_shape = project.opt_bore_shape.clone();
                        state.opt_population_size = project.opt_population_size;
                        state.opt_generations = project.opt_generations;
                        state.opt_min_length = project.opt_min_length;
                        state.opt_max_length = project.opt_max_length;
                        state.opt_min_bell = project.opt_min_bell;
                        state.opt_max_bell = project.opt_max_bell;
                        state.opt_toots_input = project.opt_toots_input.clone();
                        
                        // Clear simulation results since geometry changed
                        state.frequencies.clear();
                        state.impedances.clear();
                        state.fundamental_freq = None;
                        state.resonance_notes.clear();
                        state.tairua_loss_value = 0.0;
                        state.last_error = None;
                        
                        state.simulation_message = format!("✓ Project loaded from {}", path.display());
                    }
                    Err(e) => {
                        state.last_error = Some(format!("Invalid project file: {}", e));
                    }
                }
            }
        }
    });
    
    ui.separator();
    
    // ----- Visualization Settings -----
    ui.heading("🎨 Visualization");
    ui.checkbox(&mut state.show_3d, "Show 3D bore preview");
    ui.checkbox(&mut state.mesh_rotation_enabled, "Auto-rotate mesh");
    if state.mesh_rotation_enabled {
        ui.add(egui::Slider::new(&mut state.mesh_rotation_speed, 5.0..=180.0)
            .text("Rotation speed (°/s)")
            .step_by(5.0));
    }
    
    ui.label("Color scheme:");
    egui::ComboBox::from_id_salt("settings_color_scheme")
        .selected_text(&state.color_scheme)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.color_scheme, "wood".to_string(), "🪵 Wood");
            ui.selectable_value(&mut state.color_scheme, "metal".to_string(), "🔩 Metal");
            ui.selectable_value(&mut state.color_scheme, "custom".to_string(), "🎨 Custom");
        });
    
    ui.separator();
    
    // ----- Simulation Settings -----
    ui.heading("⚙️ Simulation");
    ui.add(egui::Slider::new(&mut state.temperature, 0.0..=40.0)
        .text("Air temperature (°C)")
        .step_by(0.5));
    ui.small(format!("Speed of sound ≈ {:.1} m/s", 331.3 + 0.606 * state.temperature as f64));
    
    ui.add(egui::Slider::new(&mut state.wall_thickness, 1.0..=10.0)
        .text("Wall thickness (mm)")
        .step_by(0.5));
    
    ui.separator();
    
    // ----- About -----
    ui.heading("ℹ️ About");
    ui.small("DidgeRust — CADSD v0.1.0");
    ui.small("Computer-Aided Didgeridoo Sound Design");
    ui.small("Based on Frank Geipel's DidgeLab methodology");
    ui.add_space(4.0);
    ui.small("Bevy + egui | Rust");

    // Auto-persist settings if they changed
    if old_settings.temperature != state.temperature
        || old_settings.wall_thickness != state.wall_thickness
        || old_settings.show_3d != state.show_3d
        || old_settings.mesh_rotation_enabled != state.mesh_rotation_enabled
        || old_settings.mesh_rotation_speed != state.mesh_rotation_speed
        || old_settings.color_scheme != state.color_scheme
    {
        let new_settings = AppSettings {
            temperature: state.temperature,
            wall_thickness: state.wall_thickness,
            show_3d: state.show_3d,
            mesh_rotation_enabled: state.mesh_rotation_enabled,
            mesh_rotation_speed: state.mesh_rotation_speed,
            color_scheme: state.color_scheme.clone(),
        };
        let _ = new_settings.save_to_file("settings.json");
    }
}
