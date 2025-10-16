//! Main entry point for hypr-notch
//!
//! This file contains the main function that initializes the application,
//! connects to the Wayland server, and starts the event loop.
//! It ties together all the other modules but keeps minimal logic itself.

mod app;
mod config;
mod draw;
mod wayland;

use app::AppData;
use config::NotchConfig;
use log::{info, warn};
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::wlr_layer::{Layer, LayerShell},
    shm::{slot::SlotPool, Shm},
};
use wayland_client::{globals::registry_queue_init, Connection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::init();
    info!("Starting hypr-notch");

    // Load configuration
    let config = NotchConfig::load_from_file().unwrap_or_default();
    info!("Configuration loaded");
    // Connect to Wayland server
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    // Bind to global objects
    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    let seat_state = SeatState::new(&globals, &qh);

    // Create buffer pool for drawing
    let pool_size = (config.expanded_width * config.expanded_height * 4) as usize;
    let pool = SlotPool::new(pool_size, &shm)?;

    // Create surface and layer surface
    let surface = compositor.create_surface(&qh);
    let layer_surface =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("hypr-notch"), None);

    // Configure and initialize the app
    let mut app_data = AppData::new(
        RegistryState::new(&globals),
        OutputState::new(&globals, &qh),
        seat_state,
        compositor,
        shm,
        layer_surface,
        pool,
        config,
    );

    // Event loop
    loop {
        event_queue.blocking_dispatch(&mut app_data)?;

        if app_data.is_configured() {
            app_data.draw()?;
        }
    }
}
