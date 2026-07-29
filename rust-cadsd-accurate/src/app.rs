//! CADSD GUI - Full-featured acoustic simulation interface

use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;
use bevy_egui::{EguiContexts, EguiPlugin};
use egui_plot::{Plot, Line, PlotPoints};

// Import egui for UI components
use bevy_egui::egui;

use crate::geo::Geo;
use crate::integration::{AcousticSimulator, EvolutionaryOptimizer};
use crate::conv::{note_name, freq_to_note};
use crate::evo::{TargetSound, BoreShapePreference};
use crate::inverse_design::DesignResult;
use crate::persistence::AppSettings;
use std::sync::Arc;

#[derive(Resource)]
#[allow(dead_code)]
pub struct CadsdState {
    // === BASE DIDGERIDOO GEOMETRY ===
    pub length: f32,               // Total length (mm)
    pub top_diameter: f32,         // Mouthpiece end diameter (mm)
    pub bottom_diameter: f32,      // Bell diameter (mm)
    pub segments: usize,           // Resolution of geometry
    pub style_type: String,        // "cone", "cylinder", "exponential", "custom"
    pub bore_curve: f32,           // Curve parameter for non-linear bores
    
    // === MOUTHPIECE MODIFICATION ===
    pub enable_mouthpiece: bool,   // Add mouthpiece to base didgeridoo
    pub mouthpiece_type: String,   // "none", "reed", "embouchure_hole", "fipple", "cup"
    pub mouthpiece_length: f32,    // Additional length from mouthpiece (mm)
    pub mouthpiece_diameter: f32,  // Mouthpiece outer diameter (mm)
    
    // === HOLE MODIFICATIONS ===
    pub enable_holes: bool,        // Enable finger/side holes
    pub hole_count: usize,         // Number of holes
    pub hole_positions: Vec<f32>,  // Position from top (mm)
    pub hole_diameters: Vec<f32>,  // Diameter of each hole (mm)
    
    // === ADVANCED PARAMETERS ===
    pub wall_thickness: f32,       // Wall thickness (mm)
    pub temperature: f32,          // Air temperature (°C) - affects sound speed
    
    // === SIMULATION RESULTS ===
    pub frequencies: Vec<f64>,
    pub impedances: Vec<f64>,
    pub fundamental_freq: Option<f64>,
    pub resonance_notes: Vec<(f64, f64)>,
    pub target_frequency: f64,
    pub tairua_loss_value: f64,
    
    // === VISUALIZATION OPTIONS ===
    pub show_3d: bool,
    pub show_wireframe: bool,
    pub show_cross_section: bool,
    pub mesh_rotation_enabled: bool,
    pub mesh_rotation_speed: f32,
    pub color_scheme: String,      // "wood", "metal", "custom"
    pub active_tab: String,        // "forward", "inverse", "analysis", "export"
    
    // === SIMULATION CONTROL ===
    pub is_simulating: bool,
    pub pending_simulation: bool,
    pub last_error: Option<String>,
    pub simulation_message: String,
    
    // === INVERSE DESIGN (OPTIMIZATION) ===
    pub enable_optimization: bool, // Toggle inverse design mode
    pub optimization_target: String,
    pub optimization_progress: f32,
    pub pending_optimization: bool,
    pub opt_population_size: usize,
    pub opt_generations: usize,
    pub opt_bore_shape: String,     // "any", "cylindrical", "conical", "flared"
    pub opt_min_length: f32,
    pub opt_max_length: f32,
    pub opt_min_bell: f32,
    pub opt_max_bell: f32,
    pub opt_toots_input: String,    // Comma-separated target toots
}

