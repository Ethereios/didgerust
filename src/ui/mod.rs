//! UI module for CADSD GUI
//! Re-exports from crate::app when gui-bevy feature is enabled

#[cfg(feature = "gui-bevy")]
use bevy::prelude::Resource;

#[cfg(feature = "gui-bevy")]
pub use crate::app::CadsdState;