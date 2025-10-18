// filepath: src/pointer.rs
//! Pointer (mouse) event handling for hypr-notch

use crate::app::AppData;
use crate::module::interface::convert_pointer_event;
use log::{debug, info};
use smithay_client_toolkit::seat::pointer::{PointerEvent, PointerEventKind};

pub fn handle_pointer_events(events: &[PointerEvent], app: &mut AppData) {
    debug!("handle_pointer_events: {} events", events.len());
    for event in events {
        match event.kind {
            PointerEventKind::Enter { .. } => {
                debug!(
                    "Mouse entered notch area at coordinates: ({:.2}, {:.2})",
                    event.position.0, event.position.1
                );
                info!("Expanding notch due to mouse enter");
                app.resize(true);
                let _ = app.draw();
            }
            PointerEventKind::Leave { .. } => {
                info!("Mouse left notch area");
                info!("Collapsing notch due to mouse leave");
                app.resize(false);
                let _ = app.draw();
            }
            PointerEventKind::Motion { .. } => {
                debug!(
                    "Mouse moved within notch area: ({:.2}, {:.2})",
                    event.position.0, event.position.1
                );
            }
            _ => {}
        }

        if app.expanded {
            if let Some(_module_event) = convert_pointer_event(event) {
                app.update_modules();
            }
        }
    }
}
