// filepath: src/main.rs
mod app;
mod config;
mod draw;
mod module;
mod modules;
mod pointer;
mod wayland;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use app::AppData;
use calloop::{timer::TimeoutAction, timer::Timer, EventLoop};
use calloop_wayland_source::WaylandSource;
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
use wayland_client::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    info!("Starting hypr-notch (minimal modular)");

    let config = NotchConfig::load_from_file().unwrap_or_default();
    info!("Configuration loaded");

    // Set up Wayland connection and event queue
    let conn = Connection::connect_to_env()?;

    // Use AppData as the state type for registry_queue_init
    let (global_list, event_queue) =
        wayland_client::globals::registry_queue_init::<AppData>(&conn)?;
    let qh = event_queue.handle();

    // Create the registry state from the global list
    let registry_state = RegistryState::new(&global_list);

    // Use &global_list and &qh for all SCTK bindings
    let compositor = CompositorState::bind(&global_list, &qh)?;
    let layer_shell = LayerShell::bind(&global_list, &qh)?;
    let shm = Shm::bind(&global_list, &qh)?;
    let seat_state = SeatState::new(&global_list, &qh);

    let pool_size = (config.expanded_width * config.expanded_height * 4) as usize;
    let pool = SlotPool::new(pool_size, &shm)?;

    let surface = compositor.create_surface(&qh);
    let layer_surface =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("hypr-notch"), None);

    // Now create your AppData instance
    let app_data = Rc::new(RefCell::new(AppData::new(
        registry_state,
        OutputState::new(&global_list, &qh),
        seat_state,
        compositor,
        shm,
        layer_surface,
        pool,
        config,
        &conn,
    )));
    let mut event_loop = EventLoop::try_new()?;

    // Register Wayland event queue as a source
    {
        let app_data = app_data.clone();
        event_loop.handle().insert_source(
            WaylandSource::new(conn.clone(), event_queue),
            move |_, queue, _| {
                let mut app = app_data.borrow_mut();
                // This callback processes all pending Wayland events
                queue.dispatch_pending(&mut *app).unwrap();
                Ok(0)
            },
        )?;
    }

    // Register a timer for periodic updates
    {
        let app_data = app_data.clone();
        let timer = Timer::from_duration(Duration::from_secs(1));
        event_loop.handle().insert_source(timer, move |_, _, _| {
            let mut app = app_data.borrow_mut();
            app.update_modules();
            if app.is_configured() && app.buffer_drawn && app.expanded {
                let _ = app.draw();
            }
            TimeoutAction::ToDuration(Duration::from_secs(1))
        })?;
    }

    info!("Entering event loop");
    event_loop.run(None, &mut (), |_| {})?;

    Ok(())
}
