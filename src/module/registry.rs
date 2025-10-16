// filepath: hypr-notch/src/module/registry.rs
//! Module registry for managing modules
//!
//! This file implements the ModuleRegistry that manages the loading,
//! layout, and rendering of modules.

use log::{error, info};
use std::collections::HashMap;

use crate::config::NotchConfig;
use crate::draw::Canvas;
use crate::module::{Module, ModuleEvent, Rect};

/// Manages the collection of loaded modules
pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
    module_areas: HashMap<String, Rect>,
}

impl ModuleRegistry {
    /// Create a new empty module registry
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            module_areas: HashMap::new(),
        }
    }

    /// Add a module to the registry
    pub fn add_module(&mut self, module: Box<dyn Module>) {
        info!("Adding module: {}", module.name());
        self.modules.push(module);
    }

    /// Load modules based on configuration
    pub fn load_modules_from_config(
        &mut self,
        _config: &NotchConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Will implement later when we have modules to load
        Ok(())
    }

    /// Calculate the layout of all modules based on available space
    pub fn calculate_layout(&mut self, total_width: u32, _total_height: u32) {
        // Simple layout: stack modules vertically with margins
        let margin = 10i32; // Change to i32
        let mut y_offset = margin;

        for module in &self.modules {
            let (width, height) = module.preferred_size();
            let width = width.min(total_width - 2 * margin as u32);

            self.module_areas.insert(
                module.id().to_string(),
                Rect {
                    x: margin,
                    y: y_offset,
                    width,
                    height,
                },
            );

            y_offset += height as i32 + margin; // Now both are i32
        }
    }

    /// Draw all modules to the canvas
    pub fn draw(&mut self, canvas: &mut Canvas) {
        // Calculate layout if not already done
        if self.module_areas.is_empty() && !self.modules.is_empty() {
            self.calculate_layout(canvas.width(), canvas.height());
        }

        // Draw each module in its area
        for module in &self.modules {
            if let Some(area) = self.module_areas.get(module.id()) {
                if let Err(e) = module.draw(canvas, *area) {
                    error!("Error drawing module {}: {}", module.name(), e);
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
}
