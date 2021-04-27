use std::collections::VecDeque;

use crate::gb;
use crate::memory::Memory;

pub struct Pixel {
    color_index: u8,
    // 0 == background pixel, 1 == sprite pixel
    prio: u8,
}

pub struct Tile {
    pixels: Vec<Pixel>,
}

impl Tile {
    pub fn from_bytes(byte_1: u8, byte_2: u8, prio: u8) -> Tile {
        let mut pixels = Vec::with_capacity(8);
        for i in 0..=7 {
            let color_index = ((byte_1 >> i) & 1) + (((byte_2 >> i) & 1) << 1);
            assert!(color_index < 4);
            pixels[7 - i] = Pixel { color_index, prio };
        }
        Tile { pixels }
    }
}

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
    BackgroundEnable,
}

pub struct Ppu {
    bg_fifo: VecDeque<Pixel>,
    obj_fifo: VecDeque<Pixel>,
    mode: u8,
    cycles: u64,
    pusher_current_x: u8,
    fetcher_current_tile_index: u16,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            bg_fifo: VecDeque::with_capacity(16),
            obj_fifo: VecDeque::with_capacity(16),
            mode: 0,
            cycles: 0,
            pusher_current_x: 0,
            fetcher_current_tile_index: 0,
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
            LcdcFlag::BackgroundEnable => value & 0x01 != 0,
        }
    }

    pub fn fetch_pixels(&mut self, memory: &Memory, is_object: u8) {
        if self.bg_fifo.len() > 8 {
            return;
        }
        let base_tilemap_location: u16 = if Ppu::check_lcdc(memory, LcdcFlag::WindowTileMapArea)
            || Ppu::check_lcdc(memory, LcdcFlag::TileMapArea)
        {
            0x9C00
        } else {
            0x09800
        };
        let base_tile_data_location = if Ppu::check_lcdc(memory, LcdcFlag::TileDataArea) {
            0x8000
        } else {
            0x9000
        };
        let tile_number = memory.read_byte(base_tilemap_location + self.fetcher_current_tile_index);
        let tile_data_low = memory.read_byte(base_tile_data_location + tile_number as u16);
        let tile_data_high = memory.read_byte(base_tile_data_location + tile_number as u16 + 1);
        let tile = Tile::from_bytes(tile_data_low, tile_data_high, is_object);
        for p in tile.pixels {
            self.bg_fifo.push_back(p);
        }
        // Get tile data low
        // Get tile data high
        // Push
        // Sleep
    }

    pub fn scan_oam(&self, memory: &Memory, ly: u8) -> Vec<Object> {
        let oam = &memory.oam;
        let mut results = vec![];
        // Check if object appears on this scanline
        for i in 0..oam.len() / 4 {
            //
            if oam[i] <= ly + 16 && oam[i] + 8 > ly + 16 && oam[i + 1] != 0
            // && oam[i + 1] <= self.current_x
            // && oam[i + 1] + TILE_DIMENSION > self.current_x
            {
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

    pub fn draw_scanline(&mut self, memory: &mut Memory, buffer: &mut Vec<u32>) {
        let ly = memory.read_byte(gb::ly_addr);
        // this is mode 2, always takes 20 cycles
        let objects = self.scan_oam(memory, ly);
        while (self.pusher_current_x as usize) < gb::screen_width {
            self.fetch_pixels(memory, 0);
            self.push_pixels(memory, buffer);
        }
        self.pusher_current_x = 0;
    }
    fn push_pixels(&mut self, memory: &mut Memory, buffer: &mut Vec<u32>) {
        if self.bg_fifo.len() > 8 {
            let ly = memory.read_byte(gb::ly_addr);
            let curr_pixel = self.bg_fifo.pop_front().unwrap();
            // TODO: use BGP or OBP to map color index -> color value
            buffer[ly as usize * gb::screen_width + self.pusher_current_x as usize] =
                curr_pixel.color_index as u32;
            self.pusher_current_x += 1;
        }
    }

    pub fn draw_frame(&mut self, memory: &mut Memory, buffer: &mut Vec<u32>) {
        let mut ly = memory.read_byte(gb::ly_addr);
        while (ly as usize) < gb::screen_height {
            self.draw_scanline(memory, buffer);
            ly += 1;
            memory.write_byte(gb::ly_addr, ly);
        }
    }
}
