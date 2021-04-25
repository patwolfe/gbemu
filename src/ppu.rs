use std::collections::VecDeque;

use crate::memory::Memory;

pub struct Pixel {}

pub struct Tile {}

pub struct Ppu {
    bg_fifo: VecDeque<u8>,
    sprite_fifo: VecDeque<u8>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            bg_fifo: VecDeque::with_capacity(16),
            sprite_fifo: VecDeque::with_capacity(16),
        }
    }

    pub fn fetch_pixels(memory: &mut Memory) {}

    pub fn draw_canvas(framebuffer: &mut Vec<u32>, memory: &mut Memory) {
        // Get tile
        // Get tile data low
        // Get tile data high
        // Push
        // Sleep
    }
}
