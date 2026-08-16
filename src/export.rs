use crate::Geo;

pub struct GeometryExporter;

impl GeometryExporter {
    pub fn export_obj(geometry: &Geo) -> String {
        let mut obj = String::from("# Wavefront OBJ\n# CADSD Geometry Export\n");
        
        for pt in &geometry.geo {
            obj.push_str(&format!("v {:.6} 0.0 {:.6}\n", pt[0], pt[1]));
        }
        
        for i in 0..geometry.geo.len().saturating_sub(1) {
            obj.push_str(&format!("l {} {}\n", i + 1, i + 2));
        }
        
        obj
    }

    pub fn export_gltf(_geometry: &Geo) -> Vec<u8> {
        Vec::new()
    }
}

pub struct DataExporter;

impl DataExporter {
    pub fn export_spectrum_csv(frequencies: &[f64], impedances: &[f64]) -> String {
        let mut csv = String::from("frequency,impedance\n");
        for (f, z) in frequencies.iter().zip(impedances.iter()) {
            csv.push_str(&format!("{},{}\n", f, z));
        }
        csv
    }
}