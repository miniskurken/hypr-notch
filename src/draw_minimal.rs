// filepath: src/draw.rs
//! Drawing utilities for hypr-notch (minimal opaque version for pointer event testing)

/// Fill a canvas with a solid opaque color (for pointer event testing)
pub fn fill_canvas_with_opaque_color(canvas: &mut [u8], color: [u8; 4]) {
    for pixel in canvas.chunks_exact_mut(4) {
        pixel.copy_from_slice(&color);
    }
}

/// Canvas abstraction for module drawing (not used in this minimal version)
pub struct Canvas<'a> {
    buffer: &'a mut [u8],
    width: u32,
    height: u32,
}

impl<'a> Canvas<'a> {
    pub fn new(buffer: &'a mut [u8], width: u32, height: u32) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    // No-op for now
    pub fn fill_rect(&mut self, _x: i32, _y: i32, _width: u32, _height: u32, _color: [u8; 4]) {}
    pub fn draw_text(&mut self, _x: i32, _y: i32, _text: &str, _color: [u8; 4], _size: f32) {}
}
