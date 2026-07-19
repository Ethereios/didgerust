//! Geometry exporter module for writing 3D meshes (OBJ format)

use crate::geo::Geo;
use crate::integration::GeometryExporter;
use std::io::Write;

/// Default exporter implementing the `GeometryExporter` trait.
pub struct DefaultExporter;

impl GeometryExporter for DefaultExporter {
    fn export_obj(&self, geo: &Geo, writer: &mut impl Write) -> std::io::Result<()> {
        writeln!(writer, "# DidgeRust CADSD 3D Bore Mesh")?;
        writeln!(writer, "# Length: {:.1} mm", geo.length())?;
        writeln!(writer, "# Bell size: {:.1} mm", geo.bellsize())?;
        writeln!(writer, "# Segments: {}", geo.geo.len() - 1)?;
        writeln!(writer, "")?;

        let segments = geo.geo.len();
        let ring_subdivisions = 32; // Number of points in each circular cross-section

        // 1. Write vertices
        for i in 0..segments {
            let x = geo.geo[i][0];
            let radius = geo.geo[i][1] / 2.0;

            for j in 0..ring_subdivisions {
                let angle = 2.0 * std::f64::consts::PI * (j as f64) / (ring_subdivisions as f64);
                let y = radius * angle.cos();
                let z = radius * angle.sin();
                
                // Write coordinates: X (bore axis), Y, Z
                writeln!(writer, "v {:.6} {:.6} {:.6}", x, y, z)?;
            }
        }

        writeln!(writer, "")?;

        // 2. Write quad faces connecting the rings
        for i in 0..(segments - 1) {
            for j in 0..ring_subdivisions {
                let current_ring = i * ring_subdivisions;
                let next_ring = (i + 1) * ring_subdivisions;
                
                let next_j = (j + 1) % ring_subdivisions;

                // OBJ vertex indices are 1-based
                let v1 = current_ring + j + 1;
                let v2 = current_ring + next_j + 1;
                let v3 = next_ring + next_j + 1;
                let v4 = next_ring + j + 1;

                // Write face quad (counter-clockwise orientation)
                writeln!(writer, "f {} {} {} {}", v1, v4, v3, v2)?;
            }
        }

        Ok(())
    }

    fn export_gltf(&self, geo: &Geo, writer: &mut impl Write) -> std::io::Result<()> {
        // A minimal valid GLTF JSON structure for the geometry
        // For a full implementation, we would write the vertices, normals, and indices to a binary buffer.
        // This serves as a structural placeholder for the requested GLTF integration.
        let gltf_json = serde_json::json!({
            "asset": {
                "version": "2.0",
                "generator": "DidgeRust CADSD"
            },
            "scene": 0,
            "scenes": [
                {
                    "nodes": [0]
                }
            ],
            "nodes": [
                {
                    "mesh": 0,
                    "name": "Didgeridoo"
                }
            ],
            "meshes": [
                {
                    "name": "Bore Mesh",
                    "extras": {
                        "length_mm": geo.length(),
                        "bell_size_mm": geo.bellsize(),
                        "segments": geo.geo.len() - 1
                    }
                }
            ]
        });
        
        let json_str = serde_json::to_string_pretty(&gltf_json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            
        writer.write_all(json_str.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::Geo;

    #[test]
    fn test_obj_export_structure() {
        let geo = Geo::make_cone(1000.0, 30.0, 60.0, 10);
        let exporter = DefaultExporter;
        
        let mut buffer = Vec::new();
        let result = exporter.export_obj(&geo, &mut buffer);
        assert!(result.is_ok());
        
        let output = String::from_utf8(buffer).unwrap();
        
        // With 11 points (10 segments) and 32 subdivisions, we expect 11 * 32 = 352 vertices
        let v_count = output.lines().filter(|l| l.starts_with("v ")).count();
        assert_eq!(v_count, 352);
        
        // With 10 segments and 32 subdivisions, we expect 10 * 32 = 320 faces
        let f_count = output.lines().filter(|l| l.starts_with("f ")).count();
        assert_eq!(f_count, 320);
    }
}
