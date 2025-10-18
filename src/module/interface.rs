// filepath: hypr-notch/src/module/interface.rs
//! Module interface definitions
//!
//! This file defines the core traits and types that all modules must implement.

use std::any::Any;

/// Rectangle used for layout
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Events that can be sent to modules
#[derive(Debug, Clone)]
pub enum ModuleEvent {
    /// Mouse entered the module area
    Enter {
        x: f64,
        y: f64,
    },

    /// Mouse left the module area
    Leave,

    /// Mouse moved within the module area
    Motion {
        x: f64,
        y: f64,
    },

    /// Mouse button pressed within the module area
    Press {
        button: u32,
        x: f64,
        y: f64,
    },

    /// Mouse button released within the module area
    Release {
        button: u32,
        x: f64,
        y: f64,
    },

    /// Module should update its state (e.g., clock tick)
    Update,
    UpdateExpanded,
    UpdateCollapsed,
}

/// Core module trait that all modules must implement
pub trait Module: Send + Sync {
    /// Get the unique identifier for this module
    fn id(&self) -> &str;

    /// Get the human-readable name of this module
    fn name(&self) -> &str;

    /// Initialize the module with configuration
    fn init(&mut self, _config: &toml::Table) -> Result<(), Box<dyn std::error::Error>> {
        // Default implementation: no configuration needed
        Ok(())
    }

    /// Draw the module's content to the canvas
    fn draw(
        &self,
        canvas: &mut crate::draw::Canvas,
        area: Rect,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Handle an event directed at this module
    /// Returns true if the event was handled and should not be propagated further
    fn handle_event(&mut self, _event: &ModuleEvent, _area: Rect) -> bool {
        // Default implementation: don't handle any events
        false
    }

    /// Get the preferred size of this module
    fn preferred_size(&self) -> (u32, u32);

    fn as_any(&self) -> &dyn Any {
        // This is a workaround - in a real impl you'd return a reference to self
        // For now, just return a static empty value
        static EMPTY: () = ();
        &EMPTY
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        // Since we can't return a mutable reference to a static,
        // this is a hack that will panic if ever called
        // In real code, implementations should override this
        panic!("as_any_mut not implemented for this module")
    }
}

/// Convert from wayland pointer events to module events
pub fn convert_pointer_event(
    event: &smithay_client_toolkit::seat::pointer::PointerEvent,
) -> Option<ModuleEvent> {
    use smithay_client_toolkit::seat::pointer::PointerEventKind;

    match event.kind {
        PointerEventKind::Enter { .. } => Some(ModuleEvent::Enter {
            x: event.position.0,
            y: event.position.1,
        }),

        PointerEventKind::Leave { .. } => Some(ModuleEvent::Leave),

        PointerEventKind::Motion { .. } => Some(ModuleEvent::Motion {
            x: event.position.0,
            y: event.position.1,
        }),

        PointerEventKind::Press { button, .. } => Some(ModuleEvent::Press {
            button,
            x: event.position.0,
            y: event.position.1,
        }),

        PointerEventKind::Release { button, .. } => Some(ModuleEvent::Release {
            button,
            x: event.position.0,
            y: event.position.1,
        }),

        _ => None,
    }
}

pub type ModuleCreateFn = unsafe extern "C" fn() -> *mut dyn Module;
