//! GUI binary entrypoint for DidgeRust CADSD
//!
//! This binary launches the Bevy + egui application with the full
//! CADSD interface including simulation, optimizer, geometry, and settings panels.
//!
//! To run: cargo run --bin gui --features gui-bevy

#[cfg(feature = "gui-bevy")]
use bevy::prelude::*;
<<<<<<< HEAD
use bevy_egui::{EguiPlugin, EguiContexts, egui};
use egui_plot::{Plot, Line, PlotPoints, VLine};

use cadsd_accurate::conv::{note_name, freq_to_note};
use cadsd_accurate::sim::{get_log_simulation_frequencies};
use cadsd_accurate::integration::{DefaultSimulator, DefaultOptimizer};
use cadsd_accurate::geo::Geo;
use rfd::FileDialog;
use std::fs;
use std::sync::mpsc;
use std::thread;
use std::sync::Arc;

#[derive(Debug)]
enum BackgroundTaskResult {
    Simulation {
        frequencies: Vec<f64>,
        impedances: Vec<f64>,
        fundamental: Option<f64>,
        resonance_notes: Vec<(f64, f64)>,
        tairua_loss: f64,
    },
    SimulationError(String),
    OptimizationProgress {
        generation: usize,
        total_generations: usize,
        best_loss: f64,
    },
    OptimizationSuccess {
        design: cadsd_accurate::inverse_design::DesignResult,
        frequencies: Vec<f64>,
        impedances: Vec<f64>,
    },
    OptimizationError(String),
}

#[derive(Resource)]
struct BackgroundChannels {
    sender: mpsc::Sender<BackgroundTaskResult>,
}

impl BackgroundChannels {
    fn new(sender: mpsc::Sender<BackgroundTaskResult>) -> Self {
        Self { sender }
    }
}

#[derive(Resource, Debug)]
pub struct CadsdState {
    pub length: f32,
    pub top_diameter: f32,
    pub bottom_diameter: f32,
    pub segments: usize,
    pub bore_style: String,
    pub bore_curve: f32,
    pub enable_mouthpiece: bool,
    pub mouthpiece_type: String,
    pub mouthpiece_length: f32,
    pub mouthpiece_diameter: f32,
    pub enable_holes: bool,
    pub hole_count: usize,
    pub hole_positions: Vec<f32>,
    pub hole_diameters: Vec<f32>,
    pub wall_thickness: f32,
    pub temperature: f32,
    pub frequencies: Vec<f64>,
    pub impedances: Vec<f64>,
    pub fundamental_freq: Option<f64>,
    pub resonance_notes: Vec<(f64, f64)>,
    pub target_frequency: f64,
    pub tairua_loss_value: f64,
    pub show_wireframe: bool,
    pub mesh_rotation_enabled: bool,
    pub mesh_rotation_speed: f32,
    pub color_scheme: String,
    pub is_simulating: bool,
    pub last_error: Option<String>,
    pub sim_message: String,
    pub pending_optimization: bool,
    pub optimization_progress: f32,
    pub opt_population_size: usize,
    pub opt_generations: usize,
    pub opt_bore_shape: String,
    pub opt_min_length: f32,
    pub opt_max_length: f32,
    pub opt_min_bell: f32,
    pub opt_max_bell: f32,
    pub opt_toots_input: String,
    pub active_tab: String,
}