impl Default for CadsdState {
    fn default() -> Self {
        // Start with a simple base didgeridoo (no modifications)
        Self {
            // Base didgeridoo geometry
            length: 1500.0,
            top_diameter: 32.0,
            bottom_diameter: 65.0,
            segments: 30,
            style_type: "cone".to_string(),
            bore_curve: 0.0,
            
            // Mouthpiece (disabled by default - traditional didgeridoo)
            enable_mouthpiece: false,
            mouthpiece_type: "none".to_string(),
            mouthpiece_length: 50.0,
            mouthpiece_diameter: 35.0,
            
            // Holes (disabled by default - traditional didgeridoo)
            enable_holes: false,
            hole_count: 0,
            hole_positions: vec![],
            hole_diameters: vec![],
            
            // Advanced parameters
            wall_thickness: 4.0,
            temperature: 20.0,
            
            // Simulation results
            frequencies: vec![],
            impedances: vec![],
            fundamental_freq: None,
            resonance_notes: vec![],
            target_frequency: 65.41,  // D1
            tairua_loss_value: 0.0,
            
            // Visualization
            show_3d: true,
            show_wireframe: false,
            show_cross_section: false,
            mesh_rotation_enabled: false,
            mesh_rotation_speed: 30.0,
            color_scheme: "wood".to_string(),
            active_tab: "forward".to_string(),  // Default to Forward Design tab
            
            // Simulation control
            is_simulating: false,
            pending_simulation: false,
            last_error: None,
            simulation_message: "Click 'Run Simulation' to analyze acoustics".to_string(),
            
            // Inverse design
            enable_optimization: false,
            optimization_target: "fundamental".to_string(),
            optimization_progress: 0.0,
            pending_optimization: false,
            opt_population_size: 40,
            opt_generations: 30,
            opt_bore_shape: "any".to_string(),
            opt_min_length: 1000.0,
            opt_max_length: 2000.0,
            opt_min_bell: 40.0,
            opt_max_bell: 100.0,
            opt_toots_input: "".to_string(),
        }
    }
}

#[derive(Component)]
struct DidgeMesh;

#[derive(Resource)]
struct PreviousState {
    segments: usize,
    style_type: String,
    length: f32,
    top_diameter: f32,
    bottom_diameter: f32,
    bore_curve: f32,
    show_3d: bool,
}

impl Default for PreviousState {
    fn default() -> Self {
        Self {
            segments: 20,
            style_type: "cone".to_string(),
            length: 1500.0,
            top_diameter: 32.0,
            bottom_diameter: 65.0,
            bore_curve: 0.0,
            show_3d: true,
        }
    }
}

#[derive(Resource, Default)]
struct FrameCounter {
    frame: u32,
}

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
        design: DesignResult,
        frequencies: Vec<f64>,
        impedances: Vec<f64>,
    },
    OptimizationError(String),
}

#[derive(Resource)]
struct BackgroundReceiver(std::sync::Mutex<std::sync::mpsc::Receiver<BackgroundTaskResult>>);

#[derive(Resource, Clone)]
struct BackgroundSender(std::sync::mpsc::Sender<BackgroundTaskResult>);

pub fn run_app() {
    let mut app = App::new();
    
    app.add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "DidgeRust - CADSD".into(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        })
    );
    
    app.add_plugins(EguiPlugin::default());
    app.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)));
    
    // Load app settings from persistent store
    let settings = AppSettings::load_from_file("settings.json");
    let mut initial_state = CadsdState::default();
    initial_state.temperature = settings.temperature;
    initial_state.wall_thickness = settings.wall_thickness;
    initial_state.show_3d = settings.show_3d;
    initial_state.mesh_rotation_enabled = settings.mesh_rotation_enabled;
    initial_state.mesh_rotation_speed = settings.mesh_rotation_speed;
    initial_state.color_scheme = settings.color_scheme;
    
    app.insert_resource(initial_state);
    app.insert_resource(FrameCounter::default());
    app.insert_resource(PreviousState::default());
    
    let (tx, rx) = std::sync::mpsc::channel();
    app.insert_resource(BackgroundSender(tx));
    app.insert_resource(BackgroundReceiver(std::sync::Mutex::new(rx)));
    
    app.add_systems(Startup, (setup_camera, setup_light, spawn_ground));
    app.add_systems(Update, (ui_system, update_mesh, sync_previous_state, poll_background_tasks));
    
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 1.5, 3.0)
            .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
}

fn setup_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0)
            .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
}

