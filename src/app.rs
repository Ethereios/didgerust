//! Application module - Full CADSD GUI with strategy selection
//!
//! This module implements the complete GUI for the CADSD didgeridoo simulator,
//! including simulation strategy selection, evolutionary optimizer controls,
//! and visualization panels.
//!
//! # Architecture
//! - `CadsdState` is the Bevy resource holding all UI state
//! - `ui_system` is the main Bevy system that renders egui panels
//! - Panel functions (`show_simulation_panel`, `show_optimizer_panel`, etc.)
//!   render individual tabs
//! - Action functions (`compute_spectrum`, `run_comparison_simulation`, etc.)
//!   perform the actual computations

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use num_complex::Complex;
use crate::sim::{SimulationStrategy, DidgeridooSimulator};
use crate::evo::{MutationStrategy, EvolutionaryOptimizer, EvolutionParameters};
use crate::Geo;
use crate::tonehole::Tonehole;
use crate::persistence::{AppSettings, OptimizerCheckpoint, OptimizerGeoState};
use std::sync::{Arc, atomic::{AtomicBool, Ordering as SyncOrdering}, mpsc};
use std::thread;
use rfd::FileDialog;
// If using audio integration, it would be:
// use crate::audio::AudioProcessor;

/// Comparison data for strategy overlay
#[derive(Clone)]
pub struct CompareData {
    pub frequencies: Vec<f64>,
    pub tlm_impedances: Vec<f64>,
    pub wg_impedances: Vec<f64>,
    pub ci_impedances: Vec<f64>,
}

/// History entry for undo/redo in geometry panel
#[derive(Clone)]
pub struct GeoHistoryEntry {
    pub geo_points: Vec<[f64; 2]>,
    pub label: String,
}

/// Progress message from optimizer background thread
#[derive(Debug)]
pub enum OptimizerProgressMsg {
    GenerationUpdate { generation: usize, total: usize, best_loss: f64 },
    Complete { best_loss: f64, genome_json: String },
    Error(String),
}

/// Resource holding the receiver for optimizer progress messages
#[derive(Resource)]
pub struct OptimizerChannels {
    pub rx: Option<std::sync::Mutex<mpsc::Receiver<OptimizerProgressMsg>>>,
    pub pause_flag: Option<Arc<AtomicBool>>,
}

/// Comprehensive application state held as a Bevy resource
#[derive(Resource, Clone)]
pub struct CadsdState {
    // Geometry parameters
    pub length: f32,
    pub top_diameter: f32,
    pub bottom_diameter: f32,
    pub segments: usize,
    
    // UI state
    pub active_tab: String,
    pub sim_message: String,
    
    // Simulation results
    pub frequencies: Vec<f64>,
    pub impedances: Vec<f64>,
    pub phases: Vec<f64>,
    pub impedances_no_toneholes: Vec<f64>,
    pub phases_no_toneholes: Vec<f64>,
    pub fundamental_freq: Option<f64>,
    
    // Strategy selection
    pub simulation_strategy: SimulationStrategy,
    pub mutation_strategy: MutationStrategy,
    pub budget_ops: f64,
    
    // Optimizer parameters
    pub population_size: usize,
    pub num_generations: usize,
    pub mutation_rate: f32,
    pub crossover_rate: f32,
    pub elite_size: usize,
    pub best_loss: Option<f64>,
    pub generation_progress: f32,
    pub current_generation: usize,
    pub optimizer_running: bool,
    pub optimizer_paused: bool,
    pub optimizer_error: Option<String>,
    pub optimizer_n_toneholes: usize,
    pub use_surrogate_loss: bool,
    pub surrogate_trained: bool,
    pub use_gradient_optimizer: bool,
    pub temperature: f32,
    
    // Frequency grid config
    pub freq_min: f32,
    pub freq_max: f32,
    pub freq_points: usize,
    pub freq_cents_step: f32,
    pub use_log_grid: bool,
    
    // ---- Phase B: Simulation Panel ----
    pub compare_data: Option<CompareData>,
    pub show_compare_dialog: bool,
    pub spectrum_hover_pos: Option<egui::Pos2>,
    pub spectrum_hover_text: String,
    pub spectrum_plot_id: u64,
    pub show_phase: bool,
    pub show_peak_markers: bool,
    pub show_tonehole_difference: bool,
    
    // ---- Phase B: Optimizer Panel ----
    pub loss_component_toggles: Vec<(String, bool, f64)>,
    pub loss_history: Vec<(usize, f64)>,
    pub validation_report: String,
    
    // ---- Phase B: Geometry Panel ----
    pub geo_history: Vec<GeoHistoryEntry>,
    pub geo_history_index: usize,
    pub show_add_bubble_dialog: bool,
    pub show_stretch_dialog: bool,
    pub bubble_center: f32,
    pub bubble_width: f32,
    pub bubble_height: f32,
    pub stretch_factor: f32,
    pub toneholes: Vec<Tonehole>,
    pub drag_tonehole_index: Option<usize>,
    pub selected_tonehole_index: Option<usize>,
    pub selected_tonehole_preset: crate::tonehole::ToneholePreset,
    pub drag_tonehole_3d: Option<usize>,
    pub tonehole_impedance_freqs: Vec<f64>,
    pub tonehole_impedances: Vec<f64>,
    pub surrogate_model: Option<crate::prime_conv::SurrogateLossFunction>,
    
    // ---- Phase B: Settings Panel ----
    pub theme: String,
    pub log_verbosity: u8,
    pub default_strategy: String,
    pub default_mutation: String,
    pub pressure_pa: f64,
    pub relative_humidity: f64,
    pub bore_style: String,
    
    // Audio controls
    pub audio_enabled: bool,
    pub audio_gain: f32,
    pub audio_vibrato_depth: f32,
    pub audio_vibrato_freq: f32,
    pub audio_sample_rate: u32,
    pub audio_running: bool,
}

impl Default for CadsdState {
    fn default() -> Self {
        // Load persisted settings if available
        let settings = AppSettings::load_from_file("settings.json");
        
        let default_strat = match settings.default_strategy.as_str() {
            "Tlm" => SimulationStrategy::Tlm,
            "Waveguide" => SimulationStrategy::Waveguide,
            "ComplexImpedance" => SimulationStrategy::ComplexImpedance,
            _ => SimulationStrategy::Tlm,
        };
        let default_mut = match settings.default_mutation.as_str() {
            "Gaussian" => MutationStrategy::Gaussian,
            "PrimeSequence" => MutationStrategy::PrimeSequence,
            _ => MutationStrategy::Gaussian,
        };
        
        Self {
            length: 1500.0,
            top_diameter: 32.0,
            bottom_diameter: 65.0,
            segments: 30,
            active_tab: "simulation".to_string(),
            sim_message: String::new(),
            frequencies: Vec::new(),
            impedances: Vec::new(),
            phases: Vec::new(),
            impedances_no_toneholes: Vec::new(),
            phases_no_toneholes: Vec::new(),
            fundamental_freq: None,
            simulation_strategy: default_strat,
            mutation_strategy: default_mut,
            budget_ops: 100000.0,
            population_size: 50,
            num_generations: 100,
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            elite_size: 5,
            best_loss: None,
            generation_progress: 0.0,
            current_generation: 0,
            optimizer_running: false,
            optimizer_paused: false,
            optimizer_error: None,
            optimizer_n_toneholes: 0,
            use_surrogate_loss: false,
            surrogate_trained: false,
            use_gradient_optimizer: false,
            temperature: settings.temperature,
            freq_min: 20.0,
            freq_max: 2000.0,
            freq_points: 200,
            freq_cents_step: 5.0,
            use_log_grid: false,
            // Phase B: Simulation
            compare_data: None,
            show_compare_dialog: false,
            spectrum_hover_pos: None,
            spectrum_hover_text: String::new(),
            spectrum_plot_id: 0,
            show_phase: false,
            show_peak_markers: false,
            show_tonehole_difference: false,
            // Phase B: Optimizer
            loss_component_toggles: vec![
                ("integer_harmonic".to_string(), true, 5.0),
                ("near_integer".to_string(), true, 5.0),
                ("stretched_odd".to_string(), true, 5.0),
                ("harmonic_splitting".to_string(), true, 5.0),
                ("peak_quantity".to_string(), true, 2.0),
                ("peak_amplitude".to_string(), true, 2.0),
                ("scale_tuning".to_string(), true, 5.0),
            ],
            loss_history: Vec::new(),
            validation_report: String::new(),
            bore_style: "cone".to_string(),
            // Phase B: Geometry
            geo_history: Vec::new(),
            geo_history_index: 0,
            show_add_bubble_dialog: false,
            show_stretch_dialog: false,
            bubble_center: 750.0,
            bubble_width: 200.0,
            bubble_height: 40.0,
            stretch_factor: 1.1,
            toneholes: Vec::new(),
            drag_tonehole_index: None,
            selected_tonehole_index: None,
            selected_tonehole_preset: crate::tonehole::ToneholePreset::None,
            drag_tonehole_3d: None,
            tonehole_impedance_freqs: Vec::new(),
            tonehole_impedances: Vec::new(),
            surrogate_model: None,
            // Phase B: Settings
            theme: settings.theme.clone(),
            log_verbosity: settings.log_verbosity,
            default_strategy: settings.default_strategy.clone(),
            default_mutation: settings.default_mutation.clone(),
            pressure_pa: settings.pressure_pa,
            relative_humidity: settings.relative_humidity,
            audio_enabled: false,
            audio_gain: 0.5,
            audio_vibrato_depth: 0.0,
            audio_vibrato_freq: 5.0,
            audio_sample_rate: 44100,
            audio_running: false,
        }
    }
}

/// Initialize the UI state
pub fn init_ui(state: &mut CadsdState) {
    state.active_tab = "simulation".to_string();
}

