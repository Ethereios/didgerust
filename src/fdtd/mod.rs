//! 3-D Finite-Difference Time-Domain (FDTD) acoustic solver.
//!
//! This module implements a Yee staggered-grid FDTD scheme for the 3-D
//! acoustic wave equation. It is adapted from the `fdtd-waveguide` EM solver
//! (Purdue CEM) by replacing E/H fields with pressure/velocity.
//!
//! # References
//!
//! - Taflove & Hagness, *Computational Electrodynamics: The Finite-Difference
//!   Time-Domain Method* (2005) — Yee algorithm, stability, dispersion.
//! - Wang (MIT 2019) — 3-D FDTD for wind instrument acoustics.

use std::f64::consts::PI;

/// Acoustic constants for FDTD simulation
#[derive(Debug, Clone, Copy)]
pub struct AcousticMedium {
    pub density: f64,
    pub sound_speed: f64,
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
    pub dt: f64,
}

impl AcousticMedium {
    pub fn new(density: f64, sound_speed: f64, dx: f64, dy: f64, dz: f64) -> Self {
        let dt = 0.5 * (1.0 / (sound_speed * (1.0 / dx.powi(2) + 1.0 / dy.powi(2) + 1.0 / dz.powi(2)).sqrt()));
        Self {
            density,
            sound_speed,
            dx,
            dy,
            dz,
            dt,
        }
    }

    pub fn cfl(&self) -> f64 {
        self.sound_speed * self.dt * (1.0 / self.dx.powi(2) + 1.0 / self.dy.powi(2) + 1.0 / self.dz.powi(2)).sqrt()
    }
}

/// 3-D FDTD acoustic solver on a Yee staggered grid.
///
/// Pressure is stored at cell centers, velocity components at cell faces.
pub struct AcousticFDTD3D {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub medium: AcousticMedium,
    pub pressure: Vec<f64>,
    pub velocity_x: Vec<f64>,
    pub velocity_y: Vec<f64>,
    pub velocity_z: Vec<f64>,
    pub mask: Vec<u8>,
    pub step_count: usize,
}

impl AcousticFDTD3D {
    pub fn new(nx: usize, ny: usize, nz: usize, medium: AcousticMedium) -> Self {
        let size = nx * ny * nz;
        Self {
            nx,
            ny,
            nz,
            medium,
            pressure: vec![0.0; size],
            velocity_x: vec![0.0; size],
            velocity_y: vec![0.0; size],
            velocity_z: vec![0.0; size],
            mask: vec![1; size],
            step_count: 0,
        }
    }

    pub fn set_solid(&mut self, x: usize, y: usize, z: usize) {
        if x < self.nx && y < self.ny && z < self.nz {
            let idx = x + y * self.nx + z * self.nx * self.ny;
            self.mask[idx] = 0;
        }
    }

    pub fn set_source(&mut self, x: usize, y: usize, z: usize, amplitude: f64) {
        if x < self.nx && y < self.ny && z < self.nz {
            let idx = self.idx(x, y, z);
            if self.mask[idx] != 0 {
                self.pressure[idx] += amplitude;
            }
        }
    }

    fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.nx + z * self.nx * self.ny
    }

    pub fn step(&mut self) {
        let dx = self.medium.dx;
        let dy = self.medium.dy;
        let dz = self.medium.dz;
        let dt = self.medium.dt;
        let rho = self.medium.density;
        let c = self.medium.sound_speed;
        let inv_dx = 1.0 / dx;
        let inv_dy = 1.0 / dy;
        let inv_dz = 1.0 / dz;

        let mut vx_new = self.velocity_x.clone();
        let mut vy_new = self.velocity_y.clone();
        let mut vz_new = self.velocity_z.clone();
        let mut p_new = self.pressure.clone();

        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let idx = self.idx(x, y, z);
                    if self.mask[idx] == 0 {
                        continue;
                    }

                    let mask = self.mask[idx];

                    let dp_dx = if x + 1 < self.nx && self.mask[self.idx(x + 1, y, z)] != 0 {
                        (self.pressure[self.idx(x + 1, y, z)] - self.pressure[idx]) * inv_dx
                    } else {
                        0.0
                    };

                    let dp_dy = if y + 1 < self.ny && self.mask[self.idx(x, y + 1, z)] != 0 {
                        (self.pressure[self.idx(x, y + 1, z)] - self.pressure[idx]) * inv_dy
                    } else {
                        0.0
                    };

                    let dp_dz = if z + 1 < self.nz && self.mask[self.idx(x, y, z + 1)] != 0 {
                        (self.pressure[self.idx(x, y, z + 1)] - self.pressure[idx]) * inv_dz
                    } else {
                        0.0
                    };

                    if mask != 0 {
                        vx_new[idx] = self.velocity_x[idx] - dt / rho * dp_dx;
                        vy_new[idx] = self.velocity_y[idx] - dt / rho * dp_dy;
                        vz_new[idx] = self.velocity_z[idx] - dt / rho * dp_dz;
                    }
                }
            }
        }

        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    let idx = self.idx(x, y, z);
                    if self.mask[idx] == 0 {
                        continue;
                    }

                    let dvx_dx = if x > 0 && self.mask[self.idx(x - 1, y, z)] != 0 {
                        (vx_new[idx] - vx_new[self.idx(x - 1, y, z)]) * inv_dx
                    } else {
                        0.0
                    };

                    let dvy_dy = if y > 0 && self.mask[self.idx(x, y - 1, z)] != 0 {
                        (vy_new[idx] - vy_new[self.idx(x, y - 1, z)]) * inv_dy
                    } else {
                        0.0
                    };

                    let dvz_dz = if z > 0 && self.mask[self.idx(x, y, z - 1)] != 0 {
                        (vz_new[idx] - vz_new[self.idx(x, y, z - 1)]) * inv_dz
                    } else {
                        0.0
                    };

                    p_new[idx] = self.pressure[idx] - rho * c * c * dt * (dvx_dx + dvy_dy + dvz_dz);
                }
            }
        }

        for idx in 0..self.pressure.len() {
            if self.mask[idx] == 0 {
                p_new[idx] = 0.0;
                vx_new[idx] = 0.0;
                vy_new[idx] = 0.0;
                vz_new[idx] = 0.0;
            }
        }

        self.velocity_x = vx_new;
        self.velocity_y = vy_new;
        self.velocity_z = vz_new;
        self.pressure = p_new;
        self.step_count += 1;
    }

    pub fn run(&mut self, n_steps: usize) {
        for _ in 0..n_steps {
            self.step();
        }
    }

    pub fn pressure_at(&self, x: usize, y: usize, z: usize) -> f64 {
        if x < self.nx && y < self.ny && z < self.nz {
            self.pressure[self.idx(x, y, z)]
        } else {
            0.0
        }
    }

    pub fn spectrum_at(&self, _x: usize, y: usize, _z: usize) -> Vec<f64> {
        let signal: Vec<f64> = self.pressure.chunks_exact(self.nx * self.ny)
            .map(|slice| slice[self.idx(0, y, 0)])
            .collect();

        let n = signal.len();
        let mut spectrum = vec![0.0; n / 2];
        for (k, spec) in spectrum.iter_mut().enumerate() {
            let mut re = 0.0;
            let mut im = 0.0;
            for (n_i, &val) in signal.iter().enumerate() {
                let angle = 2.0 * PI * k as f64 * n_i as f64 / n as f64;
                re += val * angle.cos();
                im -= val * angle.sin();
            }
            *spec = (re * re + im * im).sqrt() / n as f64;
        }
        spectrum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdtd_creation() {
        let medium = AcousticMedium::new(1.225, 343.0, 0.01, 0.01, 0.01);
        let mut fdtd = AcousticFDTD3D::new(10, 10, 10, medium);
        assert!(fdtd.medium.cfl() < 1.0, "CFL condition must be satisfied");
        fdtd.set_source(5, 5, 5, 1.0);
        fdtd.run(10);
        assert_eq!(fdtd.step_count, 10);
    }

    #[test]
    fn test_fdtd_solid_mask() {
        let medium = AcousticMedium::new(1.225, 343.0, 0.01, 0.01, 0.01);
        let mut fdtd = AcousticFDTD3D::new(10, 10, 10, medium);
        fdtd.set_solid(5, 5, 5);
        fdtd.set_source(5, 5, 5, 1.0);
        fdtd.run(5);
        assert_eq!(fdtd.pressure_at(5, 5, 5), 0.0);
    }
}