impl Default for CadsdState {
    fn default() -> Self {
        Self {
            length: 950.0,
            top_diameter: 35.0,
            bottom_diameter: 85.0,
            segments: 50,
            bore_style: "cone".to_string(),
            bore_curve: 0.0,
            enable_mouthpiece: false,
            mouthpiece_type: "reed".to_string(),
            mouthpiece_length: 50.0,
            mouthpiece_diameter: 35.0,
            enable_holes: false,
            hole_count: 0,
            hole_positions: vec![],
            hole_diameters: vec![],
            wall_thickness: 4.0,
            temperature: 20.0,
            frequencies: vec![],
            impedances: vec![],
            fundamental_freq: None,
            resonance_notes: vec![],
            target_frequency: 65.41,
            tairua_loss_value: 0.0,
            show_wireframe: false,
            mesh_rotation_enabled: false,
            mesh_rotation_speed: 30.0,
            color_scheme: "wood".to_string(),
            is_simulating: false,
            last_error: None,
            sim_message: "Ready. Click 'Run Simulation'".to_string(),
            pending_optimization: false,
            optimization_progress: 0.0,
            opt_population_size: 40,
            opt_generations: 30,
            opt_bore_shape: "any".to_string(),
            opt_min_length: 1000.0,
            opt_max_length: 2000.0,
            opt_min_bell: 40.0,
            opt_max_bell: 100.0,
            opt_toots_input: "".to_string(),
            active_tab: "forward".to_string(),
        }
    }
}
=======
#[cfg(feature = "gui-bevy")]
use bevy_egui::EguiPlugin;
#[cfg(feature = "gui-bevy")]
use cadsd::app::{CadsdState, ui_system, setup};
>>>>>>> adfb9d3 (feat: comprehensive architecture docs, UI overhaul, research foundations, and evolution engine improvements)

#[cfg(feature = "gui-bevy")]
fn main() {
<<<<<<< HEAD
    let state = CadsdState::default();
    let (tx, rx) = mpsc::channel();
    
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "CADSD - Didgeridoo Analyzer".into(),
            resolution: (1280.0, 720.0).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(EguiPlugin);
    app.insert_resource(ClearColor(Color::srgb(0.07, 0.07, 0.10)));
    app.insert_resource(state);
    app.insert_resource(BackgroundChannels::new(tx));
    app.add_systems(Startup, setup);
    app.add_systems(Update, ui_system);
    app.add_systems(Update, poll_background_tasks);
    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3d::default());
    commands.spawn(DirectionalLight {
        illuminance: 15000.0,
        ..default()
    });
}

fn create_geometry(state: &CadsdState) -> Geo {
    let n = state.segments.max(2);
    let mut points = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / (n - 1) as f64;
        let x = t * state.length as f64;
        let d = match state.bore_style.as_str() {
            "exponential" => {
                let ratio = state.top_diameter as f64 / state.bottom_diameter as f64;
                state.top_diameter as f64 * ratio.powf(t)
            }
            "cylinder" => state.top_diameter as f64,
            _ => {
                state.top_diameter as f64 + t * (state.bottom_diameter as f64 - state.top_diameter as f64)
            }
        };
        points.push([x, d]);
    }
    Geo::new(points)
}

fn start_simulation(state: &mut CadsdState, tx: &mpsc::Sender<BackgroundTaskResult>) {
    let freqs = get_log_simulation_frequencies();
    let geo = create_geometry(state);
    
    state.is_simulating = true;
    state.sim_message = "Starting simulation...".to_string();
    state.last_error = None;
    
    let tx_clone = tx.clone();
    thread::spawn(move || {
        match cadsd_accurate::sim::acoustical_simulation(&geo, &freqs, "tlm_python") {
            Ok(impedances) => {
                let (fundamental, _) = cadsd_accurate::sim::get_fundamental(&geo, "tlm_python", 20.0)
                    .unwrap_or((65.41, 1.0));
                
                let mut resonance_notes = vec![];
                for i in 1..impedances.len()-1 {
                    if impedances[i] > impedances[i-1] && impedances[i] > impedances[i+1] {
                        resonance_notes.push((freqs[i], impedances[i]));
                    }
                }
                
                let _ = tx_clone.send(BackgroundTaskResult::Simulation {
                    frequencies: freqs,
                    impedances,
                    fundamental: Some(fundamental),
                    resonance_notes,
                    tairua_loss: 0.0,
                });
            }
            Err(e) => {
                let _ = tx_clone.send(BackgroundTaskResult::SimulationError(e.to_string()));
            }
        }
    });
}