fn spawn_ground(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let ground_mesh = meshes.add(Plane3d::default().mesh().size(10.0, 10.0));
    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.25),
            metallic: 0.0,
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn apply_visual_theme(ctx: &egui::Context) {
    crate::ui::apply_visual_theme(ctx);
}

fn ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<CadsdState>,
    mut frame_counter: ResMut<FrameCounter>,
) {
    // Skip first 5 frames to let egui initialize fonts
    frame_counter.frame += 1;
    if frame_counter.frame < 5 {
        return;
    }
    
    // Use catch_unwind to prevent panic from hanging the app
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let ctx = contexts.ctx_mut().unwrap();
        apply_visual_theme(ctx);
        
        // Top panel - Title bar with tab navigation
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎵 DidgeRust");
                ui.vertical(|ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 1.0);
                    ui.small("Wind Instrument CADSD Platform");
                });
                
                ui.separator();
                
                // Tab navigation - each tab connects to real backend features
                ui.selectable_value(&mut state.active_tab, "forward".to_string(), "① Forward Design");
                ui.selectable_value(&mut state.active_tab, "inverse".to_string(), "② Inverse Design");
                ui.selectable_value(&mut state.active_tab, "analysis".to_string(), "③ Analysis");
                ui.selectable_value(&mut state.active_tab, "export".to_string(), "④ Export");
                ui.selectable_value(&mut state.active_tab, "settings".to_string(), "⑤ Settings");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small(format!("v0.1.0"));
                });
            });
        });
        
        // Left panel - Tab-specific controls
        egui::SidePanel::left("controls_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                match state.active_tab.as_str() {
                    "forward" => show_forward_design_panel(ui, &mut state),
                    "inverse" => show_inverse_design_panel(ui, &mut state),
                    "analysis" => show_analysis_panel(ui, &mut state),
                    "export" => show_export_panel(ui, &mut state),
                    "settings" => show_settings_panel(ui, &mut state),
                    _ => {}
                }
            });
        
        // Bottom panel - Impedance Spectrum (only show after simulation)
        if !state.impedances.is_empty() {
            egui::TopBottomPanel::bottom("spectrum_panel")
                .default_height(280.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Impedance Spectrum");
                        if let Some(fund) = state.fundamental_freq {
                            ui.separator();
                            ui.label(format!("Fundamental: {:.2} Hz ({})", fund, note_name(freq_to_note(fund))));
                        }
                    });
                    
                    let points: Vec<[f64; 2]> = state.frequencies.iter()
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
                             
                             // Mark fundamental frequency
                             if let Some(fund) = state.fundamental_freq {
                                 plot_ui.vline(egui_plot::VLine::new("Fundamental", fund)
                                     .color(egui::Color32::RED));
                             }
                        });
                });
        }
    }));
    
    if result.is_err() {
        // Silently skip - egui not ready yet
    }
}