/// Setup system - spawns camera for 3D view
pub fn setup(mut commands: Commands) {
    commands.spawn(Camera3d::default());
    commands.insert_resource(OptimizerChannels { rx: None, pause_flag: None });
}

/// Draw 3D bore preview using bevy_gizmos with tonehole markers
pub fn draw_bore_gizmos(
    mut gizmos: Gizmos,
    state: Res<CadsdState>,
) {
    let geo = current_geo(&state);
    let points = &geo.geo;
    
    if points.len() < 2 {
        return;
    }
    
    let scale = 0.01;
    
    // Draw bore as a series of line segments
    for i in 0..points.len() - 1 {
        let x1 = points[i][0] as f32 * scale;
        let r1 = points[i][1] as f32 * scale;
        let x2 = points[i + 1][0] as f32 * scale;
        let r2 = points[i + 1][1] as f32 * scale;
        
        gizmos.line(
            Vec3::new(x1, 0.0, 0.0),
            Vec3::new(x2, 0.0, 0.0),
            Color::WHITE,
        );
        
        gizmos.line(
            Vec3::new(x1, r1, 0.0),
            Vec3::new(x2, r2, 0.0),
            Color::srgb(0.8, 0.4, 0.2),
        );
        gizmos.line(
            Vec3::new(x1, -r1, 0.0),
            Vec3::new(x2, -r2, 0.0),
            Color::srgb(0.8, 0.4, 0.2),
        );
    }
    
    // Draw tonehole markers in 3D
    for (i, th) in state.toneholes.iter().enumerate() {
        let x = th.x as f32 * scale;
        let r = th.diameter as f32 * scale * 0.5;
        let is_selected = state.drag_tonehole_3d == Some(i) || state.selected_tonehole_index == Some(i);
        let color = if th.is_open {
            if is_selected { Color::srgb(1.0, 0.9, 0.0) } else { Color::srgb(1.0, 0.3, 0.3) }
        } else {
            if is_selected { Color::srgb(0.9, 0.9, 0.0) } else { Color::srgb(0.5, 0.5, 0.5) }
        };
        
        gizmos.sphere(Vec3::new(x, r, 0.0), r * 0.8, 8, color);
    }
}

/// Helper: get current geo from state
pub fn current_geo(state: &CadsdState) -> Geo {
    Geo::make_cone(
        state.length as f64,
        state.top_diameter as f64,
        state.bottom_diameter as f64,
        state.segments
    )
}

/// Push current geometry state onto undo history with validation
pub fn push_geo_history(state: &mut CadsdState, label: &str) {
    let geo = current_geo(state);
    
    // Validate geometry points before adding to history
    if let Err(e) = crate::persistence::validate_geo_points(&geo.geo) {
        log::warn!("Skipping history entry due to validation error: {}", e);
        return;
    }
    
    // Truncate any redo entries
    if state.geo_history_index < state.geo_history.len() {
        state.geo_history.truncate(state.geo_history_index);
    }
    state.geo_history.push(GeoHistoryEntry {
        geo_points: geo.geo.clone(),
        label: label.to_string(),
    });
    state.geo_history_index = state.geo_history.len();
    // Cap history at 50 entries
    if state.geo_history.len() > 50 {
        state.geo_history.remove(0);
        state.geo_history_index = state.geo_history.len();
    }
}

/// Run gradient-based optimization using DifferentiableTLM + Adam.
fn run_gradient_optimization(
    length: f64,
    top_diameter: f64,
    bottom_diameter: f64,
    n_segments: usize,
    n_toneholes: usize,
    num_generations: usize,
    tx: &mpsc::Sender<OptimizerProgressMsg>,
) -> OptimizerProgressMsg {
    let geo = Geo::make_cone(length, top_diameter, bottom_diameter, n_segments);
    let segments = crate::sim::create_segments_from_geo(&geo.geo);
    let constants = crate::sim::AcousticConstants::default();
    let target_freq = 50.0;

    let mut diff_tlm = crate::diff_tlm::DifferentiableTLM::new(segments, target_freq, constants);
    let mut adam = crate::diff_tlm::AdamOptimizer::new(0.01);

    let num_params = diff_tlm.segments.len() * 3;
    let mut params = vec![0.0; num_params];
    let mut grads = vec![0.0; num_params];

    for (i, seg) in diff_tlm.segments.iter().enumerate() {
        params[i * 3] = seg.length;
        params[i * 3 + 1] = seg.d0;
        params[i * 3 + 2] = seg.d1;
    }

    let mut best_loss = f64::INFINITY;

    for gen in 0..num_generations {
        let z_in = diff_tlm.forward();
        let loss = (z_in.re * z_in.re + z_in.im * z_in.im) as f64;
        let loss_grad = num_complex::Complex64::new(loss, 0.0);

        let seg_grads = diff_tlm.backward(num_complex::Complex64::new(loss_grad.re as f32, loss_grad.im as f32));

        for (i, (dl, dd0, dd1)) in seg_grads.into_iter().enumerate() {
            grads[i * 3] = dl;
            grads[i * 3 + 1] = dd0;
            grads[i * 3 + 2] = dd1;
        }

        adam.step(&mut params, &grads);

        for (i, seg) in diff_tlm.segments.iter_mut().enumerate() {
            seg.length = params[i * 3].max(1e-4);
            seg.d0 = params[i * 3 + 1].max(1e-4);
            seg.d1 = params[i * 3 + 2].max(1e-4);
        }

        if loss < best_loss {
            best_loss = loss;
        }

        let _ = tx.send(OptimizerProgressMsg::GenerationUpdate {
            generation: gen + 1,
            total: num_generations,
            best_loss,
        });
    }

    let final_segs = diff_tlm.get_segments();
    let mut geo_points = Vec::new();
    let mut x_acc = 0.0;
    for seg in &final_segs {
        x_acc += seg.l * 1000.0;
        geo_points.push([x_acc, seg.d1 * 1000.0]);
    }

    let genome_json = serde_json::json!({
        "segments": geo_points,
        "loss": best_loss,
    });

    OptimizerProgressMsg::Complete {
        best_loss,
        genome_json,
    }
}

/// Start evolutionary optimization in a background thread.
pub fn start_optimization(state: &mut CadsdState, mut channels: ResMut<OptimizerChannels>) {
    if state.optimizer_running {
        return;
    }

    let (tx, rx) = mpsc::channel();
    channels.rx = Some(std::sync::Mutex::new(rx));
    let pause_flag = Arc::new(AtomicBool::new(false));
    channels.pause_flag = Some(pause_flag.clone());
    
    state.optimizer_running = true;
    state.optimizer_paused = false;
    state.optimizer_error = None;
    state.generation_progress = 0.0;
    state.current_generation = 0;
    state.best_loss = None;

    // Clone parameters for the thread
    let population_size = state.population_size;
    let num_generations = state.num_generations;
    let mutation_rate = state.mutation_rate as f64;
    let crossover_rate = state.crossover_rate as f64;
    let elite_size = state.elite_size;
    let mutation_strategy = state.mutation_strategy;
    let _simulation_strategy = state.simulation_strategy;
    let length = state.length as f64;
    let top_diameter = state.top_diameter as f64;
    let bottom_diameter = state.bottom_diameter as f64;
    let segments = state.segments;
    let n_toneholes = state.optimizer_n_toneholes;
    let use_surrogate = state.use_surrogate_loss && state.surrogate_trained;
    let use_gradient = state.use_gradient_optimizer;
    let trained_surrogate = state.surrogate_model.take();

    thread::spawn(move || {
        let result = (|| {
            if use_gradient {
                return run_gradient_optimization(
                    length,
                    top_diameter,
                    bottom_diameter,
                    segments,
                    n_toneholes,
                    num_generations,
                    &tx,
                );
            }

            let base_loss = crate::loss::CompositeTairuaLoss::with_default_components(50.0);
            let loss_function: Box<dyn crate::evo::LossFunction> = if use_surrogate {
                if let Some(surrogate) = trained_surrogate {
                    Box::new(surrogate)
                } else {
                    Box::new(base_loss)
                }
            } else {
                Box::new(base_loss)
            };
            let genome_template = crate::evo::KigaliGenome::new(
                segments,
                top_diameter,
                bottom_diameter * 0.8,
                bottom_diameter * 1.2,
                length * 1.5,
                length * 0.5,
                0,
                0.3,
                0.0,
                length * 0.7,
                n_toneholes,
            );

            let params = EvolutionParameters {
                population_size,
                generation_size: (population_size / 2).max(1),
                num_generations,
                mutation_rate,
                crossover_rate,
                elite_size,
                mutation_strategy,
                crossover_strategy: crate::evo::CrossoverStrategy::SinglePoint,
                convergence_patience: 10,
                convergence_threshold: 1e-6,
            };

            let mut optimizer = EvolutionaryOptimizer::with_random_population(
                Box::new(loss_function),
                &genome_template,
                population_size,
                params,
            );
            optimizer.set_pause_flag(pause_flag);

            let progress_cb = {
                let tx = tx.clone();
                move |gen: usize, best_loss: f64| {
                    let _ = tx.send(OptimizerProgressMsg::GenerationUpdate {
                        generation: gen + 1,
                        total: num_generations,
                        best_loss,
                    });
                }
            };

            // Run evolution with progress callback
            let result = optimizer.evolve_with_progress(progress_cb);
            match result {
                Ok(best) => {
                    let genome_json = best.representation().to_string();
                    OptimizerProgressMsg::Complete {
                        best_loss: best.loss().unwrap_or(f64::INFINITY),
                        genome_json,
                    }
                }
                Err(e) => OptimizerProgressMsg::Error(e.to_string()),
            }
        })();

        let _ = tx.send(result);
    });
}

