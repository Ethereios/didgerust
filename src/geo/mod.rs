//! Geometry module – re-exports cadsd-accurate's Geo.
//!
//! The wrapper directly uses the accurate crate's Geo implementation
//! ensuring full Python DidgeLab parity.

pub use cadsd_accurate::geo::Geo;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_make_cone() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        assert_eq!(geo.geo.len(), 21);
        assert_eq!(geo.geo[0], [0.0, 32.0]);
        assert_eq!(geo.geo[20], [1000.0, 60.0]);
        assert!((geo.length() - 1000.0).abs() < 1.0);
        assert!((geo.bellsize() - 60.0).abs() < 1.0);
    }

    #[test]
    fn test_geo_add_bubble() {
        let mut geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        geo.make_bubble(500.0, 200.0, 70.0);
        let centre_d = geo.diameter_at_x(500.0);
        assert!(centre_d > 20.0);
    }

    #[test]
    fn test_geo_compute_volume() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 10);
        let volume = geo.compute_volume();
        assert!(volume > 0.0);
    }

    #[test]
    fn test_geo_diameter_at_x() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        let d_start = geo.diameter_at_x(0.0);
        let d_mid = geo.diameter_at_x(500.0);
        let d_end = geo.diameter_at_x(1000.0);
        assert!((d_start - 32.0).abs() < 1.0);
        assert!(d_mid > 32.0);
        assert!(d_mid < 60.0);
        assert!((d_end - 60.0).abs() < 1.0);
    }

    #[test]
    fn test_geo_stretch() {
        let mut geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        geo.stretch(2.0);
        assert!((geo.length() - 2000.0).abs() < 1.0);
        assert!((geo.bellsize() - 60.0).abs() < 1.0);
    }

    #[test]
    fn test_geo_scalediameter_to() {
        let mut geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        geo.scale_diameter(30.0);
        assert!((geo.get_max_d() - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_geo_copy() {
        let geo = Geo::make_cone(1000.0, 32.0, 60.0, 20);
        let geo_copy = geo.copy();
        assert_eq!(geo.geo.len(), geo_copy.geo.len());
        assert_eq!(geo.geo[0], geo_copy.geo[0]);
    }
}