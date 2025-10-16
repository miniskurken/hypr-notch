// filepath: src/app.rs
//! Main application logic for hypr-notch

use crate::config::NotchConfig;
use crate::draw;
use crate::module::{ModuleEvent, ModuleRegistry};
use crate::modules::ClockModule;
use log::{debug, info, warn};
use smithay_client_toolkit::{
    compositor::CompositorState,
    compositor::Region,
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
use wayland_client::Connection;
use wayland_client::Proxy;

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
    module_registry: ModuleRegistry,
    input_region: Option<Region>,
    pub(crate) buffer_drawn: bool,
}

impl AppData {
    pub fn new(
        registry_state: RegistryState,
        output_state: OutputState,
        seat_state: SeatState,
        compositor_state: CompositorState,
        shm_state: Shm,
        layer_surface: LayerSurface,
        pool: SlotPool,
        config: NotchConfig,
        _connection: &Connection,
    ) -> Self {
        info!("Configuring layer surface");
        layer_surface.set_anchor(Anchor::TOP);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.set_size(config.collapsed_width, config.collapsed_height);
        layer_surface.set_exclusive_zone(-1);
        layer_surface.set_margin(0, 0, 0, 0);
        info!("Committing layer surface configuration");
        layer_surface.wl_surface().commit();

        let mut module_registry = ModuleRegistry::new();
        if let Err(err) = module_registry.load_modules_from_config(&config) {
            log::error!("Failed to load modules from config: {}", err);
        }
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
            input_region: None,
            buffer_drawn: false,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.configured
    }

    pub fn set_configured(&mut self, configured: bool) {
        self.configured = configured;
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("AppData::draw: drawing surface");

        if !self.configured {
            debug!("draw() called before surface is configured, skipping");
            return Ok(());
        }
        let now = Instant::now();
        if let Some(last_draw) = self.last_draw {
            if now.duration_since(last_draw) < Duration::from_millis(16) {
                return Ok(());
            }
        }
        self.last_draw = Some(now);
        info!("Drawing surface {}x{}", self.width, self.height);

        let width = self.width;
        let height = self.height;
        let stride = width * 4;

        let (buffer, canvas) = self.pool.create_buffer(
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Argb8888,
        )?;

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

        if self.expanded {
            let mut canvas_wrapper = draw::Canvas::new(canvas, width, height);
            self.module_registry.draw(&mut canvas_wrapper);
        }

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

    pub fn resize(&mut self, expand: bool) {
        if self.expanded == expand {
            return;
        }

        self.expanded = expand;

        if let Some(layer_surface) = &self.layer_surface {
            if expand {
                self.width = self.config.expanded_width;
                self.height = self.config.expanded_height;
                layer_surface.set_size(self.width, self.height);
                self.module_registry
                    .calculate_layout(self.width, self.height);
            } else {
                self.width = self.config.collapsed_width;
                self.height = self.config.collapsed_height;
                layer_surface.set_size(self.width, self.height);
            }
            layer_surface.wl_surface().commit();
            self.set_full_input_region(); // <-- Add this line
        }
    }

    pub fn set_full_input_region(&mut self) {
        if let Some(layer_surface) = &self.layer_surface {
            let surface = layer_surface.wl_surface();
            match Region::new(&self.compositor_state) {
                Ok(region) => {
                    region.add(0, 0, self.width as i32, self.height as i32);
                    surface.set_input_region(Some(region.wl_region()));
                    self.input_region = Some(region);
                    info!(
                        "Set input region to (0, 0, {}, {}) for surface {:?}",
                        self.width,
                        self.height,
                        surface.id()
                    );
                }
                Err(e) => {
                    warn!("Failed to create input region for notch surface: {e}");
                }
            }
        } else {
            warn!("set_full_input_region called but no layer_surface present");
        }
    }

    pub fn update_modules(&mut self) {
        if self.expanded {
            log::debug!("AppData::update_modules: sending UpdateExpanded");
            self.module_registry
                .handle_event(&ModuleEvent::UpdateExpanded);
        } else {
            log::debug!("AppData::update_modules: sending UpdateCollapsed");
            self.module_registry
                .handle_event(&ModuleEvent::UpdateCollapsed);
        }
    }

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
        info!("Pointer set: {:?}", self.pointer.is_some());
    }

    pub fn close_layer_surface(&mut self) {
        self.layer_surface = None;
        info!("Layer surface closed");
    }
}