fn poll_background_tasks(
    receiver: Res<BackgroundReceiver>,
    sender: Res<BackgroundSender>,
    mut state: ResMut<CadsdState>,
) {
    // 1. Trigger simulation on background thread
    if state.pending_simulation && !state.is_simulating {
        state.pending_simulation = false;
        state.is_simulating = true;
        state.last_error = None;
        state.simulation_message = "Starting background simulation...".to_string();
        
        let tx = sender.0.clone();
        let geo = create_geometry(&state);
        
        // Generate linear grid from 20 Hz to 2000 Hz with 512 points
        let mut frequencies = Vec::new();
        let step = (2000.0 - 20.0) / 512.0;
        for i in 0..=512 {
            frequencies.push(20.0 + i as f64 * step);
        }
        let target_frequency = state.target_frequency;
        
        std::thread::spawn(move || {
            let sim = crate::integration::DefaultSimulator;
            match sim.simulate(&geo, &frequencies) {
                Ok(impedances) => {
                    let fund = sim.get_fundamental(&geo).ok();
                    let resonance_notes = crate::analysis::get_notes(&frequencies, &impedances);
                    let loss = crate::loss::TairuaLoss::new().with_target_frequency(target_frequency);
                    let tairua_loss = loss.compute_loss(&geo).unwrap_or(0.0);
                    
                    let _ = tx.send(BackgroundTaskResult::Simulation {
                        frequencies,
                        impedances,
                        fundamental: fund,
                        resonance_notes,
                        tairua_loss,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BackgroundTaskResult::SimulationError(e.to_string()));
                }
            }
        });
    }

    // 2. Trigger optimization on background thread
    if state.pending_optimization && !state.is_simulating {
        state.pending_optimization = false;
        state.is_simulating = true;
        state.last_error = None;
        state.simulation_message = "Initializing evolutionary designer...".to_string();
        state.optimization_progress = 0.0;
        
        let tx = sender.0.clone();
        
        // Construct target sound
        let mut target = TargetSound::new(state.target_frequency as f64);
        let bore_shape = match state.opt_bore_shape.as_str() {
            "cylindrical" => BoreShapePreference::Cylindrical,
            "conical" => BoreShapePreference::Conical,
            "flared" => BoreShapePreference::Flared,
            _ => BoreShapePreference::Any,
        };
        target = target.with_bore_shape(bore_shape);
        target = target.with_length_range(state.opt_min_length as f64, state.opt_max_length as f64);
        target = target.with_bell_range(state.opt_min_bell as f64, state.opt_max_bell as f64);
        
        if !state.opt_toots_input.trim().is_empty() {
            for part in state.opt_toots_input.split(',') {
                if let Ok(freq) = part.trim().parse::<f64>() {
                    target = target.with_toot(freq);
                }
            }
        }
        
        let pop_size = state.opt_population_size;
        let gens = state.opt_generations;
        
        std::thread::spawn(move || {
            let tx_progress = tx.clone();
            let progress_cb = Arc::new(move |gen: usize, best_loss: f64| {
                let _ = tx_progress.send(BackgroundTaskResult::OptimizationProgress {
                    generation: gen,
                    total_generations: gens,
                    best_loss,
                });
            });
            
            let opt = crate::integration::DefaultOptimizer;
            match opt.optimize(target, pop_size, gens, Some(progress_cb)) {
                Ok(result) => {
                    // Compute full impedance spectrum on background thread for plotting
                    let sim = crate::integration::DefaultSimulator;
                    let mut frequencies = Vec::new();
                    let step = (2000.0 - 20.0) / 512.0;
                    for i in 0..=512 {
                        frequencies.push(20.0 + i as f64 * step);
                    }
                    let impedances = sim.simulate(&result.geometry, &frequencies).unwrap_or_default();
                    
                    let _ = tx.send(BackgroundTaskResult::OptimizationSuccess {
                        design: result,
                        frequencies,
                        impedances,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BackgroundTaskResult::OptimizationError(e.to_string()));
                }
            }
        });
    }

    // 3. Receive results from channels
    if let Ok(rx) = receiver.0.lock() {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                BackgroundTaskResult::Simulation { frequencies, impedances, fundamental, resonance_notes, tairua_loss } => {
                    state.is_simulating = false;
                    state.frequencies = frequencies;
                    state.impedances = impedances;
                    state.fundamental_freq = fundamental;
                    state.resonance_notes = resonance_notes;
                    state.tairua_loss_value = tairua_loss;
                    state.simulation_message = format!("✓ Complete - Found {} resonances", state.resonance_notes.len());
                }
                BackgroundTaskResult::SimulationError(err) => {
                    state.is_simulating = false;
                    state.last_error = Some(err);
                    state.simulation_message = "Simulation failed".to_string();
                }
                BackgroundTaskResult::OptimizationProgress { generation, total_generations, best_loss } => {
                    state.optimization_progress = (generation + 1) as f32 / total_generations as f32;
                    state.simulation_message = format!("🧬 Evolving... Gen {}/{} (Best Loss: {:.4})", generation + 1, total_generations, best_loss);
                }
                BackgroundTaskResult::OptimizationSuccess { design, frequencies, impedances } => {
                    state.is_simulating = false;
                    state.optimization_progress = 0.0;
                    
                    // Update state geometry to match optimized result
                    let best_geo = design.geometry;
                    state.length = best_geo.length() as f32;
                    state.segments = (best_geo.geo.len() - 1).max(1);
                    if let Some(first_seg) = best_geo.geo.first() {
                        state.top_diameter = first_seg[1] as f32;
                    }
                    if let Some(last_seg) = best_geo.geo.last() {
                        state.bottom_diameter = last_seg[1] as f32;
                    }
                    let is_cyl = (state.top_diameter - state.bottom_diameter).abs() < 0.1;
                    state.style_type = if is_cyl { "cylinder".to_string() } else { "cone".to_string() };
                    state.bore_curve = 0.0;
                    
                    // Update results
                    state.fundamental_freq = Some(design.fundamental_freq);
                    state.resonance_notes = design.resonances;
                    state.tairua_loss_value = design.loss;
                    
                    // Set frequencies/impedances for plot
                    state.frequencies = frequencies;
                    state.impedances = impedances;
                    
                    state.simulation_message = format!("✓ Optimization Complete! Loss: {:.4}", design.loss);
                }
                BackgroundTaskResult::OptimizationError(err) => {
                    state.is_simulating = false;
                    state.optimization_progress = 0.0;
                    state.last_error = Some(err);
                    state.simulation_message = "Optimization failed".to_string();
                }
            }
        }
    }
}

