// filepath: src/main.rs

use log::{debug, info};
use smithay_client_toolkit::{
    compositor::CompositorState,
    compositor::Region,
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm,
    output::OutputState,
    registry::RegistryState,
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::wlr_layer::{
        Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
        LayerSurfaceConfigure,
    },
    shell::WaylandSurface,
    shm::{slot::SlotPool, Shm},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};

struct State {
    compositor_state: CompositorState,
    seat_state: SeatState,
    registry_state: RegistryState,
    output_state: OutputState,
    shm_state: Shm,
    layer_surface: Option<LayerSurface>,
    pointer: Option<wl_pointer::WlPointer>,
    input_region: Option<Region>,
    _pool: Option<SlotPool>, // Keep pool alive for buffer
    configured: bool,
}

impl State {
    fn set_full_input_region(&mut self) {
        if let Some(layer_surface) = &self.layer_surface {
            let surface = layer_surface.wl_surface();
            match Region::new(&self.compositor_state) {
                Ok(region) => {
                    region.add(0, 0, 800, 200);
                    surface.set_input_region(Some(region.wl_region()));
                    self.input_region = Some(region);
                    info!("Set input region to (0, 0, 800, 200)");
                }
                Err(e) => {
                    info!("Failed to create input region: {e}");
                }
            }
        }
    }

    fn draw_buffer(&mut self) {
        if let Some(layer_surface) = &self.layer_surface {
            let shm = &self.shm_state;
            let mut pool = SlotPool::new(800 * 200 * 4, shm).expect("Failed to create pool");
            let (buffer, canvas) = pool
                .create_buffer(800, 200, 800 * 4, wl_shm::Format::Argb8888)
                .expect("Failed to create buffer");
            for pixel in canvas.chunks_exact_mut(4) {
                pixel.copy_from_slice(&[0x22, 0x88, 0xff, 0xff]); // Opaque blue
            }
            buffer
                .attach_to(layer_surface.wl_surface())
                .expect("Failed to attach buffer");
            layer_surface.wl_surface().damage_buffer(0, 0, 800, 200);
            layer_surface.wl_surface().commit();
            self._pool = Some(pool); // Keep pool alive
            info!("Buffer drawn and attached");
        }
    }
}

impl LayerShellHandler for State {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {}
    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        info!("LayerShellHandler: configure: {:?}", configure.new_size);
        if !self.configured {
            self.set_full_input_region();
            self.draw_buffer();
            self.configured = true;
        }
    }
}

impl SeatHandler for State {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}
    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        info!("SeatHandler: new_capability: {:?}", capability);
        if capability == Capability::Pointer {
            let pointer = self.seat_state.get_pointer(qh, &seat).ok();
            self.pointer = pointer;
            info!("Pointer set: {:?}", self.pointer.is_some());
        }
    }
    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        info!("SeatHandler: remove_capability: {:?}", capability);
        if capability == Capability::Pointer {
            self.pointer = None;
        }
    }
    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {
    }
}

impl PointerHandler for State {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        info!(
            "PointerHandler: pointer_frame called with {} events",
            events.len()
        );
        for event in events {
            match event.kind {
                PointerEventKind::Enter { .. } => info!("Pointer ENTER at {:?}", event.position),
                PointerEventKind::Leave { .. } => info!("Pointer LEAVE"),
                PointerEventKind::Motion { .. } => debug!("Pointer MOTION at {:?}", event.position),
                _ => {}
            }
        }
    }
}

impl smithay_client_toolkit::compositor::CompositorHandler for State {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }
}
impl smithay_client_toolkit::output::OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
}
impl smithay_client_toolkit::shm::ShmHandler for State {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}
impl smithay_client_toolkit::registry::ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

delegate_compositor!(State);
delegate_output!(State);
delegate_shm!(State);
delegate_layer!(State);
delegate_seat!(State);
delegate_pointer!(State);
delegate_registry!(State);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    info!("Starting minimal pointer test");

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    let seat_state = SeatState::new(&globals, &qh);

    let surface = compositor.create_surface(&qh);
    let layer_surface = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Top, // Try Layer::Overlay if you want
        Some("pointer-test"),
        None,
    );
    layer_surface.set_anchor(Anchor::TOP);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer_surface.set_size(800, 200);
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_margin(0, 0, 0, 0);
    layer_surface.wl_surface().commit();

    let mut state = State {
        compositor_state: compositor,
        seat_state,
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm_state: shm,
        layer_surface: Some(layer_surface),
        pointer: None,
        input_region: None,
        _pool: None,
        configured: false,
    };

    info!("Initial roundtrip");
    event_queue.roundtrip(&mut state)?;

    info!("Entering event loop");
    loop {
        event_queue.blocking_dispatch(&mut state)?;
    }
}