/// Main UI system - renders all panels
pub fn ui_system(mut contexts: EguiContexts, mut state: ResMut<CadsdState>, channels: ResMut<OptimizerChannels>) {
    let ctx = contexts.ctx_mut().expect("Failed to get egui context");
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "viewport".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    
    // Apply theme
    if state.theme == "light" {
        viewport_ui.style_mut().visuals = egui::Visuals::light();
    } else {
        viewport_ui.style_mut().visuals = egui::Visuals::dark();
    }
    
    // Top toolbar with tab navigation
    let _ = egui::Panel::top("toolbar").show(&mut viewport_ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading("🎵 DidgeRust - CADSD GUI");
            ui.separator();
            ui.selectable_value(&mut state.active_tab, "simulation".to_string(), "🔬 Simulation");
            ui.selectable_value(&mut state.active_tab, "optimizer".to_string(), "🧬 Optimizer");
            ui.selectable_value(&mut state.active_tab, "geometry".to_string(), "📐 Geometry");
            ui.selectable_value(&mut state.active_tab, "settings".to_string(), "⚙️ Settings");
        });
    });

    // Left sidebar with strategy controls
    let _ = egui::Panel::left("sidebar")
        .resizable(true)
        .default_size(200.0)
        .show(&mut viewport_ui, |ui| {
            ui.heading("Strategy");
            ui.separator();
            
            ui.label("Simulation Method:");
            ui.radio_value(&mut state.simulation_strategy, SimulationStrategy::Tlm, "TLM (Stable)");
            ui.radio_value(&mut state.simulation_strategy, SimulationStrategy::Waveguide, "Waveguide");
            ui.radio_value(&mut state.simulation_strategy, SimulationStrategy::ComplexImpedance, "Complex Impedance");
            
            ui.separator();
            ui.label("Mutation Strategy:");
            ui.radio_value(&mut state.mutation_strategy, MutationStrategy::Gaussian, "Gaussian");
            ui.radio_value(&mut state.mutation_strategy, MutationStrategy::PrimeSequence, "Prime Sequence");
            
            ui.separator();
            ui.label("Conservation Budget:");
            ui.add(egui::Slider::new(&mut state.budget_ops, 1000.0..=1_000_000.0).text("Max Ops"));
            if state.budget_ops < 10_000.0 {
                ui.colored_label(egui::Color32::YELLOW, "⚠️ Low budget may limit evaluations");
            }
            ui.separator();
            
            if ui.button("📤 Export Geometry").clicked() {
                let geo = current_geo(&state);
                let geo_json = serde_json::to_string_pretty(&geo.geo).unwrap_or_default();
                if let Some(path) = FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .add_filter("CSV", &["csv"])
                    .set_file_name("geometry.json")
                    .save_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ext == "csv" {
                            let mut csv = String::from("x_mm,diameter_mm\n");
                            for &[x, d] in &geo.geo {
                                csv.push_str(&format!("{},{}\n", x, d));
                            }
                            if let Err(e) = std::fs::write(&path, csv) {
                                log::error!("Failed to export geometry CSV: {}", e);
                            } else {
                                log::info!("Geometry exported as CSV to {}", path.display());
                            }
                        } else {
                            if let Err(e) = std::fs::write(&path, &geo_json) {
                                log::error!("Failed to export geometry: {}", e);
                            } else {
                                log::info!("Geometry exported to {}", path.display());
                            }
                        }
                    }
                }
            }
            
            if ui.button("📊 Compare Strategies").clicked() {
                state.show_compare_dialog = true;
                run_comparison_simulation(&mut state);
            }
        });

    // Main central panel - renders the active tab
    let _ = egui::CentralPanel::default().show(&mut viewport_ui, |ui| {
        match state.active_tab.as_str() {
            "simulation" => show_simulation_panel(ui, &mut state),
            "optimizer" => show_optimizer_panel(ui, &mut state, channels),
            "geometry" => show_geometry_panel(ui, &mut state),
            "settings" => show_settings_panel(ui, &mut state),
            _ => {
                ui.heading("Unknown tab");
            }
        }
    });
    
    // Compare Strategies dialog (overlay)
    if state.show_compare_dialog {
        let mut close = false;
        egui::Window::new("📊 Strategy Comparison")
            .default_size([600.0, 400.0])
            .show(ctx, |ui| {
                show_compare_plot(ui, &mut state);
                if ui.button("Close").clicked() {
                    close = true;
                }
            });
        if close {
            state.show_compare_dialog = false;
        }
    }

    // Add Bubble dialog
    if state.show_add_bubble_dialog {
        let mut close = false;
        egui::Window::new("➕ Add Bubble")
            .default_size([300.0, 200.0])
            .show(ctx, |ui| {
                ui.label("Bubble Position (mm):");
                let max_center = state.length - 50.0;
                    ui.add(egui::Slider::new(&mut state.bubble_center, 50.0..=max_center));
                ui.label("Bubble Width (mm):");
                ui.add(egui::Slider::new(&mut state.bubble_width, 10.0..=500.0));
                ui.label("Bubble Height (mm):");
                ui.add(egui::Slider::new(&mut state.bubble_height, -80.0..=80.0));
                ui.separator();
                if ui.button("✅ Add Bubble").clicked() {
                    let mut geo = current_geo(&state);
                    push_geo_history(&mut state, "add_bubble");
                    geo.make_bubble(state.bubble_center as f64, state.bubble_width as f64, state.bubble_height as f64);
                    // Update state from geo
                    state.length = geo.length() as f32;
                    state.top_diameter = geo.geo.first().map(|p| p[1] as f32).unwrap_or(32.0);
                    state.bottom_diameter = geo.geo.last().map(|p| p[1] as f32).unwrap_or(65.0);
                    state.segments = geo.geo.len().max(5);
                    close = true;
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
        if close {
            state.show_add_bubble_dialog = false;
        }
    }

    // Stretch dialog
    if state.show_stretch_dialog {
        let mut close = false;
        egui::Window::new("↔️ Stretch Geometry")
            .default_size([300.0, 150.0])
            .show(ctx, |ui| {
                ui.label("Stretch Factor:");
                ui.add(egui::Slider::new(&mut state.stretch_factor, 0.5..=2.0).text(""));
                ui.separator();
                if ui.button("✅ Apply Stretch").clicked() {
                    let mut geo = current_geo(&state);
                    push_geo_history(&mut state, "stretch");
                    geo.stretch(state.stretch_factor as f64);
                    state.length = geo.length() as f32;
                    state.segments = geo.geo.len().max(5);
                    close = true;
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
        if close {
            state.show_stretch_dialog = false;
        }
    }
}

/// Poll optimizer progress from background thread and update state.
pub fn poll_optimizer_progress(mut state: ResMut<CadsdState>, mut channels: ResMut<OptimizerChannels>) {
    if let Some(ref flag) = channels.pause_flag {
        flag.store(state.optimizer_paused, SyncOrdering::Relaxed);
    }
    
    let rx_opt = channels.rx.take();
    if let Some(rx_opt) = rx_opt {
        if let Ok(rx) = rx_opt.lock() {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    OptimizerProgressMsg::GenerationUpdate { generation, total, best_loss } => {
                        state.current_generation = generation;
                        state.generation_progress = (generation as f32 / total as f32).min(1.0);
                        state.best_loss = Some(best_loss);
                        state.optimizer_error = None;
                        state.loss_history.push((generation, best_loss));
                    }
                    OptimizerProgressMsg::Complete { best_loss, genome_json: _ } => {
                        state.optimizer_running = false;
                        state.optimizer_paused = false;
                        state.best_loss = Some(best_loss);
                        state.generation_progress = 1.0;
                        state.current_generation = state.num_generations;
                        channels.rx = None;
                        channels.pause_flag = None;
                    }
                    OptimizerProgressMsg::Error(err) => {
                        state.optimizer_running = false;
                        state.optimizer_paused = false;
                        state.optimizer_error = Some(err);
                        channels.rx = None;
                        channels.pause_flag = None;
                    }
                }
            }
        }
    }
    
    // Update progress bar even if no new messages
    if state.optimizer_running && !state.optimizer_paused {
        state.generation_progress = (state.current_generation as f32 / state.num_generations as f32).min(1.0);
    }
}

/// Simulation Panel - shows spectrum results and controls
fn show_simulation_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("🔬 Simulation Results");
    ui.separator();
    
    // Frequency grid configuration
    ui.collapsing("⚙️ Frequency Grid Settings", |ui| {
        ui.horizontal(|ui| {
            ui.label("Min (Hz):");
            ui.add(egui::Slider::new(&mut state.freq_min, 10.0..=100.0).text(""));
        });
        ui.horizontal(|ui| {
            ui.label("Max (Hz):");
            ui.add(egui::Slider::new(&mut state.freq_max, 100.0..=5000.0).text(""));
        });
        ui.checkbox(&mut state.use_log_grid, "Logarithmic (cents-based) spacing");
        if state.use_log_grid {
            ui.horizontal(|ui| {
                ui.label("Cents per step:");
                ui.add(egui::Slider::new(&mut state.freq_cents_step, 1.0..=50.0).text("cents"));
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("Points:");
                ui.add(egui::Slider::new(&mut state.freq_points, 50..=1000).text(""));
            });
        }
    });
    
    ui.separator();
    
    // Results display
    if let Some(freq) = state.fundamental_freq {
        ui.label(format!("🎵 Fundamental Frequency: **{:.2} Hz**", freq));
    }
    
    ui.label(format!("Strategy: **{:?}**", state.simulation_strategy));
    if state.generation_progress > 0.0 {
        ui.label(format!("Budget: **{:.0} / {:.0} ops**", 
            state.budget_ops * state.generation_progress as f64, state.budget_ops));
    }
    
    if !state.toneholes.is_empty() && state.simulation_strategy == SimulationStrategy::ComplexImpedance {
        ui.colored_label(egui::Color32::YELLOW, "⚠️ Toneholes are ignored in Complex Impedance strategy");
    }
    
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.show_phase, "Show phase");
        ui.checkbox(&mut state.show_peak_markers, "Show peak markers");
        ui.checkbox(&mut state.show_tonehole_difference, "Show tonehole difference");
    });
    
    if !state.frequencies.is_empty() && !state.impedances.is_empty() {
        ui.separator();
        ui.label(format!("📈 Spectrum: **{} frequency points computed**", state.frequencies.len()));
        
        // Spectrum plot with tooltips
        draw_spectrum_plot(ui, state);
    }
    
    ui.separator();
    ui.label("Controls:");
    ui.horizontal(|ui| {
        if ui.button("Compute Spectrum").clicked() {
            compute_spectrum(state);
        }
        if ui.button("Find Resonance Peaks").clicked() {
            find_peaks(state);
        }
        if ui.button("Export CSV").clicked() {
            export_spectrum_csv(state);
        }
        if ui.button("Validate TLM").clicked() {
            validate_tlm(state);
        }
    });
    
    if !state.validation_report.is_empty() {
        ui.separator();
        ui.label("Validation Report:");
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            ui.label(egui::RichText::new(&state.validation_report).monospace());
        });
    }
}

