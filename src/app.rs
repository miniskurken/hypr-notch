// filepath: hypr-notch/src/app.rs
//! Main application logic for hypr-notch
//!
//! This file contains the AppData struct which holds the application state
//! and implements the core application logic, including initialization,
//! state management, and the main drawing routine.

use crate::config::NotchConfig;
use crate::draw;
use crate::module::{ModuleEvent, ModuleRegistry};
use crate::modules::ClockModule;
use log::info;
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, LayerSurface},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm},
};
use std::time::{Duration, Instant};
use wayland_client::protocol::{wl_pointer, wl_shm};

pub struct AppData {
    registry_state: RegistryState,
    output_state: OutputState,
    seat_state: SeatState,
    compositor_state: CompositorState,
    shm_state: Shm,
    layer_surface: Option<LayerSurface>,
    pool: SlotPool,
    pub(crate) width: u32,
    pub(crate) height: u32,
    configured: bool,
    pub(crate) expanded: bool,
    pointer: Option<wl_pointer::WlPointer>,
    config: NotchConfig,
    last_draw: Option<Instant>,
    // Add module registry
    module_registry: ModuleRegistry,
}

impl AppData {
    /// Create a new AppData instance
    pub fn new(
        registry_state: RegistryState,
        output_state: OutputState,
        seat_state: SeatState,
        compositor_state: CompositorState,
        shm_state: Shm,
        layer_surface: LayerSurface,
        pool: SlotPool,
        config: NotchConfig,
    ) -> Self {
        // Configure the layer surface
        layer_surface.set_anchor(Anchor::TOP);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.set_size(config.collapsed_width, config.collapsed_height);
        layer_surface.set_exclusive_zone(-1); // Don't reserve space
        layer_surface.set_margin(0, 0, 0, 0);
        layer_surface.wl_surface().commit();

        // Create and initialize the module registry
        let mut module_registry = ModuleRegistry::new();

        // Initialize modules based on configuration
        if let Err(err) = module_registry.load_modules_from_config(&config) {
            log::error!("Failed to load modules from config: {}", err);
        }

        // Add a clock module by default if none configured
        if !module_registry.has_modules() {
            info!("No modules configured, adding default clock module");
            module_registry.add_module(Box::new(ClockModule::new()));
        }

        Self {
            registry_state,
            output_state,
            seat_state,
            compositor_state,
            shm_state,
            layer_surface: Some(layer_surface),
            pool,
            width: config.collapsed_width,
            height: config.collapsed_height,
            configured: false,
            expanded: false,
            pointer: None,
            config,
            last_draw: None,
            module_registry,
        }
    }

    /// Check if the surface has been configured
    pub fn is_configured(&self) -> bool {
        self.configured
    }

    /// Set configured state
    pub fn set_configured(&mut self, configured: bool) {
        self.configured = configured;
    }

    /// Update the surface size based on its expanded state
    pub fn update_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Draw the notch surface
    pub fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Limit drawing to avoid excessive redraws
        let now = Instant::now();
        if let Some(last_draw) = self.last_draw {
            if now.duration_since(last_draw) < Duration::from_millis(16) {
                // ~60fps cap
                return Ok(());
            }
        }
        self.last_draw = Some(now);

        let width = self.width;
        let height = self.height;
        let stride = width * 4;

        let (buffer, canvas) = self.pool.create_buffer(
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Argb8888,
        )?;

        // Draw the background with rounded corners
        let expanded = self.expanded;
        let corner_radius = self.config.corner_radius;
        let color = self.config.background_color;

        draw::fill_canvas_with_rounded_corners(
            canvas,
            width,
            height,
            expanded,
            corner_radius,
            color,
        );

        // If expanded, draw modules
        if self.expanded {
            // Create a canvas abstraction for modules to draw on
            let mut canvas_wrapper = draw::Canvas::new(canvas, width, height);

            // Draw all modules
            self.module_registry.draw(&mut canvas_wrapper);
        }

        // Attach buffer to surface and commit
        if let Some(layer_surface) = &self.layer_surface {
            buffer
                .attach_to(layer_surface.wl_surface())
                .expect("buffer attach");
            layer_surface
                .wl_surface()
                .damage_buffer(0, 0, width as i32, height as i32);
            layer_surface.wl_surface().commit();
        }

        Ok(())
    }

    /// Resize the notch based on expanded state
    pub fn resize(&mut self, expand: bool) {
        if self.expanded == expand {
            return;
        }

        self.expanded = expand;

        if expand {
            info!(
                "Notch expanding to {}x{}",
                self.config.expanded_width, self.config.expanded_height
            );
        } else {
            info!(
                "Notch collapsing to {}x{}",
                self.config.collapsed_width, self.config.collapsed_height
            );
        }

        if let Some(layer_surface) = &self.layer_surface {
            if expand {
                self.width = self.config.expanded_width;
                self.height = self.config.expanded_height;
                layer_surface.set_size(self.width, self.height);

                // Recalculate module layout when expanding
                self.module_registry
                    .calculate_layout(self.width, self.height);
            } else {
                self.width = self.config.collapsed_width;
                self.height = self.config.collapsed_height;
                layer_surface.set_size(self.width, self.height);
            }
            layer_surface.wl_surface().commit();
        }
    }

    /// Send update event to all modules
    pub fn update_modules(&mut self) {
        self.module_registry.handle_event(&ModuleEvent::Update);
    }

    // Accessors for Wayland handlers
    pub fn registry_state(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    pub fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    pub fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    pub fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }

    pub fn set_pointer(&mut self, pointer: Option<wl_pointer::WlPointer>) {
        self.pointer = pointer;
    }

    pub fn close_layer_surface(&mut self) {
        self.layer_surface = None;
    }
}