fn start_optimization(state: &mut CadsdState, tx: &mpsc::Sender<BackgroundTaskResult>) {
    let tx_clone = tx.clone();
    
    let target_frequency = state.target_frequency;
    let opt_bore_shape = state.opt_bore_shape.clone();
    let opt_min_length = state.opt_min_length;
    let opt_max_length = state.opt_max_length;
    let opt_min_bell = state.opt_min_bell;
    let opt_max_bell = state.opt_max_bell;
    let opt_toots_input = state.opt_toots_input.clone();
    let opt_population_size = state.opt_population_size;
    let opt_generations = state.opt_generations;
    
    state.is_simulating = true;
    state.sim_message = "Initializing evolutionary designer...".to_string();
    state.last_error = None;
    
    thread::spawn(move || {
        let mut target = cadsd_accurate::evo::TargetSound::new(target_frequency as f64);
        let bore_shape = match opt_bore_shape.as_str() {
            "cylindrical" => cadsd_accurate::evo::BoreShapePreference::Cylindrical,
            "conical" => cadsd_accurate::evo::BoreShapePreference::Conical,
            "flared" => cadsd_accurate::evo::BoreShapePreference::Flared,
            _ => cadsd_accurate::evo::BoreShapePreference::Any,
        };
        target = target.with_bore_shape(bore_shape);
        target = target.with_length_range(opt_min_length as f64, opt_max_length as f64);
        target = target.with_bell_range(opt_min_bell as f64, opt_max_bell as f64);
        
        if !opt_toots_input.trim().is_empty() {
            for part in opt_toots_input.split(',') {
                if let Ok(freq) = part.trim().parse::<f64>() {
                    target = target.with_toot(freq);
                }
            }
        }
        
        let tx_progress = tx_clone.clone();
        let progress_cb = Arc::new(move |gen: usize, best_loss: f64| {
            let _ = tx_progress.send(BackgroundTaskResult::OptimizationProgress {
                generation: gen,
                total_generations: opt_generations,
                best_loss,
            });
        });
        
        let opt = DefaultOptimizer;
        match opt.optimize(target, opt_population_size, opt_generations, Some(progress_cb)) {
            Ok(result) => {
                let sim = DefaultSimulator;
                let mut frequencies = Vec::with_capacity(513);
                let step = (2000.0 - 20.0) / 512.0;
                for i in 0..=512 {
                    frequencies.push(20.0 + i as f64 * step);
                }
                let impedances = sim.simulate(&result.geometry, &frequencies).unwrap_or_default();
                
                let _ = tx_clone.send(BackgroundTaskResult::OptimizationSuccess {
                    design: result,
                    frequencies,
                    impedances,
                });
            }
            Err(e) => {
                let _ = tx_clone.send(BackgroundTaskResult::OptimizationError(e.to_string()));
            }
        }
    });
}

