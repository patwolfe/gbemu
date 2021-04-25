use std::collections::VecDeque;

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
}
