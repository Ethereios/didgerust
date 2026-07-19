//! DidgeRust - Bevy + egui CADSD Application
//!
//! Professional acoustic simulation and design tool combining:
//! - Bevy 3D engine for real-time bore visualization
//! - egui immediate mode GUI for controls and analysis
//! - CADSD acoustic simulation backend
//! - Extensible architecture for holes, mouthpieces, exotic instruments

#[cfg(feature = "gui-didgelab")]
use eframe::egui;
#[cfg(feature = "gui-didgelab")]
use egui_plot::{Plot, Line, VLine, HLine};
#[cfg(feature = "gui-didgelab")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "gui-didgelab")]
use std::thread;

#[cfg(feature = "gui-didgelab")]
use crate::inverse_design::{InverseDesigner, DesignResult};
#[cfg(feature = "gui-didgelab")]
use crate::evo::{TargetSound, BoreShapePreference};
#[cfg(feature = "gui-didgelab")]
use crate::geo::Geo;
#[cfg(feature = "gui-didgelab")]
use crate::sim::{get_log_simulation_frequencies, acoustical_simulation};
#[cfg(feature = "gui-didgelab")]
use crate::conv::note_to_freq;

/// Main application state
struct DidgeLabApp {
    // Input parameters
    target_note: String,
    target_frequency: f64,
    use_custom_frequency: bool,
    
    // Toots
    toot_notes: Vec<String>,
    
    // Overtones
    include_overtones: bool,
    overtone_count: usize,
    
    // Bore shape
    bore_shape: String,
    
    // Constraints
    min_length: f32,
    max_length: f32,
    min_bell: f32,
    max_bell: f32,
    
    // Optimization parameters
    population_size: usize,
    generations: usize,
    
    // Results
    optimization_running: bool,
    optimization_progress: Option<OptimizationProgress>,
    design_result: Option<DesignResult>,
    impedance_data: Vec<[f64; 2]>,
    error_message: Option<String>,
    
    // View options
    show_advanced: bool,
    show_3d_preview: bool,
    selected_candidate: usize,
}

#[derive(Clone, Debug)]
struct OptimizationProgress {
    current_generation: usize,
    total_generations: usize,
    best_loss: f64,
    message: String,
}

impl Default for DidgeLabApp {
    fn default() -> Self {
        Self {
            target_note: "D1".to_string(),
            target_frequency: 73.4,
            use_custom_frequency: false,
            toot_notes: vec![],
            include_overtones: true,
            overtone_count: 6,
            bore_shape: "Conical".to_string(),
            min_length: 500.0,
            max_length: 2500.0,
            min_bell: 30.0,
            max_bell: 120.0,
            population_size: 50,
            generations: 100,
            optimization_running: false,
            optimization_progress: None,
            design_result: None,
            impedance_data: vec![],
            error_message: None,
            show_advanced: false,
            show_3d_preview: true,
            selected_candidate: 0,
        }
    }
}

impl DidgeLabApp {
    fn create_target_sound(&self) -> Result<TargetSound, String> {
        let freq = if self.use_custom_frequency {
            self.target_frequency
        } else {
            let target = TargetSound::from_note(&self.target_note)?;
            target.fundamental_freq
        };
        
        let mut target = TargetSound::new(freq);
        
        // Add toots
        for toot_note in &self.toot_notes {
            if !toot_note.is_empty() {
                target = target.with_toot_note(toot_note)?;
            }
        }
        
        // Set overtones
        if self.include_overtones {
            let overtones: Vec<usize> = (2..=(self.overtone_count + 1)).collect();
            target = target.with_overtones(overtones);
        }
        
        // Set bore shape
        let bore_shape = match self.bore_shape.as_str() {
            "Cylindrical" => BoreShapePreference::Cylindrical,
            "Conical" => BoreShapePreference::Conical,
            "Flared" => BoreShapePreference::Flared,
            _ => BoreShapePreference::Any,
        };
        target = target.with_bore_shape(bore_shape);
        
        // Set constraints
        target = target.with_length_range(self.min_length as f64, self.max_length as f64);
        target = target.with_bell_range(self.min_bell as f64, self.max_bell as f64);
        
        Ok(target)
    }
    