fn ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<CadsdState>,
    channels: Res<BackgroundChannels>,
) {
    let ctx = contexts.ctx_mut();
    
    let mut visuals = egui::Visuals::dark();
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(18, 18, 22);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(26, 28, 36);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 52, 68);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 80, 120);
    ctx.set_visuals(visuals);
    
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("CADSD");
            ui.vertical(|ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 1.0);
                ui.small("Computer-Aided Didgeridoo Sound Design");
            });
            
            ui.separator();
            
            ui.selectable_value(&mut state.active_tab, "forward".to_string(), "Forward Design");
            ui.selectable_value(&mut state.active_tab, "inverse".to_string(), "Inverse Design");
            ui.selectable_value(&mut state.active_tab, "analysis".to_string(), "Analysis");
            ui.selectable_value(&mut state.active_tab, "export".to_string(), "Export");
            ui.selectable_value(&mut state.active_tab, "settings".to_string(), "Settings");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.small("v0.1.0");
            });
        });
    });
    
    egui::SidePanel::left("controls_panel")
        .default_width(300.0)
        .show(ctx, |ui| {
            match state.active_tab.as_str() {
                "forward" => show_forward_design_panel(ui, &mut state, &channels.sender),
                "inverse" => show_inverse_design_panel(ui, &mut state, &channels.sender),
                "analysis" => show_analysis_panel(ui, &mut state),
                "export" => show_export_panel(ui, &mut state),
                "settings" => show_settings_panel(ui, &mut state),
                _ => {}
            }
        });
    
    if !state.impedances.is_empty() {
        egui::TopBottomPanel::bottom("spectrum_panel")
            .default_height(280.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Impedance Spectrum");
                    if let Some(fund) = state.fundamental_freq {
                        ui.separator();
                        ui.label(format!(
                            "Fundamental: {:.2} Hz ({})",
                            fund,
                            note_name(freq_to_note(fund))
                        ));
                    }
                });
                
                let points: Vec<[f64; 2]> = state
                    .frequencies
                    .iter()
                    .zip(state.impedances.iter())
                    .map(|(&f, &z)| [f, z])
                    .collect();
                
                let line = Line::new("Impedance", PlotPoints::from(points))
                    .color(egui::Color32::from_rgb(100, 150, 255));
                
                Plot::new("impedance_plot")
                    .view_aspect(2.0)
                    .allow_zoom(true)
                    .allow_scroll(true)
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        
                        if let Some(fund) = state.fundamental_freq {
                            plot_ui.vline(VLine::new(fund));
                        }
                    });
            });
    }
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("CADSD - Didgeridoo Analyzer");
        ui.add_space(10.0);
        
        ui.label(&format!(
            "Geometry: {}mm {} | Temp: {:.0}C | Wall: {:.0}mm",
            state.length, state.bore_style, state.temperature, state.wall_thickness
        ));
        
        if state.is_simulating {
            ui.spinner();
            ui.label(&state.sim_message);
        }
        
        ui.add_space(10.0);
        ui.separator();
        
        if !state.impedances.is_empty() {
            ui.label(&format!(
                "Impedance spectrum computed: {} frequency points",
                state.frequencies.len()
            ));
            if let Some(freq) = state.fundamental_freq {
                ui.label(format!(
                    "Peak frequency: {:.1} Hz ({})",
                    freq,
                    note_name(freq_to_note(freq))
                ));
            }
        } else {
            ui.label("Click 'Run Simulation' in the Forward Design panel");
        }
    });
}

fn poll_background_tasks(
    channels: Res<BackgroundChannels>,
    mut state: ResMut<CadsdState>,
) {
    while let Ok(msg) = channels.sender.try_recv() {
        match msg {
            BackgroundTaskResult::Simulation {
                frequencies,
                impedances,
                fundamental,
                resonance_notes,
                tairua_loss,
            } => {
                state.is_simulating = false;
                state.frequencies = frequencies;
                state.impedances = impedances;
                state.fundamental_freq = fundamental;
                state.resonance_notes = resonance_notes;
                state.tairua_loss_value = tairua_loss;
                state.sim_message = format!(
                    "Complete - Found {} resonances",
                    state.resonance_notes.len()
                );
                state.last_error = None;
            }
            BackgroundTaskResult::SimulationError(err) => {
                state.is_simulating = false;
                state.last_error = Some(err);
                state.sim_message = "Simulation failed".to_string();
            }
            BackgroundTaskResult::OptimizationProgress {
                generation,
                total_generations,
                best_loss,
            } => {
                state.optimization_progress = (generation + 1) as f32 / total_generations as f32;
                state.sim_message = format!(
                    "Evolving... Gen {}/{} (Best Loss: {:.4})",
                    generation + 1,
                    total_generations,
                    best_loss
                );
            }
            BackgroundTaskResult::OptimizationSuccess {
                design,
                frequencies,
                impedances,
            } => {
                state.is_simulating = false;
                state.optimization_progress = 0.0;
                
                let best_geo = &design.geometry;
                if let (Some(first), Some(last)) = (best_geo.geo.first(), best_geo.geo.last()) {
                    state.length = best_geo.length() as f32;
                    state.segments = best_geo.geo.len().saturating_sub(1).max(1);
                    state.top_diameter = first[1] as f32;
                    state.bottom_diameter = last[1] as f32;
                    let is_cyl = (state.top_diameter - state.bottom_diameter).abs() < 0.1;
                    state.bore_style = if is_cyl {
                        "cylinder".to_string()
                    } else {
                        "cone".to_string()
                    };
                }
                
                state.fundamental_freq = Some(design.fundamental_freq);
                state.resonance_notes = design.resonances;
                state.tairua_loss_value = design.loss;
                state.frequencies = frequencies;
                state.impedances = impedances;
                
                state.sim_message = format!(
                    "Optimization Complete! Loss: {:.4}",
                    design.loss
                );
                state.last_error = None;
            }
            BackgroundTaskResult::OptimizationError(err) => {
                state.is_simulating = false;
                state.optimization_progress = 0.0;
                state.last_error = Some(err);
                state.sim_message = "Optimization failed".to_string();
            }
        }
    }
}

