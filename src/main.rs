// filepath: src/main.rs

mod app;
mod config;
mod draw;
mod module;
mod modules;
mod pointer;
mod wayland;
use std::thread;

use app::AppData;
use config::NotchConfig;
use log::info;
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::wlr_layer::{Layer, LayerShell},
    shm::{slot::SlotPool, Shm},
};
use std::time::{Duration, Instant};
use wayland_client::{globals::registry_queue_init, Connection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    info!("Starting hypr-notch (minimal modular)");

    let config = NotchConfig::load_from_file().unwrap_or_default();
    info!("Configuration loaded");

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    let seat_state = SeatState::new(&globals, &qh);

    let pool_size = (config.expanded_width * config.expanded_height * 4) as usize;
    let pool = SlotPool::new(pool_size, &shm)?;

    let surface = compositor.create_surface(&qh);
    let layer_surface =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("hypr-notch"), None);

    let mut app_data = AppData::new(
        RegistryState::new(&globals),
        OutputState::new(&globals, &qh),
        seat_state,
        compositor,
        shm,
        layer_surface,
        pool,
        config,
        &conn,
    );

    info!("Performing initial round-trip");
    event_queue.roundtrip(&mut app_data)?;

    info!("Entering event loop");
    let mut last_update = Instant::now();
    loop {
        // Process any pending events (non-blocking)
        // This line cause the problems with the mouse events not being handled / no timer updates.
        event_queue.dispatch_pending(&mut app_data)?;

        // Sleep for a short time to avoid busy-waiting
        thread::sleep(Duration::from_millis(50));

        // Periodic update every second
        if last_update.elapsed() >= Duration::from_secs(1) {
            log::debug!("Main loop: triggering update_modules()");
            app_data.update_modules();
            if app_data.is_configured() && app_data.buffer_drawn && app_data.expanded {
                log::debug!("Main loop: calling draw()");
                let _ = app_data.draw();
            }
            last_update = Instant::now();
        }
    }
}