    fn run_optimization(&mut self) {
        self.error_message = None;
        self.design_result = None;
        self.impedance_data.clear();
        
        let target = match self.create_target_sound() {
            Ok(t) => t,
            Err(e) => {
                self.error_message = Some(format!("Invalid input: {}", e));
                return;
            }
        };
        
        self.optimization_running = true;
        self.optimization_progress = Some(OptimizationProgress {
            current_generation: 0,
            total_generations: self.generations,
            best_loss: f64::INFINITY,
            message: "Starting optimization...".to_string(),
        });
        
        let pop_size = self.population_size;
        let gens = self.generations;
        
        let progress = Arc::new(Mutex::new(self.optimization_progress.clone().unwrap()));
        let progress_clone = progress.clone();
        
        // Run optimization in background thread
        thread::spawn(move || {
            let designer = InverseDesigner::new()
                .with_population_size(pop_size)
                .with_generations(gens)
                .with_verbose(false);
            
            // TODO: Add progress callbacks
            let result = designer.design(target);
            
            // Store result
            if let Ok(design) = result {
                // Could use channel to send back to main thread
            }
        });
        
        // For now, run synchronously (will upgrade to async later)
        self.run_optimization_sync();
    }
    
    fn run_optimization_sync(&mut self) {
        let target = match self.create_target_sound() {
            Ok(t) => t,
            Err(e) => {
                self.error_message = Some(format!("Invalid input: {}", e));
                return;
            }
        };
        
        self.optimization_running = true;
        
        let designer = InverseDesigner::new()
            .with_population_size(self.population_size)
            .with_generations(self.generations)
            .with_verbose(false);
        
        match designer.design(target) {
            Ok(result) => {
                // Compute impedance spectrum for visualization
                let frequencies = get_log_simulation_frequencies();
                if let Ok(impedances) = acoustical_simulation(&result.geometry, &frequencies, "tlm_python") {
                    self.impedance_data = frequencies.iter()
                        .enumerate()
                        .map(|(i, &freq)| [freq, impedances[i]])
                        .collect();
                }
                
                self.design_result = Some(result);
                self.optimization_running = false;
                self.optimization_progress = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Optimization failed: {}", e));
                self.optimization_running = false;
                self.optimization_progress = None;
            }
        }
    }
    
    fn export_geometry(&self, format: &str) {
        if let Some(result) = &self.design_result {
            let geo = if result.candidates.len() > self.selected_candidate {
                &result.candidates[self.selected_candidate]
            } else {
                &result.geometry
            };
            
            let filename = format!("didgeridoo_design.{}", format);
            match format {
                "txt" => {
                    if let Err(e) = geo.to_file(&filename) {
                        eprintln!("Failed to export: {}", e);
                    }
                }
                "json" => {
                    let json = serde_json::to_string_pretty(&geo.geo).unwrap_or_default();
                    if let Err(e) = std::fs::write(&filename, json) {
                        eprintln!("Failed to export: {}", e);
                    }
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for DidgeLabApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎵 DidgeLab - Design Your Didgeridoo");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❓ Help").clicked() {
                        // Show help dialog
                    }
                });
            });
        });
        
        egui::SidePanel::left("input_panel")
            .min_width(320.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                self.show_input_panel(ui);
            });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_results_panel(ui);
        });
    }
}

impl DidgeLabApp {
    fn show_input_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎯 Target Sound");
        ui.separator();
        
        // Fundamental frequency
        ui.label("Fundamental Key:");
        egui::ComboBox::new("target_note", "")
            .selected_text(self.target_note.as_str())
            .show_ui(ui, |ui: &mut egui::Ui| {
                for note in &["C1", "D1", "E1", "F1", "G1", "A1", "B1", "C2", "D2", "E2"] {
                    ui.selectable_value(&mut self.target_note, note.to_string(), *note);
                }
            });
        
        ui.checkbox(&mut self.use_custom_frequency, "Custom frequency");
        if self.use_custom_frequency {
            ui.add(egui::Slider::new(&mut self.target_frequency, 20.0..=500.0)
                .text("Frequency (Hz)"));
        }
        
        ui.separator();
        
        // Toots
        ui.label("Target Toots (optional):");
        if ui.button("+ Add Toot").clicked() {
            self.toot_notes.push("D4".to_string());
        }
        