fn show_forward_design_panel(ui: &mut egui::Ui, state: &mut CadsdState, tx: &mpsc::Sender<BackgroundTaskResult>) {
    ui.heading("Forward Design");
    ui.separator();
    ui.small("Define geometry -> Simulate acoustics");
    ui.separator();
    
    ui.heading("Base Didgeridoo");
    
    egui::ComboBox::new(egui::Id::new("bore_style"), "Bore style")
        .selected_text(&state.bore_style)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.bore_style, "cone".to_string(), "Cone (TLM)");
            ui.selectable_value(&mut state.bore_style, "cylinder".to_string(), "Cylinder (Waveguide)");
            ui.selectable_value(&mut state.bore_style, "exponential".to_string(), "Exponential (Complex)");
        });
    
    ui.add_enabled_ui(!state.is_simulating, |ui| {
        ui.add(egui::Slider::new(&mut state.length, 500.0..=3000.0)
            .text("Length (mm)")
            .step_by(10.0));
        
        ui.add(egui::Slider::new(&mut state.top_diameter, 10.0..=50.0)
            .text("Mouth Diameter (mm)")
            .step_by(0.5));
        
        ui.add(egui::Slider::new(&mut state.bottom_diameter, 20.0..=100.0)
            .text("Bell Diameter (mm)")
            .step_by(0.5));
        
        ui.add(egui::Slider::new(&mut state.segments, 5..=500usize).text("Segments"));
    });
    
    ui.separator();
    
    ui.collapsing("Mouthpiece", |ui| {
        ui.checkbox(&mut state.enable_mouthpiece, "Enable mouthpiece");
        if state.enable_mouthpiece {
            egui::ComboBox::new(egui::Id::new("mouthpiece_type"), "Mouthpiece type")
                .selected_text(&state.mouthpiece_type)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.mouthpiece_type, "reed".to_string(), "Reed (clarinet/sax)");
                    ui.selectable_value(&mut state.mouthpiece_type, "embouchure_hole".to_string(), "Embouchure (flute)");
                    ui.selectable_value(&mut state.mouthpiece_type, "fipple".to_string(), "Fipple (recorder)");
                    ui.selectable_value(&mut state.mouthpiece_type, "cup".to_string(), "Cup (brass)");
                });
            
            ui.add(egui::Slider::new(&mut state.mouthpiece_length, 20.0..=150.0)
                .text("Mouthpiece Length (mm)")
                .step_by(5.0));
            
            ui.add(egui::Slider::new(&mut state.mouthpiece_diameter, 10.0..=50.0)
                .text("Mouthpiece Diameter (mm)")
                .step_by(0.5));
        }
    });
    
    ui.separator();
    
    ui.collapsing("Finger Holes", |ui| {
        ui.checkbox(&mut state.enable_holes, "Enable holes");
        if state.enable_holes {
            ui.add(egui::Slider::new(&mut state.hole_count, 0usize..=12).text("Number of holes"));
            if state.hole_positions.len() != state.hole_count {
                let mut new_positions = Vec::with_capacity(state.hole_count);
                let mut new_diameters = Vec::with_capacity(state.hole_count);
                for i in 0..state.hole_count {
                    let pos = if i < state.hole_positions.len() {
                        state.hole_positions[i]
                    } else {
                        ((i + 1) as f32) * state.length / ((state.hole_count + 1) as f32)
                    };
                    let dia = if i < state.hole_diameters.len() {
                        state.hole_diameters[i]
                    } else {
                        10.0
                    };
                    new_positions.push(pos);
                    new_diameters.push(dia);
                }
                state.hole_positions = new_positions;
                state.hole_diameters = new_diameters;
            }
            for i in 0..state.hole_count {
                ui.horizontal(|ui| {
                    ui.label(format!("Hole {}:", i + 1));
                    ui.add(egui::Slider::new(&mut state.hole_positions[i], 0.0..=state.length)
                        .text("Pos (mm)")
                        .step_by(1.0));
                    ui.add(egui::Slider::new(&mut state.hole_diameters[i], 5.0..=30.0)
                        .text("Dia (mm)")
                        .step_by(0.5));
                });
            }
        }
    });
    
    ui.separator();
    
    ui.heading("Simulation");
    
    if state.is_simulating {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(&state.sim_message);
        });
    } else if let Some(error) = &state.last_error {
        ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
    } else if !state.impedances.is_empty() {
        ui.colored_label(egui::Color32::GREEN, "Complete");
    }
    
    if ui.button(if state.is_simulating { "Running..." } else { "Run Simulation" }).clicked() && !state.is_simulating {
        start_simulation(state, tx);
    }
    
    ui.separator();
    
    if !state.impedances.is_empty() {
        ui.heading("Results");
        if let Some(freq) = state.fundamental_freq {
            ui.label(format!(
                "Fundamental: {:.1} Hz ({})",
                freq,
                note_name(freq_to_note(freq))
            ));
        }
        ui.label(format!("Resonances found: {}", state.resonance_notes.len()));
        for (i, res) in state.resonance_notes.iter().take(5).enumerate() {
            ui.label(format!("{}. {:.1} Hz (Z={:.1})", i + 1, res.0, res.1));
        }
    }
}

