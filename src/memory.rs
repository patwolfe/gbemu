use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::io::SeekFrom;

pub struct Memory {
    pub rom_bank0: Vec<u8>,
    pub rom_bank1: Vec<u8>,
    pub vram: Vec<u8>,
    pub eram: Vec<u8>,
    pub wram: Vec<u8>,
    pub oam: Vec<u8>,
    pub io: Vec<u8>,
    pub hram: Vec<u8>,
    pub interrupt_register: u8,
    pub rom_low_bytes: Vec<u8>,
}

pub const ROM0_START: u16 = 0x0000;
pub const ROM0_END: u16 = 0x3FFF;
pub const ROM1_START: u16 = 0x4000;
pub const ROM1_END: u16 = 0x7FFF;
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const ERAM_START: u16 = 0xA000;
pub const ERAM_END: u16 = 0xBFFF;
pub const WRAM_START: u16 = 0xC000;
pub const WRAM_END: u16 = 0xDFFF;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const IO_START: u16 = 0xFF00;
pub const IO_END: u16 = 0xFF7F;
pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;
pub const IR: u16 = 0xFFFF;

impl Memory {
    pub fn initialize() -> Memory {
        let bootrom_path = env::var("BOOTROM").unwrap();
        let rom_path = env::var("ROM").unwrap();
        let mut rom_bank0 = vec![0; (ROM0_END - ROM0_START + 1) as usize];
        let mut rom_low_bytes = vec![0; 0x100];
        File::open(bootrom_path)
            .unwrap()
            .read_exact(&mut rom_bank0.as_mut_slice()[0..=255])
            .unwrap();
        let mut rom = File::open(rom_path).unwrap();
        rom.read_exact(&mut rom_low_bytes).unwrap();
        rom.read_exact(&mut rom_bank0.as_mut_slice()[0x100..=(ROM0_END as usize)])
            .unwrap();
        let rom_bank1 = vec![0; (ROM1_END - ROM1_START + 1) as usize];
        let vram = vec![0; (VRAM_END - VRAM_START + 1) as usize];
        let eram = vec![0; (ERAM_END - ERAM_START + 1) as usize];
        let wram = vec![0; (WRAM_END - WRAM_START + 1) as usize];
        let oam = vec![0; (OAM_END - OAM_START + 1) as usize];
        let io = vec![0; (IO_END - IO_START + 1) as usize];
        let hram = vec![0; (HRAM_END - HRAM_START + 1) as usize];
        let interrupt_register = 0;

        Memory {
            rom_bank0,
            rom_bank1,
            vram,
            eram,
            wram,
            oam,
            io,
            hram,
            interrupt_register,
            rom_low_bytes,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            ROM0_START..=ROM0_END => self.rom_bank0[address as usize],
            ROM1_START..=ROM1_END => self.rom_bank1[(address as usize) - 0x4000],
            VRAM_START..=VRAM_END => self.vram[(address as usize) - 0x8000],
            ERAM_START..=ERAM_END => self.eram[(address as usize) - 0xA000],
            WRAM_START..=WRAM_END => self.wram[(address as usize) - 0xC000],
            0xE000..=0xFDFF => 0xFF,
            OAM_START..=OAM_END => self.oam[(address as usize) - 0xFE00],
            0xFEA0..=0xFEFF => panic!(
                "Address {:#0x} attempts to access prohibited region of memory",
                address
            ),
            IO_START..=IO_END => self.io[(address as usize) - 0xFF00],
            HRAM_START..=HRAM_END => self.hram[(address as usize) - 0xFF80],
            IR => self.interrupt_register,
        }
    }

    pub fn read_2_bytes(&self, a: u16) -> u16 {
        (self.read_byte(a) as u16) | (self.read_byte(a + 1) as u16) << 8
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            ROM0_START..=ROM0_END => self.rom_bank0[address as usize] = value,
            ROM1_START..=ROM1_END => self.rom_bank1[(address as usize) - 0x4000] = value,
            VRAM_START..=VRAM_END => self.vram[(address as usize) - 0x8000] = value,
            ERAM_START..=ERAM_END => self.eram[(address as usize) - 0xA000] = value,
            WRAM_START..=WRAM_END => self.wram[(address as usize) - 0xC000] = value,
            0xE000..=0xFDFF => {}
            OAM_START..=OAM_END => self.oam[(address as usize) - 0xFE00] = value,
            0xFEA0..=0xFEFF => {}
            IO_START..=IO_END => self.io[(address as usize) - 0xFF00] = value,
            HRAM_START..=HRAM_END => self.hram[(address as usize) - 0xFF80] = value,
            IR => self.interrupt_register = value,
        };
    }
    pub fn write_2_bytes(&mut self, a: u16, value: u16) {
        self.write_byte(a + 1, value as u8);
        self.write_byte(a, (value >> 8) as u8);
    }

    pub fn replace_bootrom(&mut self) {
        self.rom_bank0
            .splice(0..0x100, self.rom_low_bytes.as_slice().iter().cloned());
    }
}
