//! Minimal CADSD example that works with older Rust versions
//!
//! This is a simplified version of the CADSD library that demonstrates
//! the core concepts without complex dependencies.

use std::f64::consts::PI;

/// Simple segment structure for acoustic simulation
#[derive(Debug, Clone)]
pub struct Segment {
    pub length: f64,    // meters
    pub diameter_in: f64,  // meters
    pub diameter_out: f64, // meters
}

impl Segment {
    pub fn new(length: f64, d_in: f64, d_out: f64) -> Self {
        Self {
            length: length.max(1e-12),
            diameter_in: d_in.max(1e-12),
            diameter_out: d_out.max(1e-12),
        }
    }
    
    /// Create segments from geometry (x_mm, diameter_mm)
    pub fn from_geometry(geo: &[[f64; 2]]) -> Vec<Self> {
        let mut segments = Vec::new();
        
        for i in 1..geo.len() {
            let x1 = geo[i][0] / 1000.0;      // mm to m
            let x0 = geo[i-1][0] / 1000.0;
            let d1 = geo[i][1] / 1000.0;      // mm to m
            let d0 = geo[i-1][1] / 1000.0;
            
            let length = (x1 - x0).max(1e-12);
            segments.push(Segment::new(length, d0, d1));
        }
        
        segments
    }
}

/// Simple didgeridoo geometry representation
#[derive(Debug, Clone)]
pub struct SimpleGeometry {
    pub segments: Vec<[f64; 2]>, // [x_mm, diameter_mm]
}

impl SimpleGeometry {
    pub fn new(segments: Vec<[f64; 2]>) -> Self {
        Self { segments }
    }
    
    /// Create a simple conical shape
    pub fn cone(length_mm: f64, mouth_d_mm: f64, bell_d_mm: f64, n_segments: usize) -> Self {
        let mut segments = vec![[0.0, mouth_d_mm]];
        
        for i in 1..n_segments {
            let x = length_mm * (i as f64) / (n_segments as f64);
            let t = x / length_mm;
            let diameter = mouth_d_mm + (bell_d_mm - mouth_d_mm) * t;
            segments.push([x, diameter]);
        }
        
        segments.push([length_mm, bell_d_mm]);
        Self::new(segments)
    }
    
    pub fn length(&self) -> f64 {
        self.segments.last().map(|seg| seg[0]).unwrap_or(0.0)
    }
    
    pub fn bell_diameter(&self) -> f64 {
        self.segments.last().map(|seg| seg[1]).unwrap_or(0.0)
    }
}

/// Simple acoustic simulator
pub struct SimpleSimulator {
    pub geometry: SimpleGeometry,
    pub min_freq: f64,
    pub max_freq: f64,
    pub step_freq: f64,
}

impl SimpleSimulator {
    pub fn new(geometry: SimpleGeometry) -> Self {
        Self {
            geometry,
            min_freq: 50.0,
            max_freq: 500.0,
            step_freq: 5.0,
        }
    }
    
    /// Compute impedance at a single frequency using simplified model
    pub fn compute_impedance(&self, frequency: f64) -> f64 {
        let segments = Segment::from_geometry(&self.geometry.segments);
        if segments.is_empty() {
            return 0.0;
        }
        
        // Simplified acoustic model
        let omega = 2.0 * PI * frequency;
        let c = 343.0; // speed of sound m/s
        let rho = 1.2; // air density kg/m³
        
        // Simple impedance calculation (very simplified)
        let total_length: f64 = segments.iter().map(|s| s.length).sum();
        let avg_diameter: f64 = segments.iter().map(|s| (s.diameter_in + s.diameter_out) / 2.0).sum::<f64>() / segments.len() as f64;
        let avg_area = PI * avg_diameter * avg_diameter / 4.0;
        
        // Very simplified model - real implementation would use transmission line theory
        let wave_number = omega / c;
        let impedance_magnitude = rho * c / avg_area * (1.0 + (wave_number * total_length).powi(2)).sqrt();
        
        impedance_magnitude
    }
    