fn show_inverse_design_panel(ui: &mut egui::Ui, state: &mut CadsdState, tx: &mpsc::Sender<BackgroundTaskResult>) {
    ui.heading("Inverse Design");
    ui.separator();
    ui.small("Define target sound & constraints -> Optimize geometry");
    ui.separator();
    
    ui.add_enabled_ui(!state.is_simulating, |ui| {
        ui.heading("Target Sound");
        
        ui.add(egui::Slider::new(&mut state.target_frequency, 40.0..=200.0)
            .text("Fundamental (Hz)")
            .step_by(0.1));
        ui.small(format!("Note: {}", note_name(freq_to_note(state.target_frequency))));
        
        ui.horizontal(|ui| {
            ui.label("Preferred Shape:");
            egui::ComboBox::new(egui::Id::new("opt_bore_shape"), "Shape")
                .selected_text(match state.opt_bore_shape.as_str() {
                    "cylindrical" => "Cylindrical",
                    "conical" => "Conical",
                    "flared" => "Flared",
                    _ => "Any",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.opt_bore_shape, "any".to_string(), "Any");
                    ui.selectable_value(&mut state.opt_bore_shape, "cylindrical".to_string(), "Cylindrical");
                    ui.selectable_value(&mut state.opt_bore_shape, "conical".to_string(), "Conical");
                    ui.selectable_value(&mut state.opt_bore_shape, "flared".to_string(), "Flared");
                });
        });
        
        ui.label("Target Toots (comma-separated Hz):");
        ui.text_edit_singleline(&mut state.opt_toots_input);
        ui.small("Example: 196.0, 261.6");
        
        ui.separator();
        ui.heading("Constraints");
        
        ui.label("Length Range (mm):");
        ui.add(egui::Slider::new(&mut state.opt_min_length, 500.0..=2500.0).text("Min Length"));
        ui.add(egui::Slider::new(&mut state.opt_max_length, 500.0..=2500.0).text("Max Length"));
        if state.opt_min_length > state.opt_max_length {
            state.opt_max_length = state.opt_min_length;
        }
        
        ui.label("Bell Diameter Range (mm):");
        ui.add(egui::Slider::new(&mut state.opt_min_bell, 30.0..=150.0).text("Min Bell"));
        ui.add(egui::Slider::new(&mut state.opt_max_bell, 30.0..=150.0).text("Max Bell"));
        if state.opt_min_bell > state.opt_max_bell {
            state.opt_max_bell = state.opt_min_bell;
        }
        
        ui.separator();
        ui.heading("Search Parameters");
        ui.add(egui::Slider::new(&mut state.opt_population_size, 10..=100).text("Population"));
        ui.add(egui::Slider::new(&mut state.opt_generations, 5..=100).text("Generations"));
    });
    
    ui.separator();
    
    if state.is_simulating {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().size(16.0));
                ui.label(&state.sim_message);
            });
            ui.add(egui::ProgressBar::new(state.optimization_progress)
                .text(format!("{:.1}%", state.optimization_progress * 100.0))
                .animate(true));
        });
    } else if let Some(error) = &state.last_error {
        ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
    } else if state.tairua_loss_value > 0.0 {
        ui.colored_label(egui::Color32::GREEN, format!("Complete. Loss: {:.4}", state.tairua_loss_value));
    }
    
    ui.add_enabled_ui(!state.is_simulating, |ui| {
        if ui.button(if state.is_simulating { "Optimizing..." } else { "Run Optimization" }).clicked() && !state.is_simulating {
            start_optimization(state, tx);
        }
    });
}

