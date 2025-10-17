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
    font_size: f32,
    background_color: [u8; 4],
}

impl ClockModule {
    pub fn new() -> Self {
        Self {
            id: "clock".to_string(),
            name: "Clock".to_string(),
            color: [255, 255, 255, 255], // White
            format: "%H:%M:%S".to_string(),
            font_size: 16.0,
            background_color: [0, 0, 0, 0], // Fully transparent
        }
    }

    fn get_current_time(&self) -> String {
        // Simple implementation that shows HH:MM:SS
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

        // Parse font size if present
        if let Some(size) = config.get("font_size").and_then(|v| v.as_float()) {
            self.font_size = size as f32;
        }

        if let Some(bg) = config.get("background_color").and_then(|v| v.as_array()) {
            if bg.len() >= 4 {
                for (i, component) in bg.iter().take(4).enumerate() {
                    if let Some(val) = component.as_integer() {
                        self.background_color[i] = val as u8;
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&self, canvas: &mut Canvas, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        // Fill background only if not fully transparent
        if self.background_color[3] > 0 {
            canvas.fill_rect(
                area.x,
                area.y,
                area.width,
                area.height,
                self.background_color,
            );
        }

        // Draw time text
        let time_str = self.get_current_time();
        let font_size = self.font_size.min(area.height as f32 * 0.8);
        let y_pos = area.y + ((area.height as f32 - font_size) / 2.0) as i32;
        canvas.draw_text(area.x + 10, y_pos, &time_str, self.color, font_size);

        Ok(())
    }

    fn handle_event(&mut self, event: &ModuleEvent, _area: Rect) -> bool {
        match event {
            ModuleEvent::Update | ModuleEvent::UpdateExpanded => {
                // Redraw on update events when visible
                true
            }
            _ => false,
        }
    }

    fn preferred_size(&self) -> (u32, u32) {
        (100, 30) // Default size for clock
    }
}
