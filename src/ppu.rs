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
            pixels.push(Pixel { color_index, prio });
        }
        pixels.reverse();
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

enum PpuMode {
    OamSearch = 2,
    PixelTransfer = 3,
    Hblank = 0,
    Vblank = 1,
}

pub struct Ppu {
    bg_fifo: VecDeque<Pixel>,
    obj_fifo: VecDeque<Pixel>,
    mode: PpuMode,
    cycles_this_frame: u64,
    x: u8,
    fetcher_current_tile_index: u16,
    sprite_buffer: Vec<Object>,
    oam_offset: usize,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            bg_fifo: VecDeque::with_capacity(16),
            obj_fifo: VecDeque::with_capacity(16),
            mode: PpuMode::OamSearch,
            cycles_this_frame: 0,
            x: 0,
            fetcher_current_tile_index: 0,
            sprite_buffer: Vec::with_capacity(10),
            oam_offset: 0,
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

    pub fn step(&mut self, cycles: u32, memory: &mut Memory, buffer: &mut Vec<u32>) {
        let mut cycles_taken = 0;
        while cycles_taken < cycles {
            if !Ppu::check_lcdc(memory, LcdcFlag::Enable) {
                return;
            }
            let curr_cycle = 1 + (self.cycles_this_frame % 114);
            let ly = memory.read_byte(gb::ly_addr);
            let lcd_stat = memory.read_byte(gb::lcd_stat);
            let mode = lcd_stat & 0x3;
            match mode {
                // OAM search
                0x2 => {
                    if self.sprite_buffer.len() == 10 {
                        continue;
                    }
                    let oam = &memory.oam;
                    for i in self.oam_offset..self.oam_offset + 4 {
                        let index = i * 4;
                        if oam[index] <= ly + 16 && oam[index] + 8 > ly + 16 && oam[index + 1] != 0
                        {
                            self.sprite_buffer.push(Object {
                                y: oam[index],
                                x: oam[index + 1],
                                index: oam[index + 2],
                                attr: oam[index + 3],
                            })
                        }
                    }
                    if curr_cycle == 20 {
                        // sort sprite buffer by x
                        memory.write_byte(gb::lcd_stat, (lcd_stat & 0xFC) | 0x3)
                    }
                }
                0x3 => {
                    if self.bg_fifo.len() <= 8 {
                        // fetch tile
                        let base_tilemap_location: u16 =
                            if Ppu::check_lcdc(memory, LcdcFlag::WindowTileMapArea)
                                || Ppu::check_lcdc(memory, LcdcFlag::TileMapArea)
                            {
                                0x9C00
                            } else {
                                0x09800
                            };
                        let base_tile_data_location =
                            if Ppu::check_lcdc(memory, LcdcFlag::TileDataArea) {
                                0x8000
                            } else {
                                0x9000
                            };
                        let tile_number = memory
                            .read_byte(base_tilemap_location + self.fetcher_current_tile_index);
                        let tile_data_low =
                            memory.read_byte(base_tile_data_location + tile_number as u16);
                        let tile_data_high =
                            memory.read_byte(base_tile_data_location + tile_number as u16 + 1);
                        let tile = Tile::from_bytes(tile_data_low, tile_data_high, 0);
                        for p in tile.pixels {
                            self.bg_fifo.push_back(p);
                        }
                    } else {
                        // push pixel
                        let ly = memory.read_byte(gb::ly_addr);
                        let curr_pixel = self.bg_fifo.pop_front().unwrap();
                        // TODO: use BGP or OBP to map color index -> color value
                        if ly as usize * gb::screen_width + self.x as usize > buffer.len() {
                            panic!("ly: {} * 144 + x: {}", ly, self.x);
                        }
                        buffer[ly as usize * gb::screen_width + self.x as usize] =
                            Ppu::get_color(curr_pixel.color_index) as u32;
                        self.x += 1;
                    }
                    if self.x == 160 {
                        self.x = 0;
                        memory.write_byte(gb::lcd_stat, lcd_stat & 0xFC)
                    }
                }
                0x0 => {
                    if curr_cycle == 114 {
                        self.bg_fifo.clear();
                        if ly == 144 {
                            memory.write_byte(gb::lcd_stat, (lcd_stat & 0xFC) | 0x1)
                        } else {
                            memory.write_byte(gb::lcd_stat, (lcd_stat & 0xFC) | 0x2)
                        }
                    }
                }
                0x1 => {
                    if memory.read_byte(gb::ly_addr) == 153 {
                        memory.write_byte(gb::lcd_stat, (lcd_stat & 0xFC) | 0x2);
                    }
                }
                _ => panic!("Unexpected PPU mode: {}", mode),
            };
            if curr_cycle == 114 {
                if ly == 153 {
                    memory.write_byte(gb::ly_addr, 0);
                    println!("setting ly to 0");
                } else {
                    memory.write_byte(gb::ly_addr, ly + 1);
                    println!("setting ly to {}", ly + 1);
                }
            }
            if self.cycles_this_frame == 17555 {
                self.cycles_this_frame = 0
            } else {
                self.cycles_this_frame += 1;
            }
            cycles_taken += 1;
        }
    }
    pub fn get_color(i: u8) -> u32 {
        match i {
            0 => 0x00FF,
            1 => 0x0FF0,
            2 => 0x00F0,
            3 => 0x000F,
            _ => panic!(),
        }
    }
}