/// Draw the impedance spectrum plot with tooltips
fn draw_spectrum_plot(ui: &mut egui::Ui, state: &mut CadsdState) {
    let min_freq = state.frequencies.first().copied().unwrap_or(0.0) as f32;
    let max_freq = state.frequencies.last().copied().unwrap_or(1.0) as f32;
    let max_imp_with = state.impedances.iter().cloned().fold(0.0f64, f64::max).max(1.0) as f32;
    let max_imp_no = state.impedances_no_toneholes.iter().cloned().fold(0.0f64, f64::max).max(1.0) as f32;
    let max_imp = max_imp_with.max(max_imp_no);
    
    let font_id = egui::FontId::proportional(12.0);
    let has_no_toneholes = !state.impedances_no_toneholes.is_empty() && state.toneholes.len() > 1;
    
    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
        let plot_size = egui::vec2(ui.available_width(), 300.0);
        let (response, painter) = ui.allocate_painter(plot_size, egui::Sense::hover());
        let rect = response.rect;
        
        // Draw axes
        painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::StrokeKind::Middle);
        
        // Draw impedance curve
        if state.frequencies.len() > 1 {
            let freq_range = (max_freq - min_freq).max(1.0);
            let points: Vec<egui::Pos2> = state.frequencies.iter().zip(state.impedances.iter())
                .map(|(&f, &z)| {
                    let x = rect.left() + ((f as f32 - min_freq) / freq_range) * rect.width();
                    let y = rect.bottom() - (z as f32 / max_imp) * rect.height();
                    egui::pos2(x, y)
                })
                .collect();
            
            if points.len() > 1 {
                painter.add(egui::Shape::line(points.clone(), egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 200, 255))));
            }
            
            // Draw no-tonehole curve for comparison
            if has_no_toneholes {
                let points_no: Vec<egui::Pos2> = state.frequencies.iter().zip(state.impedances_no_toneholes.iter())
                    .map(|(&f, &z)| {
                        let x = rect.left() + ((f as f32 - min_freq) / freq_range) * rect.width();
                        let y = rect.bottom() - (z as f32 / max_imp) * rect.height();
                        egui::pos2(x, y)
                    })
                    .collect();
                
                if points_no.len() > 1 {
                    painter.add(egui::Shape::line(points_no.clone(), egui::Stroke::new(1.5, egui::Color32::from_rgb(150, 150, 150))));
                }
                
                // Draw filled difference region if enabled
                if state.show_tonehole_difference && points.len() > 1 && points_no.len() > 1 {
                    let mut diff_points = Vec::new();
                    for (p_with, p_no) in points.iter().zip(points_no.iter()) {
                        diff_points.push(*p_with);
                        diff_points.push(*p_no);
                    }
                    if diff_points.len() >= 4 {
                        painter.add(egui::Shape::convex_polygon(
                            diff_points,
                            egui::Color32::from_rgba_unmultiplied(255, 100, 100, 40),
                            egui::Stroke::new(0.0, egui::Color32::TRANSPARENT),
                        ));
                    }
                }
            }
            
            // Draw phase overlay
            if state.show_phase && !state.phases.is_empty() {
                let max_phase = state.phases.iter().cloned().fold(0.0f64, |a, b| a.max(b.abs())).max(180.0) as f32;
                let phase_points: Vec<egui::Pos2> = state.frequencies.iter().zip(state.phases.iter())
                    .map(|(&f, &p)| {
                        let x = rect.left() + ((f as f32 - min_freq) / freq_range) * rect.width();
                        let y = rect.bottom() - (p as f32 / max_phase) * rect.height();
                        egui::pos2(x, y)
                    })
                    .collect();
                
                if phase_points.len() > 1 {
                    painter.add(egui::Shape::line(phase_points, egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 165, 0))));
                }
                
                painter.text(
                    egui::pos2(rect.right() - 5.0, rect.top() + 5.0),
                    egui::Align2::RIGHT_TOP,
                    format!("±{:.0}°", max_phase),
                    egui::FontId::proportional(10.0),
                    egui::Color32::from_rgb(255, 165, 0),
                );
            }
            
            // Draw peak markers
            if state.show_peak_markers && state.frequencies.len() > 2 {
                let complex_spectrum: Vec<Complex<f64>> = state.impedances.iter()
                    .zip(state.phases.iter())
                    .map(|(&z, _p)| Complex::new(z, 0.0))
                    .collect();
                
                let peaks = crate::sim::find_peaks(&state.frequencies, &complex_spectrum);
                for (_, freq, mag) in peaks.iter().take(10) {
                    let x = rect.left() + ((*freq as f32 - min_freq) / freq_range) * rect.width();
                    let y = rect.bottom() - (*mag as f32 / max_imp) * rect.height();
                    painter.circle_filled(egui::pos2(x, y), 4.0, egui::Color32::RED);
                    painter.line_segment(
                        [egui::pos2(x, rect.bottom()), egui::pos2(x, y)],
                        egui::Stroke::new(1.0, egui::Color32::RED),
                    );
                }
            }
            
            // Tooltip on hover
            if response.hovered() {
                if let Some(mouse_pos) = response.hover_pos() {
                    state.spectrum_hover_pos = Some(mouse_pos);
                    if let Some(nearest_idx) = find_nearest_point(points.as_slice(), mouse_pos) {
                        if nearest_idx < state.frequencies.len() {
                            let f = state.frequencies[nearest_idx];
                            let z = state.impedances[nearest_idx];
                            let phase = state.phases.get(nearest_idx).map_or(0.0, |&p| p);
                            let mut text = format!("f: {:.2} Hz\nZ: {:.2e}\nPhase: {:.1}°", f, z, phase);
                            if has_no_toneholes {
                                let z_no = state.impedances_no_toneholes.get(nearest_idx).map_or(0.0, |&v| v);
                                text.push_str(&format!("\nZ (no holes): {:.2e}", z_no));
                            }
                            state.spectrum_hover_text = text;
                        }
                    }
                }
            } else {
                state.spectrum_hover_pos = None;
            }
            
            // Draw tooltip
            if let Some(hover_pos) = state.spectrum_hover_pos {
                if !state.spectrum_hover_text.is_empty() {
                    painter.text(
                        egui::pos2(hover_pos.x + 10.0, hover_pos.y - 20.0),
                        egui::Align2::LEFT_TOP,
                        state.spectrum_hover_text.clone(),
                        egui::FontId::proportional(12.0),
                        egui::Color32::WHITE,
                    );
                }
            }
        }
        
        // Axis labels
        painter.text(
            egui::pos2(rect.left() + 5.0, rect.bottom() - 15.0),
            egui::Align2::LEFT_BOTTOM,
            format!("{:.0} Hz", min_freq),
            font_id.clone(),
            egui::Color32::WHITE,
        );
        painter.text(
            egui::pos2(rect.right() - 5.0, rect.bottom() - 15.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("{:.0} Hz", max_freq),
            font_id.clone(),
            egui::Color32::WHITE,
        );
        painter.text(
            egui::pos2(rect.left() + 5.0, rect.top() + 5.0),
            egui::Align2::LEFT_TOP,
            format!("{:.2e}", max_imp),
            font_id.clone(),
            egui::Color32::WHITE,
        );
        
        // Legend
        if has_no_toneholes {
            let legend_x = rect.right() - 120.0;
            let legend_y = rect.top() + 5.0;
            painter.line_segment(
                [egui::pos2(legend_x, legend_y), egui::pos2(legend_x + 20.0, legend_y)],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 200, 255)),
            );
            painter.text(
                egui::pos2(legend_x + 25.0, legend_y),
                egui::Align2::LEFT_CENTER,
                "With holes",
                egui::FontId::proportional(10.0),
                egui::Color32::WHITE,
            );
            painter.line_segment(
                [egui::pos2(legend_x, legend_y + 15.0), egui::pos2(legend_x + 20.0, legend_y + 15.0)],
                egui::Stroke::new(1.5, egui::Color32::from_rgb(150, 150, 150)),
            );
            painter.text(
                egui::pos2(legend_x + 25.0, legend_y + 15.0),
                egui::Align2::LEFT_CENTER,
                "No holes",
                egui::FontId::proportional(10.0),
                egui::Color32::WHITE,
            );
        }
    });
}

/// Find nearest point index to a position
fn find_nearest_point(points: &[egui::Pos2], pos: egui::Pos2) -> Option<usize> {
    points.iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = (a.x - pos.x).abs();
            let db = (b.x - pos.x).abs();
            da.partial_cmp(&db).unwrap()
        })
        .map(|(i, _)| i)
}