fn sync_previous_state(mut prev: ResMut<PreviousState>, state: Res<CadsdState>) {
    prev.segments = state.segments;
    prev.style_type = state.style_type.clone();
    prev.length = state.length;
    prev.top_diameter= state.top_diameter;
    prev.bottom_diameter= state.bottom_diameter;
    prev.bore_curve = state.bore_curve;
    prev.show_3d = state.show_3d;
}

fn update_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Transform, Entity), With<DidgeMesh>>,
    mut commands: Commands,
    state: Res<CadsdState>,
    prev: Res<PreviousState>,
) {
    // Don't update if not showing 3D
    if !state.show_3d {
        for (_, entity) in query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }
    
    // Mesh rotation system – rotates the mesh if enabled
    if state.mesh_rotation_enabled {
        for (mut transform, _) in query.iter_mut() {
            let delta = 0.016; // Approx 60 fps
            let angle = state.mesh_rotation_speed.to_radians() * delta;
            transform.rotate_y(angle);
        }
    }
    
    // Check if geometry changed OR mesh doesn't exist yet
    let mesh_exists = query.iter().len() > 0;
    let geometry_changed = state.segments != prev.segments
        || state.style_type != prev.style_type
        || state.length != prev.length
        || state.top_diameter != prev.top_diameter
        || state.bottom_diameter != prev.bottom_diameter
        || (state.bore_curve - prev.bore_curve).abs() > 0.001;
    
    // Skip update only if mesh exists AND nothing changed
    if mesh_exists && !geometry_changed {
        return; // Mesh exists and is up to date
    }
    
    // Remove old mesh if it exists
    for (_, entity) in query.iter() {
        commands.entity(entity).despawn();
    }
    
    let geo = create_geometry(&state);
    
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    
    let scale = 0.001;
    for i in 0..=state.segments {
        let pos = (i as f64 / state.segments as f64) * geo.length();
        let diameter = geo.diameter_at_x(pos) * scale;
        let radius = diameter / 2.0;
        let x = (pos - geo.length() / 2.0) * scale;
        
        let circle_segments = 32;
        for j in 0..circle_segments {
            let angle = 2.0 * std::f64::consts::PI * (j as f64) / (circle_segments as f64);
            let y = radius * angle.cos();
            let z = radius * angle.sin();
            
            positions.push([x as f32, y as f32, z as f32]);
            normals.push([0.0, angle.cos() as f32, angle.sin() as f32]);
        }
    }
    
    let circle_segments = 32;
    for i in 0..state.segments {
        for j in 0..circle_segments {
            let current = i * circle_segments + j;
            let next = i * circle_segments + (j + 1) % circle_segments;
            let next_ring = (i + 1) * circle_segments + j;
            let next_ring_next = (i + 1) * circle_segments + (j + 1) % circle_segments;
            
            indices.extend_from_slice(&[current as u32, next_ring as u32, next as u32]);
            indices.extend_from_slice(&[next as u32, next_ring as u32, next_ring_next as u32]);
        }
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    
    // Apply selected color scheme when spawning mesh
    let base_color = match state.color_scheme.as_str() {
        "wood" => Color::srgb(0.7, 0.5, 0.3),
        "metal" => Color::srgb(0.8, 0.8, 0.8),
        "custom" => Color::srgb(0.5, 0.7, 0.9),
        _ => Color::srgb(0.7, 0.5, 0.3),
    };

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color,
            metallic: 0.15,
            perceptual_roughness: 0.4,
            reflectance: 0.35,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        DidgeMesh,
    ));
}

