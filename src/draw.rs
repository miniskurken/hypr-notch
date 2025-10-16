// filepath: hypr-notch/src/draw.rs
//! Drawing utilities for hypr-notch
//!
//! This file contains functions for drawing the notch UI,
//! including handling transparency, rounded corners,
//! and other visual elements.

use fontdue::{Font, FontSettings};
use log::{info, warn};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

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

fn get_system_font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();

    FONT.get_or_init(|| {
        // Try to load system fonts in order of preference
        let font_paths = [
            // Common font paths on Arch-based systems
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/Arial.ttf",
            "/usr/share/fonts/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
            // Add more potential paths here
        ];

        for path in &font_paths {
            if let Ok(font) = load_font_from_path(path) {
                info!("Loaded system font from {}", path);
                return font;
            }
        }

        // Fallback to a simple built-in font if no system fonts found
        warn!("No system fonts found, using embedded fallback font");
        let fallback_data = include_bytes!("../assets/fallback.ttf");
        Font::from_bytes(fallback_data as &[u8], FontSettings::default())
            .expect("Failed to load fallback font")
    })
}

fn load_font_from_path(path: &str) -> Result<Font, Box<dyn std::error::Error>> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(format!("Font file not found: {}", path.display()).into());
    }

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let font = Font::from_bytes(buffer, FontSettings::default())?;
    Ok(font)
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

    /// Draw text with given color, size and position
    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, color: [u8; 4], size: f32) {
        let font = get_system_font();
        let scale = size;

        // Track current position
        let mut cursor_x = x;

        for c in text.chars() {
            // Get the rasterized glyph
            let (metrics, bitmap) = font.rasterize(c, scale);

            // Skip non-renderable characters
            if metrics.width == 0 || metrics.height == 0 {
                cursor_x += (metrics.advance_width + 1.0) as i32;
                continue;
            }

            // Render the glyph
            let glyph_x = cursor_x + metrics.xmin;
            let glyph_y = y + metrics.ymin;

            for glyph_y_offset in 0..metrics.height {
                let canvas_y = glyph_y + glyph_y_offset as i32;
                if canvas_y < 0 || canvas_y >= self.height as i32 {
                    continue;
                }

                for glyph_x_offset in 0..metrics.width {
                    let canvas_x = glyph_x + glyph_x_offset as i32;
                    if canvas_x < 0 || canvas_x >= self.width as i32 {
                        continue;
                    }

                    // Get alpha value from bitmap
                    let alpha = bitmap[glyph_y_offset * metrics.width + glyph_x_offset] as u16;
                    if alpha == 0 {
                        continue;
                    }

                    // Calculate the index in our canvas buffer
                    let idx = (canvas_y as u32 * self.width + canvas_x as u32) as usize * 4;
                    if idx + 3 < self.buffer.len() {
                        // Blend the glyph with existing color
                        let blend_alpha = alpha as f32 / 255.0;

                        for i in 0..3 {
                            let existing = self.buffer[idx + i] as f32;
                            let new = color[i] as f32;
                            self.buffer[idx + i] =
                                (existing * (1.0 - blend_alpha) + new * blend_alpha) as u8;
                        }

                        // Update alpha channel
                        let existing_alpha = self.buffer[idx + 3] as f32 / 255.0;
                        let new_alpha = (color[3] as f32 / 255.0) * blend_alpha;
                        let final_alpha =
                            (existing_alpha + new_alpha * (1.0 - existing_alpha)) * 255.0;
                        self.buffer[idx + 3] = final_alpha.min(255.0) as u8;
                    }
                }
            }

            // Advance cursor position
            cursor_x += metrics.advance_width as i32;
        }
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
