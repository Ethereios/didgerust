//! FDTD-based validation utilities for acoustic simulations.
//! 
//! Provides 3-D finite-difference time-domain validation of TLM results.

mod validator;

pub use validator::{
    FDTDGrid,
    validate_fdtd_vs_tlm,
    generate_fdtd_validation_report,
};