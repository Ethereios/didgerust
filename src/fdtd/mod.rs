//! FDTD-based validation utilities for acoustic simulations.
//!
//! Provides 3-D finite-difference time-domain validation of TLM results.

mod validator;

pub use validator::{generate_fdtd_validation_report, validate_fdtd_vs_tlm, FDTDGrid};
