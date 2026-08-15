use crate::Geo;

pub struct GeometryExporter;

impl GeometryExporter {
    pub fn export_obj(_geometry: &Geo) -> String {
        String::from("# Wavefront OBJ\n# CADSD Geometry Export\n")
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