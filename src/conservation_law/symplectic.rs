//! Symplectic integration wrapper for DidgeRust.
//!
//! Provides energy-conserving integration for long-time wave simulations.

use conservation_law::lagrangian::{AgentState, Lagrangian, MechanicalLagrangian, SymplecticIntegrator};

/// Wrapper around `conservation_law::SymplecticIntegrator` for 1D wave segments.
pub struct SymplecticIntegratorWrapper {
    dt: f64,
}

impl SymplecticIntegratorWrapper {
    /// Create a new symplectic integrator with the given timestep.
    pub fn new(dt: f64) -> Result<Self, String> {
        if dt <= 0.0 {
            return Err("timestep must be positive".to_string());
        }
        Ok(Self { dt })
    }

    /// Perform one Verlet step for a harmonic oscillator with mass `m`
    /// and stiffness `k`, returning the new state.
    pub fn step_harmonic(&self, m: f64, k: f64, state: &AgentState<f64, 1>) -> Result<AgentState<f64, 1>, String> {
        let integrator = SymplecticIntegrator::new(self.dt).map_err(|e| format!("{:?}", e))?;
        integrator
            .step(m, &|q: &[f64; 1]| 0.5 * k * q[0] * q[0], state)
            .map_err(|e| format!("{:?}", e))
    }

    /// Integrate a harmonic oscillator for `steps` timesteps.
    pub fn integrate_harmonic(
        &self,
        m: f64,
        k: f64,
        initial: &AgentState<f64, 1>,
        steps: usize,
    ) -> Result<Vec<AgentState<f64, 1>>, String> {
        let integrator = SymplecticIntegrator::new(self.dt).map_err(|e| format!("{:?}", e))?;
        integrator
            .integrate(m, &|q: &[f64; 1]| 0.5 * k * q[0] * q[0], initial, steps)
            .map_err(|e| format!("{:?}", e))
    }

    /// Compute total mechanical energy for a harmonic oscillator.
    pub fn total_energy(&self, m: f64, k: f64, state: &AgentState<f64, 1>) -> f64 {
        let lagrangian = MechanicalLagrangian {
            mass: m,
            potential_fn: |q: &[f64; 1]| 0.5 * k * q[0] * q[0],
        };
        lagrangian.kinetic(state) + lagrangian.potential(state)
    }
}