    /// Compute impedance spectrum
    pub fn compute_spectrum(&self) -> Vec<(f64, f64)> {
        let mut spectrum = Vec::new();
        let mut freq = self.min_freq;
        
        while freq <= self.max_freq {
            let impedance = self.compute_impedance(freq);
            spectrum.push((freq, impedance));
            freq += self.step_freq;
        }
        
        spectrum
    }
    
    /// Find simple peaks in spectrum
    pub fn find_peaks(&self) -> Vec<(f64, f64)> {
        let spectrum = self.compute_spectrum();
        let mut peaks = Vec::new();
        
        for i in 1..spectrum.len() - 1 {
            let (_, prev_imp) = spectrum[i - 1];
            let (freq, imp) = spectrum[i];
            let (_, next_imp) = spectrum[i + 1];
            
            // Simple peak detection
            if imp > prev_imp && imp > next_imp && imp > 1000.0 {
                peaks.push((freq, imp));
            }
        }
        
        peaks
    }
}

/// Simple evolutionary algorithm
pub struct SimpleEvolver {
    pub target_frequency: f64,
    pub population_size: usize,
    pub generations: usize,
}

impl SimpleEvolver {
    pub fn new(target_frequency: f64) -> Self {
        Self {
            target_frequency,
            population_size: 20,
            generations: 50,
        }
    }
    
    /// Simple optimization to match target frequency
    pub fn evolve(&self) -> SimpleGeometry {
        let mut best_geometry = SimpleGeometry::cone(1500.0, 32.0, 60.0, 20);
        let mut best_error = f64::INFINITY;
        
        // Simple parameter sweep instead of real evolution
        for length in (1000..2000).step_by(50) {
            for bell_d in (40..80).step_by(5) {
                let geometry = SimpleGeometry::cone(length as f64, 32.0, bell_d as f64, 20);
                let simulator = SimpleSimulator::new(geometry);
                let spectrum = simulator.compute_spectrum();
                
                // Find fundamental frequency (lowest peak)
                if let Some(&(freq, _)) = spectrum.first() {
                    let error = (freq - self.target_frequency).abs();
                    if error < best_error {
                        best_error = error;
                        best_geometry = simulator.geometry;
                    }
                }
            }
        }
        
        println!("Evolution completed. Best error: {:.2} Hz", best_error);
        best_geometry
    }
}

fn main() {
    println!("=== Simple CADSD Demo ===\n");
    
    // Example 1: Basic simulation
    println!("Example 1: Basic Acoustic Simulation");
    let geometry = SimpleGeometry::cone(1500.0, 32.0, 65.0, 25);
    println!("Geometry: Length = {:.0} mm, Bell = {:.0} mm", 
             geometry.length(), geometry.bell_diameter());
    
    let simulator = SimpleSimulator::new(geometry);
    let spectrum = simulator.compute_spectrum();
    println!("Computed {} frequency points", spectrum.len());
    
    let peaks = simulator.find_peaks();
    println!("Found {} peaks", peaks.len());
    for (i, (freq, imp)) in peaks.iter().enumerate() {
        println!("  Peak {}: {:.1} Hz (impedance: {:.0})", i + 1, freq, imp);
    }
    
    // Example 2: Simple optimization
    println!("\nExample 2: Simple Optimization");
    println!("Target frequency: 73.4 Hz (D note)");
    
    let evolver = SimpleEvolver::new(73.4);
    let optimized_geometry = evolver.evolve();
    
    println!("Optimized geometry:");
    println!("  Length: {:.0} mm", optimized_geometry.length());
    println!("  Bell diameter: {:.0} mm", optimized_geometry.bell_diameter());
    
    let optimized_sim = SimpleSimulator::new(optimized_geometry);
    let optimized_peaks = optimized_sim.find_peaks();
    if let Some((freq, imp)) = optimized_peaks.first() {
        println!("  Fundamental: {:.1} Hz (error: {:.1} Hz)", freq, (freq - 73.4).abs());
        println!("  Impedance: {:.0}", imp);
    }
    
    println!("\n=== Demo Complete ===");
}