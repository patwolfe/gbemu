use std::collections::VecDeque;

use crate::gb;
use crate::memory::Memory;

pub struct Pixel {
    color: u8,
    palette: u8,
    prio: u8,
}

pub struct Tile {}

pub struct Object {
    y: u8,
    x: u8,
    index: u8,
    attr: u8,
}

pub enum LcdcFlag {
    Enable,
    WindowTileMapArea,
    EnableWindow,
    TileDataArea,
    TileMapArea,
    ObjectSize,
    ObjectEnable,
    BgWindowPriority,
}

pub struct Ppu {
    bg_fifo: VecDeque<u8>,
    obj_fifo: VecDeque<u8>,
    current_x: u8,
    current_y: u8,
    mode: u8,
    cycles: u64,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            bg_fifo: VecDeque::with_capacity(16),
            obj_fifo: VecDeque::with_capacity(16),
            current_x: 0,
            current_y: 0,
            mode: 0,
            cycles: 0,
        }
    }

    pub fn check_lcdc(memory: &Memory, flag: LcdcFlag) -> bool {
        let value: u8 = memory.read_byte(gb::lcdc_addr);
        match flag {
            LcdcFlag::Enable => value & 0x80 != 0,
            LcdcFlag::WindowTileMapArea => value & 0x40 != 0,
            LcdcFlag::EnableWindow => value & 0x20 != 0,
            LcdcFlag::TileDataArea => value & 0x10 != 0,
            LcdcFlag::TileMapArea => value & 0x08 != 0,
            LcdcFlag::ObjectSize => value & 0x04 != 0,
            LcdcFlag::ObjectEnable => value & 0x02 != 0,
            LcdcFlag::BgWindowPriority => value & 0x01 != 0,
        }
    }

    pub fn fetch_pixel(_memory: &mut Memory) {
        // Get tile
        // Get tile data low
        // Get tile data high
        // Push
        // Sleep
    }

    pub fn scan_oam(&self, memory: &Memory) -> Vec<Object> {
        let oam = &memory.oam;
        let mut results = vec![];
        for i in 0..oam.len() / 4 {
            if oam[i] == self.current_y && oam[i + 1] == self.current_x {
                results.push(Object {
                    y: oam[i],
                    x: oam[i + 1],
                    index: oam[i + 2],
                    attr: oam[i + 3],
                })
            }
            // can only have 10 objects per scanline
            if results.len() == 10 {
                break;
            }
        }
        results
    }

    pub fn draw_frame(&self, memory: &Memory, buffer: &mut Vec<u8>) {}
}
