use bevy_egui::egui;
use rfd::FileDialog;
use std::fs;
use crate::app::CadsdState;
// Note: create_geometry will be exposed from app.rs for now
// We also need access to DefaultExporter and DefaultSynthesizer

pub fn show_export_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("Export & Import");
    ui.separator();

    // ----- Export Geometry -----
    ui.heading("Export Geometry");
    ui.horizontal(|ui| {
        if ui.button("💾 Export as JSON").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name("geometry.json")
                .save_file()
            {
                let geo = crate::app::create_geometry(state);
                if let Ok(json) = serde_json::to_string_pretty(&geo) {
                    let _ = fs::write(&path, json);
                }
            }
        }
        if ui.button("💾 Export as TXT").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("TXT", &["txt"])
                .set_file_name("geometry.txt")
                .save_file()
            {
                let geo = crate::app::create_geometry(state);
                let mut txt = String::new();
                for pt in &geo.geo {
                    txt.push_str(&format!("{},{}\n", pt[0], pt[1]));
                }
                let _ = fs::write(&path, txt);
            }
        }
        if ui.button("💾 Export as 3D OBJ").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("OBJ Mesh", &["obj"])
                .set_file_name("geometry.obj")
                .save_file()
            {
                let geo = crate::app::create_geometry(state);
                if let Ok(mut file) = fs::File::create(&path) {
                    let exporter = crate::export::DefaultExporter;
                    let _ = crate::integration::GeometryExporter::export_obj(&exporter, &geo, &mut file);
                }
            }
        }
        if ui.button("💾 Export as 3D GLTF").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("GLTF Mesh", &["gltf"])
                .set_file_name("geometry.gltf")
                .save_file()
            {
                let geo = crate::app::create_geometry(state);
                if let Ok(mut file) = fs::File::create(&path) {
                    let exporter = crate::export::DefaultExporter;
                    let _ = crate::integration::GeometryExporter::export_gltf(&exporter, &geo, &mut file);
                }
            }
        }
    });
    ui.separator();

    // ----- Import Geometry -----
    ui.heading("Import Geometry");
    ui.horizontal(|ui| {
        if ui.button("📂 Import JSON").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name("geometry.json")
                .pick_file()
            {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(geo) = serde_json::from_str::<crate::geo::Geo>(&content) {
                        state.length = geo.length() as f32;
                        state.top_diameter = geo.geo.first().unwrap()[1] as f32;
                        state.bottom_diameter = geo.geo.last().unwrap()[1] as f32;
                        state.segments = geo.geo.len() - 1;
                        state.enable_holes = false;
                        state.hole_positions.clear();
                        state.hole_diameters.clear();
                        state.pending_simulation = false;
                    }
                }
            }
        }
        if ui.button("📂 Import TXT").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("TXT", &["txt"])
                .set_file_name("geometry.txt")
                .pick_file()
            {
                if let Ok(content) = fs::read_to_string(&path) {
                    let mut points = Vec::new();
                    for line in content.lines() {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() == 2 {
                            if let (Ok(x), Ok(d)) = (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
                                points.push([x, d]);
                            }
                        }
                    }
                    if !points.is_empty() {
                        let geo = crate::geo::Geo { geo: points };
                        state.length = geo.length() as f32;
                        state.top_diameter = geo.geo.first().unwrap()[1] as f32;
                        state.bottom_diameter = geo.geo.last().unwrap()[1] as f32;
                        state.segments = geo.geo.len() - 1;
                        state.enable_holes = false;
                        state.hole_positions.clear();
                        state.hole_diameters.clear();
                        state.pending_simulation = false;
                    }
                }
            }
        }
    });
    ui.separator();

    // ----- Export Simulation Data -----
    ui.heading("Export Simulation Data");
    ui.horizontal(|ui| {
        if ui.button("💾 Export Impedance CSV").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("CSV", &["csv"])
                .set_file_name("impedance.csv")
                .save_file()
            {
                let mut csv = String::new();
                csv.push_str("frequency_hz,impedance_magnitude\n");
                for (f, z) in state.frequencies.iter().zip(state.impedances.iter()) {
                    csv.push_str(&format!("{},{}\n", f, z));
                }
                let _ = fs::write(&path, csv);
            }
        }
        if ui.button("🎵 Export Audio WAV").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("WAV Audio", &["wav"])
                .set_file_name("didgeridoo.wav")
                .save_file()
            {
                let geo = crate::app::create_geometry(state);
                if let Ok(mut file) = fs::File::create(&path) {
                    let synth = crate::audio::DefaultSynthesizer;
                    let samples = crate::integration::AudioSynthesizer::synthesize(&synth,
                        &geo,
                        &state.frequencies,
                        &state.impedances,
                        4.0, // 4 seconds duration
                        44100, // CD quality
                    );
                    let _ = crate::audio::write_wav_file(&samples, 44100, &mut file);
                }
            }
        }
    });
    ui.separator();

    // ----- Current Design Summary -----
    ui.heading("Current Design");
    ui.small(format!("Length: {:.0} mm", state.length));
    ui.small(format!("Mouth: {:.1} mm", state.top_diameter));
    ui.small(format!("Bell: {:.1} mm", state.bottom_diameter));
    ui.small(format!("Segments: {}", state.segments));
}