fn show_analysis_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("Analysis");
    ui.separator();
    
    if state.resonance_notes.is_empty() {
        ui.label("Run a simulation first to see analysis");
        return;
    }
    
    ui.heading("Resonance Peaks");
    
    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
        for (i, (freq, amp)) in state.resonance_notes.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("{}", i + 1));
                ui.label(format!("{:.2} Hz", freq));
                ui.label(format!("({})", note_name(freq_to_note(*freq))));
                ui.small(format!("imp: {:.2e}", amp));
            });
        }
    });
    
    ui.separator();
    ui.heading("Harmonic Analysis");
    
    if let Some(fund) = state.fundamental_freq {
        ui.small(format!(
            "Fundamental: {:.2} Hz ({})",
            fund,
            note_name(freq_to_note(fund))
        ));
        
        for (i, (freq, _)) in state.resonance_notes.iter().take(8).enumerate() {
            let ratio = freq / fund;
            let expected = (i + 1) as f64;
            let deviation = (ratio - expected) / expected * 100.0;
            
            ui.horizontal(|ui| {
                ui.small(format!("H{}", i + 1));
                ui.small(format!("{:.2}x", ratio));
                ui.small(format!("{:+.1}%", deviation));
            });
        }
    }
}

fn show_export_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("Export");
    ui.separator();
    
    ui.heading("Export Geometry");
    ui.horizontal(|ui| {
        if ui.button("JSON").clicked() {
            let geo = create_geometry(state);
            if let Ok(json) = serde_json::to_string_pretty(&geo.geo) {
                if let Some(path) = FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name("geometry.json")
                    .save_file()
                {
                    let _ = fs::write(&path, json);
                    state.sim_message = format!("Exported to {}", path.display());
                }
            }
        }
        if ui.button("TXT").clicked() {
            let geo = create_geometry(state);
            let mut txt = String::new();
            for pt in &geo.geo {
                txt.push_str(&format!("{:.6}\t{:.6}\n", pt[0], pt[1]));
            }
            if let Some(path) = FileDialog::new()
                .add_filter("TXT", &["txt"])
                .set_file_name("geometry.txt")
                .save_file()
            {
                let _ = fs::write(&path, txt);
                state.sim_message = format!("Exported to {}", path.display());
            }
        }
        if ui.button("OBJ").clicked() {
            let geo = create_geometry(state);
            let mut obj = String::from("# CADSD Geometry - Exported by GUI\n");
            for (i, pt) in geo.geo.iter().enumerate() {
                obj.push_str(&format!("v {:.6} 0.0 {:.6}\n", pt[0], pt[1]));
                if i > 0 {
                    obj.push_str(&format!("l {} {}\n", i, i + 1));
                }
            }
            if let Some(path) = FileDialog::new()
                .add_filter("OBJ", &["obj"])
                .set_file_name("geometry.obj")
                .save_file()
            {
                let _ = fs::write(&path, obj);
                state.sim_message = format!("Exported to {}", path.display());
            }
        }
    });
    
    ui.separator();
    ui.heading("Export Simulation Data");
    
    if state.impedances.is_empty() {
        ui.label("No simulation data. Run simulation first.");
    } else if ui.button("CSV (Impedance)").clicked() {
        let mut csv = String::new();
        csv.push_str("frequency_hz,impedance_magnitude\n");
        for (f, z) in state.frequencies.iter().zip(state.impedances.iter()) {
            csv.push_str(&format!("{:.6},{:.6}\n", f, z));
        }
        if let Some(path) = FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name("impedance.csv")
            .save_file()
        {
            let _ = fs::write(&path, csv);
            state.sim_message = format!("Exported to {}", path.display());
        }
    }
    
    ui.separator();
    ui.heading("Current Design");
    ui.label(&format!("Length: {:.1} mm", state.length));
    ui.label(&format!("Mouth: {:.1} mm", state.top_diameter));
    ui.label(&format!("Bell: {:.1} mm", state.bottom_diameter));
    ui.label(&format!("Segments: {}", state.segments));
}

