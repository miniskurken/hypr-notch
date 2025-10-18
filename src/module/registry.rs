// filepath: hypr-notch/src/module/registry.rs
//! Module registry for managing modules
//!
//! This file implements the ModuleRegistry that manages the loading,
//! layout, and rendering of modules.

use crate::layout::calculate_module_layout;
use log::info;
use std::collections::HashMap;

use crate::draw::Canvas;
use crate::module::interface::ModuleCreateFn;
use crate::module::{Module, ModuleEvent, Rect};

use libloading::{Library, Symbol};

/// Manages the collection of loaded modules
pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
    module_areas: HashMap<String, Rect>,
    external_libs: Vec<libloading::Library>,
}

impl ModuleRegistry {
    /// Create a new empty module registry
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            module_areas: HashMap::new(),
            external_libs: Vec::new(),
        }
    }

    /// Add a module to the registry
    pub fn add_module(&mut self, module: Box<dyn Module>) {
        info!("Adding module: {}", module.name());
        self.modules.push(module);
    }

    fn load_external_module(&mut self, path: &str) -> Option<Box<dyn Module>> {
        unsafe {
            let lib = Library::new(path).ok()?;
            let func: Symbol<ModuleCreateFn> = lib.get(b"create_module").ok()?;
            let boxed = func();
            self.external_libs.push(lib); // Keep the library alive!
            Some(Box::from_raw(boxed))
        }
    }

    /// Load modules based on configuration
    pub fn load_modules_from_config(
        &mut self,
        config: &crate::config::NotchConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let enabled = &config.modules.enabled;

        // Remove modules not enabled
        self.modules
            .retain(|m| enabled.contains(&m.id().to_string()));

        // Add enabled modules that are missing
        for module_id in enabled {
            if !self.modules.iter().any(|m| m.id() == module_id) {
                if let Some(path) = config.modules.aliases.get(module_id) {
                    // Load external module using alias path
                    match self.load_external_module(path) {
                        Some(module) => self.add_module(module),
                        None => log::warn!("Failed to load external module: {}", path),
                    }
                } else if let Some(path) = module_id.strip_prefix("external:") {
                    // Legacy: support external: prefix
                    match self.load_external_module(path) {
                        Some(module) => self.add_module(module),
                        None => log::warn!("Failed to load external module: {}", path),
                    }
                } else {
                    match module_id.as_str() {
                        "clock" => self.add_module(Box::new(crate::modules::ClockModule::new())),
                        // Add other built-in modules here as needed
                        _ => log::warn!("Unknown module: {}", module_id),
                    }
                }
            }
        }

        // Initialize enabled modules with their config
        for module in &mut self.modules {
            if let Some(cfg) = config.modules.module_configs.get(module.id()) {
                module.init(cfg)?;
            }
        }
        Ok(())
    }

    /// Draw all modules to the canvas
    pub fn draw(&mut self, canvas: &mut Canvas) {
        // Only draw modules that have an assigned area
        for module in &self.modules {
            if let Some(area) = self.module_areas.get(module.id()) {
                if let Err(e) = module.draw(canvas, *area) {
                    log::error!("Error drawing module {}: {}", module.name(), e);
                }
            }
        }
    }

    /// Send an event to the appropriate module
    pub fn handle_event(&mut self, event: &ModuleEvent) -> bool {
        // For Enter/Motion/Press events, find which module contains the point
        log::debug!("ModuleRegistry::handle_event: received event {:?}", event);

        match event {
            ModuleEvent::Enter { x, y }
            | ModuleEvent::Motion { x, y }
            | ModuleEvent::Press { x, y, .. }
            | ModuleEvent::Release { x, y, .. } => {
                // Find module that contains this point
                for module in &mut self.modules {
                    if let Some(area) = self.module_areas.get(module.id()) {
                        if *x >= area.x as f64
                            && *y >= area.y as f64
                            && *x < (area.x + area.width as i32) as f64
                            && *y < (area.y + area.height as i32) as f64
                        {
                            // Point is within this module's area
                            return module.handle_event(event, *area);
                        }
                    }
                }
            }

            // For other events, send to all modules
            _ => {
                for module in &mut self.modules {
                    if let Some(area) = self.module_areas.get(module.id()) {
                        if module.handle_event(event, *area) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Update the Canvas structure to prepare for module implementation
    pub fn has_modules(&self) -> bool {
        !self.modules.is_empty()
    }

    pub fn calculate_layout(&mut self, config: &crate::config::NotchConfig, expanded: bool) {
        let layout = calculate_module_layout(config, &self.modules, expanded);
        self.module_areas = layout.areas;
    }
}
