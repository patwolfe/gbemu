use crate::gb;
use crate::memory::Memory;

pub enum Interrupt {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

pub struct InterruptHandler {
    pub ime: bool,
}
impl InterruptHandler {
    pub fn interrupts_enabled(&self) -> bool {
        self.ime
    }

    pub fn enable_interrupts(&mut self) {
        self.ime = true;
    }

    pub fn disable_interrupts(&mut self) {
        self.ime = false;
    }

    pub fn set_interrupt(&self, memory: &mut Memory, interrupt: Interrupt) {
        memory.write_byte(
            gb::iflags,
            memory.read_byte(gb::iflags)
                | match interrupt {
                    Interrupt::VBlank => 0x1,
                    Interrupt::LcdStat => 0x2,
                    Interrupt::Timer => 0x4,
                    Interrupt::Serial => 0x8,
                    Interrupt::Joypad => 0x10,
                },
        );
    }

    pub fn clear_interrupt(&self, memory: &mut Memory, interrupt: Interrupt) {
        memory.write_byte(
            gb::iflags,
            memory.read_byte(gb::iflags)
                & match interrupt {
                    Interrupt::VBlank => !0x1,
                    Interrupt::LcdStat => !0x2,
                    Interrupt::Timer => !0x4,
                    Interrupt::Serial => !0x8,
                    Interrupt::Joypad => !0x10,
                },
        );
    }

    pub fn check_interrupts(&self, memory: &mut Memory) -> Option<Interrupt> {
        let ie = memory.read_byte(gb::ie);
        let iflags = memory.read_byte(gb::iflags);
        if ie & 0x1 != 0 && iflags & 0x1 != 0 {
            println!("VBlank int");
            self.clear_interrupt(memory, Interrupt::VBlank);
            Some(Interrupt::VBlank)
        } else if ie & 0x2 != 0 && iflags & 0x2 != 0 {
            println!("LcdStat int");
            self.clear_interrupt(memory, Interrupt::LcdStat);
            Some(Interrupt::LcdStat)
        } else if ie & 0x4 != 0 && iflags & 0x4 != 0 {
            println!("Timer int");
            self.clear_interrupt(memory, Interrupt::Timer);
            Some(Interrupt::Timer)
        } else if ie & 0x8 != 0 && iflags & 0x8 != 0 {
            println!("Serial int");
            self.clear_interrupt(memory, Interrupt::Serial);
            Some(Interrupt::Serial)
        } else if ie & 0x10 != 0 && iflags & 0x10 != 0 {
            println!("Joyapd int");
            self.clear_interrupt(memory, Interrupt::Joypad);
            Some(Interrupt::Joypad)
        } else {
            None
        }
    }
}

pub fn address_for_interrupt(interrupt: Interrupt) -> u16 {
    match interrupt {
        Interrupt::VBlank => 0x40,
        Interrupt::LcdStat => 0x48,
        Interrupt::Timer => 0x50,
        Interrupt::Serial => 0x58,
        Interrupt::Joypad => 0x60,
    }
}
