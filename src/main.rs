// filepath: src/main.rs

mod app;
mod config;
mod draw;
mod module;
mod modules;
mod pointer;
mod wayland;

use app::AppData;
use calloop::{
    timer::{TimeoutAction, Timer},
    EventLoop, Interest, Mode, PostAction,
};
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
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use wayland_client::{globals::registry_queue_init, Connection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    info!("Starting hypr-notch");

    let config = NotchConfig::load_from_file().unwrap_or_default();
    info!("Configuration loaded");

    let conn = Connection::connect_to_env()?;
    let backend = conn.backend();
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
        layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("hypr-notch"), None);

    let app_data = Rc::new(RefCell::new(AppData::new(
        RegistryState::new(&globals),
        OutputState::new(&globals, &qh),
        seat_state,
        compositor,
        shm,
        layer_surface,
        pool,
        config,
        &conn,
    )));

    info!("Performing initial round-trip");
    event_queue.roundtrip(&mut *app_data.borrow_mut())?;

    info!("Entering main loop");

    let mut event_loop: EventLoop<()> = EventLoop::try_new().expect("Failed to create event loop");

    let event_queue = Rc::new(RefCell::new(event_queue));

    let app_data_wl = app_data.clone();
    let event_queue_fd = event_queue.clone();
    let wayland_fd = backend.poll_fd();

    event_loop
        .handle()
        .insert_source(
            calloop::generic::Generic::new(wayland_fd, Interest::READ, Mode::Level),
            move |_, _, _| {
                let mut eq = event_queue_fd.borrow_mut();
                let _ = eq.dispatch_pending(&mut *app_data_wl.borrow_mut());
                let _ = eq.flush();
                Ok(PostAction::Continue)
            },
        )
        .expect("Failed to insert Wayland source");

    let app_data_timer = app_data.clone();
    let event_queue_timer = event_queue.clone();
    let wayland_timer = Timer::from_duration(Duration::from_millis(10));
    event_loop
        .handle()
        .insert_source(wayland_timer, move |_, _, _| {
            let mut eq = event_queue_timer.borrow_mut();
            let _ = eq.dispatch_pending(&mut *app_data_timer.borrow_mut());
            let _ = eq.flush();
            TimeoutAction::ToDuration(Duration::from_millis(10))
        })
        .expect("Failed to insert Wayland periodic timer");

    let app_data_update = app_data.clone();
    let update_timer = Timer::from_duration(Duration::from_secs(1));
    event_loop
        .handle()
        .insert_source(update_timer, move |_, _, _| {
            info!("Updating modules (timer)");
            let mut app_data = app_data_update.borrow_mut();
            app_data.update_modules();
            if app_data.is_configured() {
                info!("Redrawing surface (timer)");
                let _ = app_data.draw();
            }
            TimeoutAction::ToDuration(Duration::from_secs(1))
        })
        .expect("Failed to insert update timer");

    event_loop
        .run(None, &mut (), |_| {})
        .expect("Event loop failed");

    info!("Exiting hypr-notch");
    Ok(())
}
