// filepath: hypr-notch/src/draw.rs
//! Drawing utilities for hypr-notch
//!
//! This file contains functions for drawing the notch UI,
//! including handling transparency, rounded corners,
//! and other visual elements.

/// Fill a canvas with color and rounded corners if expanded
pub fn fill_canvas_with_rounded_corners(
    canvas: &mut [u8],
    width: u32,
    height: u32,
    expanded: bool,
    corner_radius: u32,
    color: [u8; 4],
) {
    if !expanded || corner_radius == 0 {
        // If not expanded or radius is zero, just fill with solid color
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
        return;
    }

    // Draw with rounded corners at the bottom when expanded
    let radius = corner_radius as i32;

    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let idx = (y * width as i32 + x) as usize * 4;

            // Check if this pixel is in the rounded corner areas
            let in_rounded_area = if y > height as i32 - radius {
                // Bottom left corner
                if x < radius {
                    let dx = radius - x;
                    let dy = y - (height as i32 - radius);
                    dx * dx + dy * dy > radius * radius
                }
                // Bottom right corner
                else if x >= width as i32 - radius {
                    let dx = x - (width as i32 - radius);
                    let dy = y - (height as i32 - radius);
                    dx * dx + dy * dy > radius * radius
                } else {
                    false
                }
            } else {
                false
            };

            if !in_rounded_area {
                canvas[idx..idx + 4].copy_from_slice(&color);
            } else {
                // Set transparent for rounded corners
                canvas[idx..idx + 3].copy_from_slice(&[0, 0, 0]);
                canvas[idx + 3] = 0; // Transparent
            }
        }
    }
}

/// Canvas abstraction for module drawing
pub struct Canvas<'a> {
    buffer: &'a mut [u8],
    width: u32,
    height: u32,
}

impl<'a> Canvas<'a> {
    /// Create a new canvas from a raw buffer
    pub fn new(buffer: &'a mut [u8], width: u32, height: u32) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    /// Get the width of the canvas
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height of the canvas
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32, color: [u8; 4]) {
        // Ensure the rectangle is within bounds
        let x_start = x.max(0) as u32;
        let y_start = y.max(0) as u32;
        let x_end = (x + width as i32).min(self.width as i32) as u32;
        let y_end = (y + height as i32).min(self.height as i32) as u32;

        if x_end <= x_start || y_end <= y_start {
            return; // Nothing to draw
        }

        for y in y_start..y_end {
            for x in x_start..x_end {
                let idx = (y * self.width + x) as usize * 4;
                if idx + 3 < self.buffer.len() {
                    self.buffer[idx..idx + 4].copy_from_slice(&color);
                }
            }
        }
    }

    /// Draw simple text (placeholder implementation - will need a proper font renderer)
    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, color: [u8; 4]) {
        // This is a very simple placeholder that just draws a rectangle
        // Later, you'll want to implement proper text rendering with a font library
        let text_width = text.len() as u32 * 8; // Assume 8px per character
        let text_height = 16; // Assume 16px height

        self.fill_rect(x, y, text_width, text_height, color);
    }
}

/// Draw an anti-aliased rounded corner
/// This function can be used later for smoother corners
#[allow(dead_code)]
pub fn draw_antialiased_rounded_corner(
    _canvas: &mut [u8],
    _width: u32,
    _height: u32,
    _corner_radius: u32,
    _color: [u8; 4],
) {
    // Implementation for future enhancement
}
