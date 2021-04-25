use std::collections::VecDeque;

pub struct Pixel {}

pub struct Tile {}

pub struct Ppu {
    pub fifo: VecDeque<u8>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            fifo: VecDeque::with_capacity(16),
        }
    }
}
