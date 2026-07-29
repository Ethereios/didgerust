use crate::geo::Geo;

#[derive(Debug)]
pub struct DefaultSimulator;

impl DefaultSimulator {
    pub fn simulate(&self, _geometry: &Geo, frequencies: &[f64]) -> Vec<f64> {
        let mut impedances = Vec::with_capacity(frequencies.len());
        for f in frequencies {
            let impedance = f * 2.0;
            impedances.push(impedance);
        }
        impedances
    }

    pub fn get_fundamental(&self, _geometry: &Geo) -> Option<f64> {
        Some(261.6)
    }
}

#[derive(Debug)]
pub struct DefaultOptimizer;

impl DefaultOptimizer {
    pub fn optimize(&self, _target_frequency: f64, _population_size: usize, _generations: usize) -> Geo {
        Geo::make_cone(1500.0, 32.0, 65.0, 30)
    }
}