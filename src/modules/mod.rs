// filepath: hypr-notch/src/modules/mod.rs
//! Built-in modules for hypr-notch
//!
//! This module contains all the built-in modules that come with hypr-notch.

pub mod clock;

// Re-export all modules for convenience
pub use clock::ClockModule;