pub fn create_geometry(state: &CadsdState) -> Geo {
    let mut geo = match state.style_type.as_str() {
        "cylinder" => Geo::make_cone(
            state.length as f64,
            state.top_diameter as f64,
            state.top_diameter as f64,
            state.segments,
        ),
        "cone" => Geo::make_cone(
            state.length as f64,
            state.top_diameter as f64,
            state.bottom_diameter as f64,
            state.segments,
        ),
        "exponential" | "kigali" => Geo::make_kigali(
            state.length as f64,
            state.top_diameter as f64,
            state.bottom_diameter as f64,
            state.bore_curve as f64,
            state.segments,
        ),
        "mbeya" => Geo::make_mbeya(
            state.length as f64,
            state.top_diameter as f64,
            state.bottom_diameter as f64,
            state.bore_curve as f64,
            state.segments,
        ),
        _ => Geo::make_cone(
            state.length as f64,
            state.top_diameter as f64,
            state.bottom_diameter as f64,
            state.segments,
        ),
    };

    if state.enable_holes {
        for (pos, dia) in state.hole_positions.iter().zip(state.hole_diameters.iter()) {
            let width = 5.0_f64;
            let height = -(dia / 2.0) as f64;
            geo.make_bubble(*pos as f64, width, height);
        }
    }

    geo
}

// === PANEL FUNCTIONS FOR EACH TAB ===

fn show_forward_design_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("Forward Design");
    ui.separator();
    ui.small("Define geometry → Simulate acoustics");
    ui.separator();
    
    ui.heading("Base Didgeridoo");
    
    ui.horizontal(|ui| {
        ui.label("Style:");
        egui::ComboBox::from_id_salt("style")
            .selected_text(&state.style_type)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.style_type, "cone".to_string(), "Cone");
                ui.selectable_value(&mut state.style_type, "cylinder".to_string(), "Cylinder");
                ui.selectable_value(&mut state.style_type, "exponential".to_string(), "Exponential");
            });
    });
    
    ui.add_enabled_ui(!state.is_simulating, |ui| {
        ui.add(egui::Slider::new(&mut state.length, 500.0..=3000.0)
            .text("Length (mm)")
            .step_by(10.0));
        
        ui.add(egui::Slider::new(&mut state.top_diameter, 15.0..=50.0)
            .text("Mouth Diameter (mm)")
            .step_by(1.0));
        
        ui.add(egui::Slider::new(&mut state.bottom_diameter, 40.0..=150.0)
            .text("Bell Diameter (mm)")
            .step_by(1.0));
        
        ui.add(egui::Slider::new(&mut state.segments, 10..=100)
            .text("Segments")
            .step_by(5.0));
    });
    
    ui.separator();
    
    ui.collapsing("Mouthpiece", |ui| {
        ui.checkbox(&mut state.enable_mouthpiece, "Enable mouthpiece");
        if state.enable_mouthpiece {
            egui::ComboBox::from_id_salt("mouthpiece_type")
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
        }
    });
    
    ui.collapsing("Finger Holes", |ui| {
        ui.checkbox(&mut state.enable_holes, "Enable holes");
        if state.enable_holes {
            ui.add(egui::Slider::new(&mut state.hole_count, 0usize..=12)
                .text("Number of holes"));
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
            ui.label(&state.simulation_message);
        });
    } else if let Some(error) = &state.last_error {
        ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
    } else if !state.impedances.is_empty() {
        ui.colored_label(egui::Color32::GREEN, "✓ Complete");
    }
    
    ui.add_enabled_ui(!state.is_simulating, |ui| {
        if ui.button("▶ Run Simulation").clicked() {
            state.pending_simulation = true;
            state.last_error = None;
            state.simulation_message = "Computing impedance spectrum...".to_string();
        }
    });
    
    ui.separator();
    ui.collapsing("Visualization", |ui| {
        ui.checkbox(&mut state.mesh_rotation_enabled, "Enable mesh rotation");
        if state.mesh_rotation_enabled {
            ui.add(egui::Slider::new(&mut state.mesh_rotation_speed, 0.0..=180.0)
                .text("Rotation speed (°/s)")
                .step_by(5.0));
        }
        ui.label("Color scheme:");
        egui::ComboBox::from_id_salt("color_scheme")
            .selected_text(&state.color_scheme)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.color_scheme, "wood".to_string(), "Wood");
                ui.selectable_value(&mut state.color_scheme, "metal".to_string(), "Metal");
                ui.selectable_value(&mut state.color_scheme, "custom".to_string(), "Custom");
            });
    });
}