/// Draw loss curve over generations
fn draw_loss_curve(ui: &mut egui::Ui, state: &CadsdState) {
    if state.loss_history.len() < 2 {
        return;
    }
    
    let max_gen = state.loss_history.last().map(|(g, _)| *g).unwrap_or(1) as f32;
    let losses: Vec<f64> = state.loss_history.iter().map(|(_, l)| *l).collect();
    let max_loss = losses.iter().cloned().fold(f64::NEG_INFINITY, f64::max).max(0.0) as f32;
    let min_loss = losses.iter().cloned().fold(f64::INFINITY, f64::min) as f32;
    let loss_range = (max_loss - min_loss).max(0.001);
    
    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
        let plot_size = egui::vec2(ui.available_width(), 150.0);
        let (response, painter) = ui.allocate_painter(plot_size, egui::Sense::hover());
        let rect = response.rect;
        
        painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::StrokeKind::Middle);
        
        let points: Vec<egui::Pos2> = state.loss_history.iter()
            .map(|(gen, loss)| {
                let x = rect.left() + (*gen as f32 / max_gen.max(1.0)) * rect.width();
                let y = rect.bottom() - ((loss - min_loss as f64) as f32 / loss_range) * rect.height();
                egui::pos2(x, y)
            })
            .collect();
        
        if points.len() > 1 {
            painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 100, 100))));
        }
        
        painter.text(
            egui::pos2(rect.left() + 5.0, rect.bottom() - 10.0),
            egui::Align2::LEFT_BOTTOM,
            format!("Gen 0"),
            egui::FontId::proportional(10.0),
            egui::Color32::GRAY,
        );
        painter.text(
            egui::pos2(rect.right() - 5.0, rect.bottom() - 10.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("Gen {}", max_gen as i32),
            egui::FontId::proportional(10.0),
            egui::Color32::GRAY,
        );
        painter.text(
            egui::pos2(rect.left() + 5.0, rect.top() + 5.0),
            egui::Align2::LEFT_TOP,
            format!("Loss: {:.4e}", max_loss),
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
    });
}

/// Draw loss curve over generations
/// Compare plot overlay - shows all three strategies
fn show_compare_plot(ui: &mut egui::Ui, state: &mut CadsdState) {
    if let Some(data) = &state.compare_data {
        let max_imp = data.tlm_impedances.iter()
            .chain(data.wg_impedances.iter())
            .chain(data.ci_impedances.iter())
            .cloned()
            .fold(0.0f64, f64::max)
            .max(1.0) as f32;
        let min_freq = data.frequencies.first().copied().unwrap_or(0.0) as f32;
        let max_freq = data.frequencies.last().copied().unwrap_or(1.0) as f32;
        let freq_range = (max_freq - min_freq).max(1.0);
        
        ui.label("Strategy Comparison (TLM = cyan, Waveguide = green, Complex Impedance = orange)");
        
        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            let plot_size = egui::vec2(ui.available_width(), 300.0);
            let (response, painter) = ui.allocate_painter(plot_size, egui::Sense::hover());
            let rect = response.rect;
            
            painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::StrokeKind::Middle);
            
            let to_screen = |f: f64, z: f64| -> egui::Pos2 {
                let x = rect.left() + ((f as f32 - min_freq) / freq_range) * rect.width();
                let y = rect.bottom() - (z as f32 / max_imp) * rect.height();
                egui::pos2(x, y)
            };
            
            let tlm_pts: Vec<egui::Pos2> = data.frequencies.iter().zip(data.tlm_impedances.iter())
                .map(|(&f, &z)| to_screen(f, z)).collect();
            let wg_pts: Vec<egui::Pos2> = data.frequencies.iter().zip(data.wg_impedances.iter())
                .map(|(&f, &z)| to_screen(f, z)).collect();
            let ci_pts: Vec<egui::Pos2> = data.frequencies.iter().zip(data.ci_impedances.iter())
                .map(|(&f, &z)| to_screen(f, z)).collect();
            
            if tlm_pts.len() > 1 {
                painter.add(egui::Shape::line(tlm_pts, egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 200, 255))));
            }
            if wg_pts.len() > 1 {
                painter.add(egui::Shape::line(wg_pts, egui::Stroke::new(1.5, egui::Color32::GREEN)));
            }
            if ci_pts.len() > 1 {
                painter.add(egui::Shape::line(ci_pts, egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 165, 0))));
            }
        });
    }
}

/// Optimizer Panel - evolutionary optimization controls
fn show_optimizer_panel(ui: &mut egui::Ui, state: &mut CadsdState, channels: ResMut<OptimizerChannels>) {
    ui.heading("🧬 Evolutionary Optimizer");
    ui.separator();
    
    ui.label("Population Parameters:");
    ui.add(egui::Slider::new(&mut state.population_size, 10..=200).text("Population Size"));
    ui.add(egui::Slider::new(&mut state.num_generations, 1..=500).text("Generations"));
    ui.add(egui::Slider::new(&mut state.mutation_rate, 0.0..=1.0).text("Mutation Rate"));
    ui.add(egui::Slider::new(&mut state.crossover_rate, 0.0..=1.0).text("Crossover Rate"));
    ui.add(egui::Slider::new(&mut state.elite_size, 1..=20).text("Elite Size"));
    
    ui.separator();
    ui.label("Tonehole Parameters:");
    ui.add(egui::Slider::new(&mut state.optimizer_n_toneholes, 0..=5).text("Number of Toneholes"));
    
    ui.separator();
    ui.heading("Neural Surrogate");
    ui.checkbox(&mut state.use_surrogate_loss, "Use surrogate loss");
    if state.use_surrogate_loss {
    if ui.button("Train Surrogate on Current Geometry").clicked() {
        let geo = current_geo(state);
        let segments = crate::sim::create_segments_from_geo(&geo.geo);
        let constants = crate::sim::AcousticConstants::for_conditions(
            state.temperature as f64,
            state.pressure_pa,
            state.relative_humidity,
        );
        let freqs: Vec<f64> = (20..=2000).step_by(20).map(|x| x as f64).collect();
        
        let mut surrogate = crate::prime_conv::SurrogateLossFunction::new(
            20, 100, 1, &[32, 32], 50,
        );
        
        let simulator_fn = move |genome: &[f64]| -> Vec<f64> {
            let mut geo_points = Vec::new();
            let mut x_acc = 0.0;
            for chunk in genome.chunks(3) {
                let l = chunk.get(0).copied().unwrap_or(0.1).max(0.01);
                let d0 = chunk.get(1).copied().unwrap_or(0.03).max(0.005);
                let d1 = chunk.get(2).copied().unwrap_or(0.05).max(0.005);
                x_acc += l * 1000.0;
                geo_points.push([x_acc, d1 * 1000.0]);
            }
            
            if geo_points.len() >= 2 {
                let geo = Geo { geo: geo_points };
                let mut sim = crate::sim::DidgeridooSimulator::from_geo(&geo.geo);
                sim.acoustic_constants = constants.clone();
                let spectrum = sim.impedance(&freqs);
                spectrum.iter().map(|c| c.norm()).collect()
            } else {
                vec![0.0; 50]
            }
        };
        
        surrogate.train_from_simulator(simulator_fn, 100, 0.01, 10);
        state.surrogate_model = Some(surrogate);
        state.surrogate_trained = true;
    }
    ui.label(format!("Status: {}", if state.surrogate_trained { "Trained" } else { "Not trained (falling back to TLM)" }));
    }
    
    ui.separator();
    ui.heading("Optimization Mode");
    ui.checkbox(&mut state.use_gradient_optimizer, "Use gradient-based optimizer (Adam + DiffTLM)");
    
    ui.separator();
    ui.label("Loss Function Components:");
    let mut any_changed = false;
    for (name, enabled, weight) in &mut state.loss_component_toggles {
        ui.horizontal(|ui| {
            let prev = *enabled;
            ui.checkbox(enabled, format!("{} (w={:.1})", name, weight));
            if *enabled {
                ui.add(egui::Slider::new(weight, 0.0..=20.0).text(""));
            }
            if prev != *enabled {
                any_changed = true;
            }
        });
    }
    if any_changed {
        log::info!("Loss component toggles updated");
    }
    
    ui.separator();
    ui.label(format!("Mutation Strategy: **{:?}**", state.mutation_strategy));
    
    if let Some(loss) = state.best_loss {
        ui.label(format!("🏆 Best Loss: **{:.6}**", loss));
    }
    
    ui.label(format!("Generation: **{}/{}**", state.current_generation, state.num_generations));
    ui.add(
        egui::ProgressBar::new(state.generation_progress)
            .text(format!(
                "Generation {}/{}",
                state.current_generation,
                state.num_generations
            ))
    );
    
    if !state.loss_history.is_empty() {
        ui.separator();
        ui.label("📉 Loss Curve");
        draw_loss_curve(ui, state);
    }
    
    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("▶️ Run Optimization").clicked() && !state.optimizer_running {
            start_optimization(state, channels);
        }
        if ui.button("⏸️ Pause").clicked() && state.optimizer_running && !state.optimizer_paused {
            state.optimizer_paused = true;
            log::info!("Optimization paused");
        }
        if ui.button("▶️ Resume").clicked() && state.optimizer_running && state.optimizer_paused {
            state.optimizer_paused = false;
            log::info!("Optimization resumed");
        }
        if state.optimizer_running {
            ui.spinner();
        }
    });
    ui.horizontal(|ui| {
        if ui.button("💾 Save Checkpoint").clicked() {
            let cp = OptimizerCheckpoint {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs().to_string())
                    .unwrap_or_else(|_| "unknown".to_string()),
                population_size: state.population_size,
                num_generations: state.num_generations,
                current_generation: state.current_generation,
                mutation_rate: state.mutation_rate as f64,
                crossover_rate: state.crossover_rate as f64,
                elite_size: state.elite_size,
                best_loss: state.best_loss,
                generation_progress: state.generation_progress as f64,
                mutation_strategy: format!("{:?}", state.mutation_strategy),
                simulation_strategy: format!("{:?}", state.simulation_strategy),
                geometry: OptimizerGeoState {
                    length: state.length as f64,
                    top_diameter: state.top_diameter as f64,
                    bottom_diameter: state.bottom_diameter as f64,
                    segments: state.segments,
                },
                loss_component_weights: state.loss_component_toggles.iter()
                    .filter(|(_, enabled, _)| *enabled)
                    .map(|(name, _, weight)| (name.clone(), *weight))
                    .collect(),
                toneholes: state.toneholes.clone(),
            };
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name(&format!("checkpoint_{}.json", cp.timestamp))
                .save_file() {
                if let Err(e) = cp.save_to_file(path.to_str().unwrap_or_default()) {
                    log::error!("Failed to save checkpoint: {}", e);
                } else {
                    log::info!("Checkpoint saved to {}", path.display());
                }
            }
        }
        if ui.button("📂 Load Checkpoint").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file() {
                if let Some(cp) = OptimizerCheckpoint::load_from_file(path.to_str().unwrap_or_default()) {
                    state.population_size = cp.population_size;
                    state.num_generations = cp.num_generations;
                    state.current_generation = cp.current_generation;
                    state.mutation_rate = cp.mutation_rate as f32;
                    state.crossover_rate = cp.crossover_rate as f32;
                    state.elite_size = cp.elite_size;
                    state.best_loss = cp.best_loss;
                    state.generation_progress = cp.generation_progress as f32;
                    state.length = cp.geometry.length as f32;
                    state.top_diameter = cp.geometry.top_diameter as f32;
                    state.bottom_diameter = cp.geometry.bottom_diameter as f32;
                    state.segments = cp.geometry.segments;
                    state.toneholes = cp.toneholes;
                    log::info!("Resumed from checkpoint: {}", path.display());
                } else {
                    log::error!("Failed to load checkpoint from {}", path.display());
                }
            }
        }
        if ui.button("📤 Export Best Genome").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name("best_genome.json")
                .save_file() {
                let genome_data = serde_json::json!({
                    "geometry": {
                        "length": state.length,
                        "top_diameter": state.top_diameter,
                        "bottom_diameter": state.bottom_diameter,
                        "segments": state.segments,
                    },
                    "best_loss": state.best_loss,
                    "strategy": format!("{:?}", state.simulation_strategy),
                    "fundamental_freq": state.fundamental_freq,
                });
                let json_str = serde_json::to_string_pretty(&genome_data).unwrap_or_default();
                if let Err(e) = std::fs::write(&path, json_str) {
                    log::error!("Failed to export genome: {}", e);
                } else {
                    log::info!("Best genome exported to {}", path.display());
                }
            }
        }
    });
}

