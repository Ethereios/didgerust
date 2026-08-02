use bevy_egui::egui;
use crate::app::CadsdState;
use rfd::FileDialog;
use std::fs;
use crate::geo::Geo;
use serde_json::to_string_pretty;

pub fn show_export_panel(ui: &mut egui::Ui, state: &mut CadsdState) {
    ui.heading("📦 Export");
    ui.separator();

    ui.heading("Export Geometry");
    ui.horizontal(|ui| {
        if ui.button("💾 Export as JSON").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name("geometry.json")
                .save_file()
            {
                let geo = create_geometry(state);
                if let Ok(json) = to_string_pretty(&geo) {
                    let _ = fs::write(&path, json);
                    state.sim_message = format!("Exported to {}", path.display());
                }
            }
        }
        if ui.button("💾 Export as TXT").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("TXT", &["txt"])
                .set_file_name("geometry.txt")
                .save_file()
            {
                let geo = create_geometry(state);
                let mut txt = String::new();
                for pt in &geo.geo {
                    txt.push_str(&format!("{:.6},{:.6}\n", pt[0], pt[1]));
                }
                let _ = fs::write(&path, txt);
                state.sim_message = format!("Exported to {}", path.display());
            }
        }
        if ui.button("💾 Export as OBJ").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("OBJ Mesh", &["obj"])
                .set_file_name("geometry.obj")
                .save_file()
            {
                let geo = create_geometry(state);
                let obj_content = export_to_obj(&geo);
                let _ = fs::write(&path, obj_content);
                state.sim_message = format!("Exported to {}", path.display());
            }
        }
    });
    ui.separator();

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
                state.sim_message = format!("Exported to {}", path.display());
            }
        }
        if ui.button("🎵 Export Audio WAV").clicked() {
            state.sim_message = "Audio export placeholder".to_string();
        }
    });
    ui.separator();

    ui.heading("Current Design");
    ui.label(&format!("Length: {:.1} mm", state.length));
    ui.label(&format!("Mouth: {:.1} mm", state.top_diameter));
    ui.label(&format!("Bell: {:.1} mm", state.bottom_diameter));
    ui.label(&format!("Segments: {}", state.segments));
}

fn create_geometry(state: &CadsdState) -> Geo {
    let n = (state.segments as usize).max(1);
    let mut points = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / (n - 1) as f64;
        let x = t * state.length as f64;
        let d = state.top_diameter as f64
            + t * (state.bottom_diameter as f64 - state.top_diameter as f64);
        points.push([x, d]);
    }
    Geo::new(points)
}

fn export_to_obj(geo: &Geo) -> String {
    let mut obj = String::from("# CADSD Geometry Export\n");
    for pt in geo.geo.iter() {
        obj.push_str(&format!("v {:.6} 0.0 {:.6}\n", pt[0], pt[1]));
    }
    for i in 0..geo.geo.len().saturating_sub(1) {
        obj.push_str(&format!("l {} {}\n", i + 1, i + 2));
    }
    obj
}