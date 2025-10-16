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