fn show_inverse_design_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("Inverse Design");
    ui.separator();
    ui.small("Define target sound & constraints → Optimize geometry");
    ui.separator();
    
    ui.add_enabled_ui(!state.is_simulating, |ui| {
        ui.heading("Target Sound");
        
        ui.add(egui::Slider::new(&mut state.target_frequency, 40.0..=200.0)
            .text("Fundamental (Hz)")
            .step_by(0.1));
        ui.small(format!("Note: {}", note_name(freq_to_note(state.target_frequency))));
        
        ui.horizontal(|ui| {
            ui.label("Preferred Shape:");
            egui::ComboBox::from_id_salt("opt_bore_shape")
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
                ui.label(&state.simulation_message);
            });
            ui.add(egui::ProgressBar::new(state.optimization_progress)
                .text(format!("{:.1}%", state.optimization_progress * 100.0))
                .animate(true));
        });
    } else if let Some(error) = &state.last_error {
        ui.colored_label(egui::Color32::RED, format!("âŒ {}", error));
    } else if state.tairua_loss_value > 0.0 {
        ui.colored_label(egui::Color32::GREEN, format!("âœ“ Complete. Loss: {:.4}", state.tairua_loss_value));
    }
    
    ui.add_enabled_ui(!state.is_simulating, |ui| {
        if ui.button("ðŸš€ Run Optimization").clicked() {
            state.pending_optimization = true;
            state.is_simulating = true;
            state.last_error = None;
            state.simulation_message = "Initializing population...".to_string();
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
                ui.label(format!("{}", i+1));
                ui.label(format!("{:.2} Hz", freq));
                ui.label(format!("({})", note_name(freq_to_note(*freq))));
                ui.small(format!("imp: {:.2e}", amp));
            });
        }
    });
    
    ui.separator();
    ui.heading("Harmonic Analysis");
    
    if let Some(fund) = state.fundamental_freq {
        ui.small(format!("Fundamental: {:.2} Hz ({})", fund, note_name(freq_to_note(fund))));
        
        for (i, (freq, _)) in state.resonance_notes.iter().take(8).enumerate() {
            let ratio = freq / fund;
            let expected = (i + 1) as f64;
            let deviation = (ratio - expected) / expected * 100.0;
            
            ui.horizontal(|ui| {
                ui.small(format!("H{}", i+1));
                ui.small(format!("{:.2}x", ratio));
                ui.small(format!("{:+.1}%", deviation));
            });
        }
    }
}

// Export & Import panel consolidating functionality
fn show_export_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    crate::ui::show_export_panel(ui, state);
}

// === SETTINGS PANEL ===

fn show_settings_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    crate::ui::show_settings_panel(ui, state);
}



// Recreated helper function for geometry creation
// NOTE: `create_geometry` is implemented above in this file.
// This duplicate definition previously caused a compile error in `cadsd-accurate`.
// Keeping the old code here would break `cargo test`.

