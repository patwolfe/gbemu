pub struct Memory {
    pub bootrom: Box<[u8]>,
    pub rom_bank0: Box<[u8]>,
    pub rom_bank1: Box<[u8]>,
    pub vram: Box<[u8]>,
    pub eram: Box<[u8]>,
    pub wram: Box<[u8]>,
    pub oam: Box<[u8]>,
    pub io: Box<[u8]>,
    pub hram: Box<[u8]>,
    pub interrupt_register: u8,
}

impl Memory {
    pub fn initialize() -> Memory {
        let bootrom = Box::new([0; 16]);
        let rom_bank0 = Box::new([0; 0x4000]);
        let rom_bank1 = Box::new([0; 0x4000]);
        let vram = Box::new([0; 0x2000]);
        let eram = Box::new([0; 0x2000]);
        let wram = Box::new([0; 0x2000]);
        let oam = Box::new([0; 0x100]);
        let io = Box::new([0; 0x80]);
        let hram = Box::new([0; 0]);
        let interrupt_register = 0;

        Memory {
            bootrom,
            rom_bank0,
            rom_bank1,
            vram,
            eram,
            wram,
            oam,
            io,
            hram,
            interrupt_register,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0..=0x3FFF => self.rom_bank0[address as usize],
            0x4000..=0x7FFF => self.rom_bank1[(address as usize) - 0x4000],
            0x8000..=0x9FFF => self.vram[(address as usize) - 0x8000],
            0xA000..=0xBFFF => self.eram[(address as usize) - 0xA000],
            0xC000..=0xDFFF => self.wram[(address as usize) - 0xC000],
            0xE000..=0xFDFF => panic!(
                "Address {:#0x} attempts to access prohibited region of memory",
                address
            ),
            0xFE00..=0xFE9F => self.oam[(address as usize) - 0xFE00],
            0xFEA0..=0xFEFF => panic!(
                "Address {:#0x} attempts to access prohibited region of memory",
                address
            ),
            0xFF00..=0xFF7F => self.io[(address as usize) - 0xFF00],
            0xFF80..=0xFFFE => self.hram[(address as usize) - 0xFF80],
            0xFFFF => self.interrupt_register,
        }
    }
}