/// Geometry Panel - parametric geometry controls
fn show_geometry_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("📐 Geometry Parameters");
    ui.separator();
    
    // Undo/Redo controls
    ui.horizontal(|ui| {
        let can_undo = state.geo_history_index > 0;
        let can_redo = state.geo_history_index < state.geo_history.len();
        if ui.add_enabled(can_undo, egui::Button::new("↩️ Undo")).clicked() {
            if state.geo_history_index > 0 {
                state.geo_history_index -= 1;
                if let Some(entry) = state.geo_history.get(state.geo_history_index) {
                    let geo = Geo::new(entry.geo_points.clone());
                    state.length = geo.length() as f32;
                    state.top_diameter = geo.geo.first().map(|p| p[1] as f32).unwrap_or(32.0);
                    state.bottom_diameter = geo.geo.last().map(|p| p[1] as f32).unwrap_or(65.0);
                    state.segments = geo.geo.len().max(5);
                }
            }
        }
        if ui.add_enabled(can_redo, egui::Button::new("↪️ Redo")).clicked() {
            if state.geo_history_index < state.geo_history.len() {
                if let Some(entry) = state.geo_history.get(state.geo_history_index) {
                    let geo = Geo::new(entry.geo_points.clone());
                    state.length = geo.length() as f32;
                    state.top_diameter = geo.geo.first().map(|p| p[1] as f32).unwrap_or(32.0);
                    state.bottom_diameter = geo.geo.last().map(|p| p[1] as f32).unwrap_or(65.0);
                    state.segments = geo.geo.len().max(5);
                }
                state.geo_history_index += 1;
            }
        }
        ui.label(format!("History: {}/{}", state.geo_history_index, state.geo_history.len()));
    });
    ui.separator();
    
    // Geometry sliders
    let prev_length = state.length;
    let prev_top = state.top_diameter;
    let prev_bottom = state.bottom_diameter;
    let prev_segments = state.segments;
    
    ui.add(egui::Slider::new(&mut state.length, 500.0..=3000.0).text("Length (mm)"));
    ui.add(egui::Slider::new(&mut state.top_diameter, 10.0..=100.0).text("Top Diameter (mm)"));
    ui.add(egui::Slider::new(&mut state.bottom_diameter, 10.0..=200.0).text("Bottom Diameter (mm)"));
    ui.add(egui::Slider::new(&mut state.segments, 5..=100).text("Segments"));
    
    // Boundary checks: ensure geometry remains physically plausible
    let geo_valid = state.length > 0.0 && 
                    state.top_diameter > 0.0 && 
                    state.bottom_diameter > 0.0 &&
                    state.segments >= 2; // Need at least 2 segments for a valid geometry
    
    if !geo_valid {
        ui.colored_label(egui::Color32::RED, "⚠️ Invalid geometry parameters");
    }
    
    if prev_length != state.length || prev_top != state.top_diameter || 
       prev_bottom != state.bottom_diameter || prev_segments != state.segments {
        if geo_valid {
            push_geo_history(state, "param_edit");
        }
    }
    
    ui.separator();
    
    let geo = current_geo(state);
    
    ui.label(format!("📏 Volume: **{:.2} mm³**", geo.compute_volume()));
    ui.label(format!("📏 Length: **{:.2} mm**", geo.length()));
    ui.label(format!("📏 Max Diameter: **{:.2} mm**", geo.get_max_d()));
    ui.label(format!("📏 Segments: **{}**", geo.geo.len()));
    
    // Simple bore profile preview
    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
        let plot_size = egui::vec2(ui.available_width(), 150.0);
        let (response, painter) = ui.allocate_painter(plot_size, egui::Sense::click_and_drag());
        let rect = response.rect;
        
        painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::StrokeKind::Middle);
        
        // Draw bore profile
        let max_d = geo.get_max_d().max(1.0);
        let points: Vec<egui::Pos2> = geo.geo.iter().map(|&[x, d]| {
            let px = rect.left() + (x as f32 / state.length.max(1.0)) * rect.width();
            let py = rect.center().y - (d as f32 / max_d as f32) * rect.height() / 2.0;
            egui::pos2(px, py)
        }).collect();
        
        if points.len() > 1 {
            painter.add(egui::Shape::line(points.clone(), egui::Stroke::new(2.0, egui::Color32::GREEN)));
            
            // Mirror for bottom half
            let bottom_points: Vec<egui::Pos2> = geo.geo.iter().map(|&[x, d]| {
                let px = rect.left() + (x as f32 / state.length.max(1.0)) * rect.width();
                let py = rect.center().y + (d as f32 / max_d as f32) * rect.height() / 2.0;
                egui::pos2(px, py)
            }).collect();
            painter.add(egui::Shape::line(bottom_points, egui::Stroke::new(2.0, egui::Color32::GREEN)));
        }
        
        // Draw tonehole markers with drag support
        for (i, th) in state.toneholes.iter().enumerate() {
            let px = rect.left() + (th.x as f32 / state.length.max(1.0)) * rect.width();
            let top_y = rect.center().y - (th.diameter as f32 / max_d as f32) * rect.height() / 2.0;
            let bottom_y = rect.center().y + (th.diameter as f32 / max_d as f32) * rect.height() / 2.0;
            let color = if th.is_open { egui::Color32::RED } else { egui::Color32::GRAY };
            let highlight = state.drag_tonehole_index == Some(i);
            let radius = if highlight { 5.0 } else { 3.0 };
            painter.circle_filled(egui::pos2(px, top_y), radius, color);
            painter.circle_filled(egui::pos2(px, bottom_y), radius, color);
            painter.line_segment(
                [egui::pos2(px, top_y), egui::pos2(px, bottom_y)],
                egui::Stroke::new(if highlight { 2.0 } else { 1.0 }, color),
            );
        }
        
        // Handle tonehole dragging
        if response.clicked() || response.dragged() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let mouse_x = mouse_pos.x;
                let mut closest_idx = None;
                let mut closest_dist = f32::INFINITY;
                
                for (i, th) in state.toneholes.iter().enumerate() {
                    let th_px = rect.left() + (th.x as f32 / state.length.max(1.0)) * rect.width();
                    let dist = (mouse_x - th_px).abs();
                    if dist < 10.0 && dist < closest_dist {
                        closest_dist = dist;
                        closest_idx = Some(i);
                    }
                }
                
                if response.clicked() {
                    state.drag_tonehole_index = closest_idx;
                } else if response.dragged() {
                    if let Some(idx) = state.drag_tonehole_index {
                        if idx < state.toneholes.len() {
                            let new_x = ((mouse_x - rect.left()) / rect.width() * state.length).max(0.0).min(state.length);
                            state.toneholes[idx].x = new_x as f64;
                        }
                    }
                }
            }
        }
        
        if response.clicked() && state.drag_tonehole_index.is_some() {
            let mouse_pos = response.interact_pointer_pos();
            if let Some(mouse_pos) = mouse_pos {
                let mouse_x = mouse_pos.x;
                let mut near_any = false;
                for th in &state.toneholes {
                    let th_px = rect.left() + (th.x as f32 / state.length.max(1.0)) * rect.width();
                    if (mouse_x - th_px).abs() < 10.0 {
                        near_any = true;
                        break;
                    }
                }
                if !near_any {
                    state.drag_tonehole_index = None;
                }
            } else {
                state.drag_tonehole_index = None;
            }
        }
        
        // Scroll wheel to adjust tonehole diameter
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 {
            if let Some(mouse_pos) = response.hover_pos() {
                let mouse_x = mouse_pos.x;
                for th in state.toneholes.iter_mut() {
                    let th_px = rect.left() + (th.x as f32 / state.length.max(1.0)) * rect.width();
                    if (mouse_x - th_px).abs() < 10.0 {
                        let delta = -scroll_delta * 0.5;
                        th.diameter = (th.diameter + delta as f64).clamp(2.0, 30.0);
                        break;
                    }
                }
            }
        }
    });
    
    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("➕ Add Bubble").clicked() {
            state.show_add_bubble_dialog = true;
        }
        if ui.button("↔️ Stretch Geometry").clicked() {
            state.show_stretch_dialog = true;
        }
    });
    ui.horizontal(|ui| {
        if ui.button("📤 Export Geometry JSON").clicked() {
            let geo_json = serde_json::to_string_pretty(&geo.geo).unwrap_or_default();
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name("geometry.json")
                .save_file() {
                if let Err(e) = std::fs::write(&path, &geo_json) {
                    log::error!("Failed to export geometry: {}", e);
                } else {
                    log::info!("Geometry exported to {}", path.display());
                }
            }
        }
        if ui.button("📂 Import Geometry JSON").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(geo_points) = serde_json::from_str::<Vec<[f64; 2]>>(&content) {
                        push_geo_history(state, "import");
                        let imported_geo = Geo::new(geo_points);
                        state.length = imported_geo.length() as f32;
                        state.top_diameter = imported_geo.geo.first().map(|p| p[1] as f32).unwrap_or(32.0);
                        state.bottom_diameter = imported_geo.geo.last().map(|p| p[1] as f32).unwrap_or(65.0);
                        state.segments = imported_geo.geo.len().max(5);
                        log::info!("Geometry imported from {}", path.display());
                    } else {
                        log::error!("Invalid geometry JSON format in {}", path.display());
                    }
                }
            }
        }
    });

    ui.separator();
    ui.heading("Toneholes");
    
    let toneholes_to_remove: Vec<(usize, Option<bool>)> = state.toneholes.iter().enumerate().filter_map(|(i, th)| {
        let mut remove = false;
        let mut duplicate = false;
        ui.horizontal(|ui| {
            ui.label(format!("#{}: x={:.0}mm d={:.1}mm depth={:.1}mm {}", 
                i + 1, th.x, th.diameter, th.depth, 
                if th.is_open { "open" } else { "closed" }));
            if ui.button("Duplicate").clicked() {
                duplicate = true;
            }
            if ui.button("Delete").clicked() {
                remove = true;
            }
        });
        if remove { Some((i, None)) } else if duplicate { Some((i, Some(true))) } else { None }
    }).collect();
    
    for (i, action) in toneholes_to_remove.iter().rev() {
        if let Some(true) = action {
            let mut new_th = state.toneholes[*i].clone();
            new_th.x = (new_th.x + 50.0).min(state.length as f64);
            state.toneholes.push(new_th);
        } else {
            if state.drag_tonehole_index == Some(*i) {
                state.drag_tonehole_index = None;
            }
            state.toneholes.remove(*i);
            if state.selected_tonehole_index == Some(*i) {
                state.selected_tonehole_index = None;
            } else if let Some(sel) = state.selected_tonehole_index {
                if sel > *i {
                    state.selected_tonehole_index = Some(sel - 1);
                }
            }
        }
    }
    
    for (i, th) in state.toneholes.iter().enumerate() {
        ui.horizontal(|ui| {
            let is_selected = state.selected_tonehole_index == Some(i);
            if ui.selectable_label(is_selected, format!("#{}: x={:.0}mm d={:.1}mm depth={:.1}mm {}", 
                i + 1, th.x, th.diameter, th.depth, 
                if th.is_open { "open" } else { "closed" })).clicked() {
                state.selected_tonehole_index = Some(i);
            }
        });
    }
    
    ui.label("Tonehole Preset:");
    egui::ComboBox::from_label("Preset")
        .selected_text(state.selected_tonehole_preset.name())
        .show_ui(ui, |ui| {
            for preset in crate::tonehole::ToneholePreset::all() {
                ui.selectable_value(&mut state.selected_tonehole_preset, *preset, preset.name());
            }
        });
    
    if ui.button("Apply Preset").clicked() {
        state.toneholes = state.selected_tonehole_preset.generate(state.length as f64);
        state.selected_tonehole_index = None;
    }
    
    if ui.button("Add Tonehole").clicked() {
        state.toneholes.push(Tonehole::new(
            state.length as f64 * 0.5,
            10.0,
            5.0,
            true,
        ));
    }
    
    if let Some(idx) = state.selected_tonehole_index {
        if idx < state.toneholes.len() {
            ui.separator();
            ui.heading("Edit Tonehole");
            let cloned_for_dup = state.toneholes[idx].clone();
            {
                let th = &mut state.toneholes[idx];
                ui.add(egui::Slider::new(&mut th.diameter, 2.0..=30.0).text("Diameter (mm)"));
                ui.add(egui::Slider::new(&mut th.depth, 1.0..=20.0).text("Depth (mm)"));
                ui.add(egui::Slider::new(&mut th.x, 0.0..=state.length as f64).text("Position (mm)"));
                ui.add(egui::Slider::new(&mut th.coverage, 0.0..=1.0).text("Coverage"));
                ui.checkbox(&mut th.is_open, "Open");
            }
            let is_open = state.toneholes[idx].is_open;
            if ui.button("Duplicate Tonehole").clicked() {
                let mut new_th = cloned_for_dup;
                new_th.x = (new_th.x + 50.0).min(state.length as f64);
                state.toneholes.push(new_th);
            }
            if ui.button("Compute Impedance Spectrum").clicked() {
                let constants = crate::sim::AcousticConstants::for_conditions(
                    state.temperature as f64,
                    state.pressure_pa,
                    state.relative_humidity,
                );
                let freqs: Vec<f64> = (20..=2000).step_by(20).map(|x| x as f64).collect();
                let spectrum: Vec<f64> = freqs.iter()
                    .map(|&f| if is_open { state.toneholes[idx].open_impedance(f, &constants).norm() } else { state.toneholes[idx].closed_impedance(f, &constants).norm() })
                    .collect();
                state.tonehole_impedance_freqs = freqs;
                state.tonehole_impedances = spectrum;
            }
            if !state.tonehole_impedances.is_empty() {
                ui.separator();
                ui.label("Tonehole Impedance Spectrum:");
                egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    let plot_size = egui::vec2(ui.available_width(), 150.0);
                    let (response, painter) = ui.allocate_painter(plot_size, egui::Sense::hover());
                    let rect = response.rect;
                    painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::StrokeKind::Middle);
                    
                    let min_freq = state.tonehole_impedance_freqs.first().copied().unwrap_or(0.0) as f32;
                    let max_freq = state.tonehole_impedance_freqs.last().copied().unwrap_or(1.0) as f32;
                    let max_imp = state.tonehole_impedances.iter().cloned().fold(0.0f64, f64::max).max(1.0) as f32;
                    let freq_range = (max_freq - min_freq).max(1.0);
                    
                    let points: Vec<egui::Pos2> = state.tonehole_impedance_freqs.iter().zip(state.tonehole_impedances.iter())
                        .map(|(&f, &z)| {
                            let x = rect.left() + ((f as f32 - min_freq) / freq_range) * rect.width();
                            let y = rect.bottom() - (z as f32 / max_imp) * rect.height();
                            egui::pos2(x, y)
                        })
                        .collect();
                    
                    if points.len() > 1 {
                        let color = if is_open { egui::Color32::RED } else { egui::Color32::GRAY };
                        painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, color)));
                    }
                    
                    painter.text(
                        egui::pos2(rect.left() + 5.0, rect.bottom() - 15.0),
                        egui::Align2::LEFT_BOTTOM,
                        format!("{:.0} Hz", min_freq),
                        egui::FontId::proportional(10.0),
                        egui::Color32::WHITE,
                    );
                    painter.text(
                        egui::pos2(rect.right() - 5.0, rect.bottom() - 15.0),
                        egui::Align2::RIGHT_BOTTOM,
                        format!("{:.0} Hz", max_freq),
                        egui::FontId::proportional(10.0),
                        egui::Color32::WHITE,
                    );
                });
                if !state.tonehole_impedances.is_empty() {
                    let (resonance_freq, resonance_type) = if is_open {
                        let (min_idx, _) = state.tonehole_impedances.iter()
                            .enumerate()
                            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                            .unwrap();
                        (state.tonehole_impedance_freqs.get(min_idx).copied().unwrap_or(0.0), "min impedance")
                    } else {
                        let (max_idx, _) = state.tonehole_impedances.iter()
                            .enumerate()
                            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                            .unwrap();
                        (state.tonehole_impedance_freqs.get(max_idx).copied().unwrap_or(0.0), "max impedance")
                    };
                    ui.label(format!("Resonance: {:.0} Hz ({})", resonance_freq, resonance_type));
                }
                if ui.button("Export Tonehole Impedance CSV").clicked() {
                    let mut csv = String::from("frequency_hz,impedance_magnitude\n");
                    for (f, z) in state.tonehole_impedance_freqs.iter().zip(state.tonehole_impedances.iter()) {
                        csv.push_str(&format!("{},{}\n", f, z));
                    }
                    let default_name = format!("tonehole_impedance_{}.csv", 
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0));
                    if let Some(path) = FileDialog::new()
                        .add_filter("CSV", &["csv"])
                        .set_file_name(&default_name)
                        .save_file() {
                        if let Err(e) = std::fs::write(&path, &csv) {
                            log::error!("Failed to export tonehole CSV: {}", e);
                        } else {
                            log::info!("Tonehole impedance CSV exported to {}", path.display());
                        }
                    }
                }
            }
        }
    }
}

