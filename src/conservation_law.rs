//! Conservation-law integration (feature-gated).
//!
//! When the `conservation-law` feature is enabled, this module provides
//! wrappers around `conservation_law` primitives for use in the simulation
//! and optimizer loops.

#[cfg(feature = "conservation-law")]
pub mod symplectic;

#[cfg(feature = "conservation-law")]
pub use symplectic::SymplecticIntegratorWrapper;