        let mut to_remove = None;
        for (i, toot) in self.toot_notes.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("Toot {}:", i + 1));
                egui::ComboBox::new(format!("toot_{}", i), "")
                    .selected_text(toot.as_str())
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        for note in &["D3", "A3", "D4", "A4", "E4", "G4", "C5", "D5"] {
                            ui.selectable_value(toot, note.to_string(), *note);
                        }
                    });
                if ui.button("✕").clicked() {
                    to_remove = Some(i);
                }
            });
        }
        if let Some(idx) = to_remove {
            self.toot_notes.remove(idx);
        }
        
        ui.separator();
        
        // Overtones
        ui.checkbox(&mut self.include_overtones, "Target overtones");
        if self.include_overtones {
            ui.add(egui::Slider::new(&mut self.overtone_count, 2usize..=10)
                .text("Number of overtones"));
        }
        
        ui.separator();
        
        // Bore shape
        ui.label("Bore Shape Preference:");
        egui::ComboBox::new("bore_shape", "")
            .selected_text(self.bore_shape.as_str())
            .show_ui(ui, |ui: &mut egui::Ui| {
                for shape in &["Any", "Cylindrical", "Conical", "Flared"] {
                    ui.selectable_value(&mut self.bore_shape, shape.to_string(), *shape);
                }
            });
        
        ui.separator();
        
        // Advanced options
        ui.collapsing("⚙️ Advanced Options", |ui| {
            ui.label("Length Range (mm):");
            ui.add(egui::Slider::new(&mut self.min_length, 500.0..=2000.0).text("Min"));
            ui.add(egui::Slider::new(&mut self.max_length, 1000.0..=3000.0).text("Max"));
            
            ui.label("Bell Diameter Range (mm):");
            ui.add(egui::Slider::new(&mut self.min_bell, 20.0..=80.0).text("Min"));
            ui.add(egui::Slider::new(&mut self.max_bell, 50.0..=150.0).text("Max"));
            
            ui.separator();
            
            ui.label("Optimization:");
            ui.add(egui::Slider::new(&mut self.population_size, 20usize..=100).text("Population"));
            ui.add(egui::Slider::new(&mut self.generations, 20usize..=200).text("Generations"));
        });
        
        ui.separator();
        
        // Run button
        if self.optimization_running {
            ui.add_enabled(false, egui::Button::new("⏳ Running..."));
            if let Some(progress) = &self.optimization_progress {
                let progress_pct = progress.current_generation as f32 / progress.total_generations as f32;
                ui.add(egui::ProgressBar::new(progress_pct).text(&progress.message));
            }
        } else {
            if ui.button("🚀 Run Optimization").clicked() {
                self.run_optimization();
            }
        }
        
        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
        }
    }
    
    fn show_results_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(result) = &self.design_result {
            ui.horizontal(|ui| {
                ui.heading("📊 Design Results");
                
                if ui.button("💾 Export TXT").clicked() {
                    self.export_geometry("txt");
                }
                if ui.button("💾 Export JSON").clicked() {
                    self.export_geometry("json");
                }
            });
            
            ui.separator();
            
            // Show key metrics
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.label("Fundamental");
                    ui.heading(format!("{:.1} Hz", result.fundamental_freq));
                });
                ui.group(|ui| {
                    ui.label("Length");
                    ui.heading(format!("{:.0} mm", result.geometry.length()));
                });
                ui.group(|ui| {
                    ui.label("Bell");
                    ui.heading(format!("{:.1} mm", result.geometry.bellsize()));
                });
                ui.group(|ui| {
                    ui.label("Resonances");
                    ui.heading(format!("{}", result.resonances.len()));
                });
            });
            
            ui.separator();
            
            // Impedance spectrum plot
            ui.heading("Impedance Spectrum");
            if !self.impedance_data.is_empty() {
                Plot::new("impedance_plot")
                    .view_aspect(2.0)
                    .height(250.0)
                    .show(ui, |plot_ui: &mut egui_plot::PlotUi| {
                        plot_ui.line(Line::new(self.impedance_data.clone()).width(2.0));
                        if let Some((fund, _)) = result.resonances.first() {
                            plot_ui.vline(VLine::new(*fund).color(egui::Color32::RED));
                        }
                    });
            }
            
            ui.separator();
            
            // Bore profile visualization
            ui.heading("Bore Profile");
            if self.show_3d_preview {
                self.show_bore_profile(ui, &result.geometry);
            }
            
            ui.separator();
            
            // Resonance peaks
            ui.heading("Resonance Peaks");
            for (i, (freq, imp)) in result.resonances.iter().take(8).enumerate() {
                ui.label(format!("Peak {}: {:.1} Hz (impedance: {:.2e})", i + 1, freq, imp));
            }
            
            // Show multiple candidates if available
            if result.candidates.len() > 1 {
                ui.separator();
                ui.heading("Alternative Designs");
                
                ui.horizontal(|ui| {
                    ui.label("Candidate:");
                    ui.add(egui::Slider::new(&mut self.selected_candidate, 0..=(result.candidates.len() - 1)));
                });
                
                if self.selected_candidate < result.candidates.len() {
                    let geo = &result.candidates[self.selected_candidate];
                    ui.label(format!("Length: {:.0} mm, Bell: {:.1} mm, Volume: {:.0} mm³",
                        geo.length(), geo.bellsize(), geo.compute_volume()));
                }
            }
        } else if self.optimization_running {
            ui.vertical_centered(|ui| {
                ui.add(egui::Spinner::new().size(60.0));
                ui.heading("Optimizing Design...");
                if let Some(progress) = &self.optimization_progress {
                    ui.label(&progress.message);
                    let progress_pct = progress.current_generation as f32 / progress.total_generations as f32;
                    ui.add(egui::ProgressBar::new(progress_pct));
                }
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.heading("🎵 Design Your Didgeridoo");
                ui.label("Describe the sound you want in the left panel");
                ui.label("Then click 'Run Optimization' to find matching geometries");
            });
        }
    }
    
    fn show_bore_profile(&self, ui: &mut egui::Ui, geo: &Geo) {
        use egui::epaint::{PathShape, RectShape, Rounding, Stroke};
        
        let response = ui.allocate_response(egui::vec2(ui.available_width(), 150.0), egui::Sense::hover());
        
        let length = geo.length() as f32;
        let max_diam = geo.get_max_d() as f32;
        
        let mut points = Vec::new();
        let margin = 20.0;
        let width = response.rect.width() - 2.0 * margin;
        let height = response.rect.height() - 2.0 * margin;
        
        // Generate bore profile points
        for i in 0..=100 {
            let x = (i as f32 / 100.0) * length;
            let diam = geo.diameter_at_x(x as f64) as f32;
            
            let px = margin + (x / length) * width;
            let py_upper = response.rect.top() + margin + height / 2.0 - (diam / max_diam) * height / 2.0;
            let py_lower = response.rect.top() + margin + height / 2.0 + (diam / max_diam) * height / 2.0;
            
            points.push((egui::pos2(px, py_upper), egui::pos2(px, py_lower)));
        }
        
        // Draw bore shape
        let mut shape_points: Vec<egui::Pos2> = points.iter().map(|(p, _)| *p).collect();
        shape_points.extend(points.iter().rev().map(|(_, p)| *p));
        
        let fill_color = egui::Color32::from_rgb(180, 140, 100);
        let stroke = Stroke::new(2.0, egui::Color32::from_rgb(120, 80, 40));
        
        ui.painter().add(egui::Shape::convex_polygon(
            shape_points,
            fill_color,
            stroke,
        ));
        
        // Draw centerline
        let centerline: Vec<egui::Pos2> = (0..=100).map(|i| {
            let x = margin + (i as f32 / 100.0) * width;
            let y = response.rect.top() + margin + height / 2.0;
            egui::pos2(x, y)
        }).collect();
        
        ui.painter().add(egui::Shape::line(
            centerline,
            Stroke::new(1.0, egui::Color32::GRAY),
        ));
    }
}

pub fn run_didgelab_app() {
    let app = DidgeLabApp::default();
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1024.0, 600.0])
            .with_title("DidgeLab - Design Your Didgeridoo"),
        ..Default::default()
    };
    
    eframe::run_native(
        "DidgeLab",
        native_options,
        Box::new(|_cc| Box::new(app)),
    ).expect("Failed to run DidgeLab app");
}
