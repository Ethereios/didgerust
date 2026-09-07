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

    pub fn export_gltf(geometry: &Geo) -> Vec<u8> {
        let positions: Vec<f32> = geometry.geo
            .iter()
            .flat_map(|pt| vec![pt[0] as f32, 0.0, pt[1] as f32])
            .collect();
        
        let indices: Vec<u32> = (0..geometry.geo.len().saturating_sub(1) as u32)
            .flat_map(|i| vec![i, i + 1])
            .collect();

        let json = format!(
            r#"{{
  "asset": {{ "version": "2.0", "generator": "CADSD" }},
  "scene": 0,
  "scenes": [{{ "nodes": [0] }}],
  "nodes": [{{ "mesh": 0 }}],
  "meshes": [{{ "primitives": [{{ "mode": 1, "attributes": {{ "POSITION": 0 }}, "indices": 1 }}] }}],
  "accessors": [
    {{
      "bufferView": 0,
      "componentType": 5126,
      "count": {},
      "type": "VEC3",
      "max": [{}, 0.0, {}],
      "min": [{}, 0.0, {}]
    }},
    {{
      "bufferView": 1,
      "componentType": 5125,
      "count": {},
      "type": "SCALAR"
    }}
  ],
  "bufferViews": [
    {{ "buffer": 0, "byteOffset": 0, "byteLength": {}, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": {}, "byteLength": {}, "target": 34963 }}
  ],
  "buffers": [{{ "byteLength": {} }}]
}}"#,
            positions.len() / 3,
            positions.iter().step_by(3).cloned().fold(f32::NEG_INFINITY, f32::max),
            positions.iter().step_by(3).skip(2).cloned().fold(f32::NEG_INFINITY, f32::max),
            positions.iter().step_by(3).cloned().fold(f32::INFINITY, f32::min),
            positions.iter().step_by(3).skip(2).cloned().fold(f32::INFINITY, f32::min),
            indices.len(),
            positions.len() * 4,
            positions.len() * 4,
            indices.len() * 4,
            positions.len() * 4 + indices.len() * 4
        );

        let mut output = Vec::new();
        output.extend_from_slice(json.as_bytes());
        output.extend_from_slice(&positions.iter().flat_map(|&f| f.to_le_bytes()).collect::<Vec<u8>>());
        output.extend_from_slice(&indices.iter().flat_map(|&i| i.to_le_bytes()).collect::<Vec<u8>>());
        output
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