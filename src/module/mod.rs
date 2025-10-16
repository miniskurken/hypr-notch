// filepath: hypr-notch/src/module/mod.rs
//! Module system for hypr-notch
//!
//! This is the main entry point for the module system.
//! It re-exports the core traits and types needed to create and manage modules.

pub mod interface;
mod registry;

pub use interface::{Module, ModuleEvent, Rect};
pub use registry::ModuleRegistry;

// Remove this unused import
// pub use crate::draw::Canvas;
