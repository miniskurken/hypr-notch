//! Wayland protocol handlers for hypr-notch
//!
//! This file implements the various Wayland protocol handlers
//! required by the application, including compositor, output,
//! layer shell, seat, and pointer handlers.

use crate::app::AppData;
use log::{debug, info};
use smithay_client_toolkit::{
    compositor::CompositorHandler,
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
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
        // Handle scale factor changes if needed
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Handle transform changes if needed
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        // Handle frame callbacks if needed
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
        // Handle new outputs
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        // Handle output updates
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        // Handle output destruction
    }
}

impl LayerShellHandler for AppData {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
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
        // Only update dimensions if server provides non-zero values
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
    }
}

impl SeatHandler for AppData {
    fn seat_state(&mut self) -> &mut SeatState {
        self.seat_state()
    }

    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {
        // Handle new seat
    }

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        // Get pointer capability when available
        if capability == Capability::Pointer {
            let pointer = self.seat_state().get_pointer(qh, &seat).ok();
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
        // Release pointer when capability is removed
        if capability == Capability::Pointer {
            self.set_pointer(None);
        }
    }

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {
        // Handle seat removal
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
        for event in events {
            match event.kind {
                PointerEventKind::Enter { .. } => {
                    info!(
                        "Mouse entered notch area at coordinates: ({:.2}, {:.2})",
                        event.position.0, event.position.1
                    );
                    self.resize(true);
                }
                PointerEventKind::Leave { .. } => {
                    info!("Mouse left notch area");
                    self.resize(false);
                }
                PointerEventKind::Motion { .. } => {
                    debug!(
                        "Mouse moved within notch area: ({:.2}, {:.2})",
                        event.position.0, event.position.1
                    );
                }
                PointerEventKind::Press { .. } | PointerEventKind::Release { .. } => {
                    debug!("Mouse button event in notch area");
                }
                _ => {}
            }
        }
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