/// Settings Panel - application configuration
fn show_settings_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("⚙️ Settings");
    ui.separator();
    
    ui.label("Theme:");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.theme, "dark".to_string(), "🌙 Dark");
        ui.selectable_value(&mut state.theme, "light".to_string(), "☀️ Light");
    });
    
    ui.separator();
    ui.label("Logging Verbosity:");
    ui.horizontal(|ui| {
        ui.radio_value(&mut state.log_verbosity, 0u8, "Error");
        ui.radio_value(&mut state.log_verbosity, 1u8, "Warn");
        ui.radio_value(&mut state.log_verbosity, 2u8, "Info");
        ui.radio_value(&mut state.log_verbosity, 3u8, "Debug");
        ui.radio_value(&mut state.log_verbosity, 4u8, "Trace");
    });
    
    ui.separator();
    ui.label("Acoustic Environment:");
    ui.add(egui::Slider::new(&mut state.temperature, -20.0..=50.0).text("Temperature (°C)"));
    ui.add(egui::Slider::new(&mut state.pressure_pa, 50000.0..=200000.0).text("Pressure (Pa)"));
    ui.add(egui::Slider::new(&mut state.relative_humidity, 0.0..=1.0).text("Relative Humidity"));
    
    ui.separator();
    ui.label("Default Simulation Strategy:");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.default_strategy, "Tlm".to_string(), "TLM");
        ui.selectable_value(&mut state.default_strategy, "Waveguide".to_string(), "Waveguide");
        ui.selectable_value(&mut state.default_strategy, "ComplexImpedance".to_string(), "Complex Impedance");
    });
    
    ui.label("Default Mutation Strategy:");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.default_mutation, "Gaussian".to_string(), "Gaussian");
        ui.selectable_value(&mut state.default_mutation, "PrimeSequence".to_string(), "Prime Sequence");
    });
    
    ui.separator();
    ui.label("Conservation Settings:");
    ui.add(egui::Slider::new(&mut state.budget_ops, 1000.0..=1_000_000.0).text("Max Operations"));
    
    ui.separator();
    ui.label("Display:");
    ui.checkbox(&mut state.use_log_grid, "Logarithmic frequency grid");
    
    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("💾 Save Configuration").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name("settings.json")
                .save_file() {
                let settings = AppSettings {
                    temperature: state.temperature as f32,
                    wall_thickness: 4.0,
                    show_3d: true,
                    mesh_rotation_enabled: false,
                    mesh_rotation_speed: 30.0,
                    color_scheme: "wood".to_string(),
                    theme: state.theme.clone(),
                    log_verbosity: state.log_verbosity,
                    default_strategy: format!("{:?}", state.simulation_strategy),
                    default_mutation: format!("{:?}", state.mutation_strategy),
                    pressure_pa: state.pressure_pa,
                    relative_humidity: state.relative_humidity,
                };
                if let Err(e) = settings.save_to_file(path.to_str().unwrap_or_default()) {
                    log::error!("Failed to save config: {}", e);
                } else {
                    log::info!("Configuration saved to {}", path.display());
                }
            }
        }
        if ui.button("📂 Load Configuration").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file() {
                let settings = AppSettings::load_from_file(path.to_str().unwrap_or_default());
                state.theme = settings.theme;
                state.log_verbosity = settings.log_verbosity;
                state.temperature = settings.temperature;
                state.default_strategy = settings.default_strategy;
                state.default_mutation = settings.default_mutation;
                log::info!("Configuration loaded from {}", path.display());
            }
        }
        if ui.button("🔄 Reset to Defaults").clicked() {
            *state = CadsdState::default();
        }
    });
}

