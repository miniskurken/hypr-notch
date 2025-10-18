pub mod config;
pub mod draw;
pub mod layout;
pub mod module;
pub mod modules;

// Re-export for plugin authors
pub use crate::draw::Canvas;
pub use crate::module::interface::{Module, ModuleCreateFn, ModuleEvent, Rect};