fn show_settings_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("Settings");
    ui.separator();
    
    ui.heading("Visualization");
    ui.checkbox(&mut state.mesh_rotation_enabled, "Enable mesh rotation");
    if state.mesh_rotation_enabled {
        ui.add(egui::Slider::new(&mut state.mesh_rotation_speed, 0.0..=180.0)
            .text("Rotation speed (deg/s)")
            .step_by(5.0));
    }
    ui.label("Color scheme:");
    egui::ComboBox::new(egui::Id::new("color_scheme"), "Scheme")
        .selected_text(&state.color_scheme)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.color_scheme, "wood".to_string(), "Wood");
            ui.selectable_value(&mut state.color_scheme, "metal".to_string(), "Metal");
            ui.selectable_value(&mut state.color_scheme, "custom".to_string(), "Custom");
        });
    
    ui.separator();
    ui.heading("Audio");
    ui.add(egui::Slider::new(&mut state.temperature, 0.0..=40.0)
        .text("Temperature (C)")
        .step_by(1.0));
    ui.add(egui::Slider::new(&mut state.wall_thickness, 1.0..=10.0)
        .text("Wall thickness (mm)")
        .step_by(0.5));
}
=======
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "DidgeRust - CADSD GUI".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
.add_plugins(EguiPlugin)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.2)))
        .insert_resource(CadsdState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, ui_system)
        .run();
}

#[cfg(not(feature = "gui-bevy"))]
fn main() {
    println!("GUI feature is not enabled.");
    println!("Run with: cargo run --bin gui --features gui-bevy");
}
>>>>>>> adfb9d3 (feat: comprehensive architecture docs, UI overhaul, research foundations, and evolution engine improvements)
