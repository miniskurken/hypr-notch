// filepath: hypr-notch/src/modules/clock.rs
//! Simple clock module for hypr-notch
//!
//! Displays the current time in the notch.

use crate::draw::Canvas;
use crate::module::{Module, ModuleEvent, Rect};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ClockModule {
    id: String,
    name: String,
    color: [u8; 4],
    format: String,
}

impl ClockModule {
    pub fn new() -> Self {
        Self {
            id: "clock".to_string(),
            name: "Clock".to_string(),
            color: [255, 255, 255, 255], // White
            format: "%H:%M:%S".to_string(),
        }
    }

    fn get_current_time(&self) -> String {
        // Simple implementation that shows HH:MM:SS
        // Later you can use chrono for better formatting
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let hours = (now / 3600) % 24;
        let minutes = (now / 60) % 60;
        let seconds = now % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

impl Module for ClockModule {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn init(&mut self, config: &toml::Table) -> Result<(), Box<dyn std::error::Error>> {
        // Parse color from config if present
        if let Some(color) = config.get("color").and_then(|v| v.as_array()) {
            if color.len() >= 4 {
                for (i, component) in color.iter().take(4).enumerate() {
                    if let Some(val) = component.as_integer() {
                        self.color[i] = val as u8;
                    }
                }
            }
        }

        // Parse format from config if present
        if let Some(format) = config.get("format").and_then(|v| v.as_str()) {
            self.format = format.to_string();
        }

        Ok(())
    }

    fn draw(&self, canvas: &mut Canvas, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        // Fill background for debugging
        canvas.fill_rect(area.x, area.y, area.width, area.height, [50, 50, 50, 200]);

        // Draw time text
        let time_str = self.get_current_time();
        canvas.draw_text(area.x + 5, area.y + 5, &time_str, self.color);

        Ok(())
    }

    fn handle_event(&mut self, event: &ModuleEvent, _area: Rect) -> bool {
        match event {
            ModuleEvent::Update => {
                // Redraw on update events
                true
            }
            _ => false,
        }
    }

    fn preferred_size(&self) -> (u32, u32) {
        (100, 30) // Default size for clock
    }
}
