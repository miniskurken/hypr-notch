// filepath: src/wayland.rs
//! Wayland protocol handlers for hypr-notch

use crate::app::AppData;
use crate::pointer::handle_pointer_events;
use log::{debug, info, warn};
use smithay_client_toolkit::{
    compositor::CompositorHandler,
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
    shm::{Shm, ShmHandler},
};
use wayland_client::{
    protocol::{wl_output, wl_pointer, wl_seat, wl_surface},
    Connection, QueueHandle,
};

impl CompositorHandler for AppData {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        debug!("CompositorHandler: scale_factor_changed");
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        debug!("CompositorHandler: transform_changed");
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        if self.expanded {
            self.update_modules();
        }
    }
}

impl OutputHandler for AppData {
    fn output_state(&mut self) -> &mut OutputState {
        self.output_state()
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        info!("OutputHandler: new_output");
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        info!("OutputHandler: update_output");
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        info!("OutputHandler: output_destroyed");
    }
}

impl LayerShellHandler for AppData {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        info!("LayerShellHandler: closed");
        self.close_layer_surface();
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        info!("Surface configured to size: {:?}", configure.new_size);
        let mut width = self.width;
        let mut height = self.height;

        if configure.new_size.0 != 0 {
            width = configure.new_size.0;
        }
        if configure.new_size.1 != 0 {
            height = configure.new_size.1;
        }

        self.update_size(width, height);
        self.set_configured(true);

        info!("Surface now configured with size: {}x{}", width, height);
    }
}

impl SeatHandler for AppData {
    fn seat_state(&mut self) -> &mut SeatState {
        self.seat_state()
    }

    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {
        info!("SeatHandler: new_seat");
    }

    fn new_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        info!("SeatHandler: new_capability: {:?}", capability);
        if capability == Capability::Pointer {
            let pointer = self.seat_state().get_pointer(_qh, &seat).ok();
            if pointer.is_some() {
                info!("Pointer capability acquired and pointer created");
            } else {
                warn!("Pointer capability acquired but pointer creation failed");
            }
            self.set_pointer(pointer);
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
            self.set_pointer(None);
        }
    }

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {
        info!("SeatHandler: remove_seat");
    }
}

impl PointerHandler for AppData {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        debug!(
            "PointerHandler: pointer_frame called with {} events",
            events.len()
        );
        handle_pointer_events(events, self);
    }
}

impl ShmHandler for AppData {
    fn shm_state(&mut self) -> &mut Shm {
        self.shm_state()
    }
}

impl ProvidesRegistryState for AppData {
    fn registry(&mut self) -> &mut RegistryState {
        self.registry_state()
    }

    registry_handlers![OutputState];
}

delegate_compositor!(AppData);
delegate_output!(AppData);
delegate_shm!(AppData);
delegate_layer!(AppData);
delegate_seat!(AppData);
delegate_pointer!(AppData);
delegate_registry!(AppData);