/// Compute impedance spectrum for the current geometry
fn compute_spectrum(state: &mut CadsdState) {
    let geo = current_geo(state);
    
    let freqs: Vec<f64> = if state.use_log_grid {
        let cents_step = state.freq_cents_step as f64;
        let step_ratio = 2.0f64.powf(cents_step / 1200.0);
        let mut f = state.freq_min as f64;
        let mut freqs = Vec::new();
        while f <= state.freq_max as f64 {
            freqs.push(f);
            f *= step_ratio;
        }
        freqs
    } else {
        // Linear grid
        let step = (state.freq_max - state.freq_min) / state.freq_points.max(1) as f32;
        (0..=state.freq_points)
            .map(|i| state.freq_min as f64 + i as f64 * step as f64)
            .collect()
    };
    
    state.frequencies = freqs.clone();
    
    let mut simulator = DidgeridooSimulator::with_strategy(
        &geo.geo,
        state.simulation_strategy
    );
    simulator.acoustic_constants = crate::sim::AcousticConstants::for_conditions(
        state.temperature as f64,
        state.pressure_pa,
        state.relative_humidity,
    );
    simulator.toneholes = state.toneholes.clone();
    
    let spectrum = simulator.impedance(&freqs);
    state.impedances = spectrum.iter().map(|c| c.norm()).collect();
    state.phases = spectrum.iter().map(|c| c.arg().to_degrees()).collect();
    
    let mut simulator_no_th = DidgeridooSimulator::with_strategy(
        &geo.geo,
        state.simulation_strategy
    );
    simulator_no_th.acoustic_constants = crate::sim::AcousticConstants::for_conditions(
        state.temperature as f64,
        state.pressure_pa,
        state.relative_humidity,
    );
    let spectrum_no_th = simulator_no_th.impedance(&freqs);
    state.impedances_no_toneholes = spectrum_no_th.iter().map(|c| c.norm()).collect();
    state.phases_no_toneholes = spectrum_no_th.iter().map(|c| c.arg().to_degrees()).collect();
    
    // Find fundamental frequency from resonance peaks
    let resonances = simulator.find_resonance_peaks();
    if let Some(first_peak) = resonances.first() {
        state.fundamental_freq = Some(first_peak.frequency);
    }
    
    state.generation_progress = 1.0;
}

/// Find resonance peaks in the current spectrum
fn find_peaks(state: &mut CadsdState) {
    if !state.frequencies.is_empty() && !state.impedances.is_empty() {
        // Convert f64 magnitudes to Complex<f64> for peak finding
        let complex_spectrum: Vec<Complex<f64>> = state.impedances.iter()
            .map(|&mag| Complex::new(mag, 0.0))
            .collect();
        
        let peaks = crate::sim::find_peaks(&state.frequencies, &complex_spectrum);
        if let Some(peak) = peaks.first() {
            state.fundamental_freq = Some(peak.1);
        }
    }
}

/// Run comparison across all three simulation strategies
fn run_comparison_simulation(state: &mut CadsdState) {
    let geo = current_geo(state);
    
    let freqs: Vec<f64> = (20..=2000).step_by(10).map(|x| x as f64).collect();
    
    // Run all three strategies
    let mut tlm_sim = DidgeridooSimulator::from_geo(&geo.geo);
    let wg_sim = DidgeridooSimulator::with_strategy(&geo.geo, SimulationStrategy::Waveguide);
    let ci_sim = DidgeridooSimulator::with_strategy(&geo.geo, SimulationStrategy::ComplexImpedance);
    tlm_sim.acoustic_constants = crate::sim::AcousticConstants::for_conditions(
        state.temperature as f64,
        state.pressure_pa,
        state.relative_humidity,
    );
    tlm_sim.toneholes = state.toneholes.clone();
    
    let tlm_spec = tlm_sim.impedance(&freqs);
    let wg_spec = wg_sim.impedance(&freqs);
    let ci_spec = ci_sim.impedance(&freqs);
    
    state.compare_data = Some(CompareData {
        frequencies: freqs.clone(),
        tlm_impedances: tlm_spec.iter().map(|c| c.norm()).collect(),
        wg_impedances: wg_spec.iter().map(|c| c.norm()).collect(),
        ci_impedances: ci_spec.iter().map(|c| c.norm()).collect(),
    });
    
    // Store TLM results as default
    state.impedances = tlm_spec.iter().map(|c| c.norm()).collect();
    state.frequencies = freqs;
    
    log::info!("Comparison complete - TLM: {} pts, WG: {} pts, CI: {} pts",
        tlm_spec.len(), wg_spec.len(), ci_spec.len());
}

/// Export spectrum data as CSV
fn export_spectrum_csv(state: &CadsdState) {
    if state.frequencies.is_empty() || state.impedances.is_empty() {
        log::warn!("No spectrum data to export");
        return;
    }
    
    let mut csv = String::from("frequency_hz,impedance_magnitude\n");
    for (f, z) in state.frequencies.iter().zip(state.impedances.iter()) {
        csv.push_str(&format!("{},{}\n", f, z));
    }
    
    let default_name = format!("spectrum_export_{}.csv", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0));
    
    if let Some(path) = FileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_file_name(&default_name)
        .save_file() {
        if let Err(e) = std::fs::write(&path, &csv) {
            log::error!("Failed to export CSV: {}", e);
        } else {
            log::info!("Spectrum CSV exported to {} ({} lines)", path.display(), csv.lines().count());
        }
    }
}

fn validate_tlm(state: &mut CadsdState) {
    let geo = current_geo(state);
    let constants = crate::sim::AcousticConstants::for_conditions(
        state.temperature as f64,
        state.pressure_pa,
        state.relative_humidity,
    );
    let freqs: Vec<f64> = (20..=2000).step_by(20).map(|x| x as f64).collect();
    state.validation_report = crate::validation::generate_validation_report(&geo, &freqs, &constants);
    log::info!("Validation complete");
}
