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

    pub fn set_interrupt(&mut self, memory: &mut Memory, interrupt: Interrupt) {
        memory.write_byte(
            gb::ie,
            memory.read_byte(gb::ie)
                | match interrupt {
                    Interrupt::VBlank => 0x1,
                    Interrupt::LcdStat => 0x2,
                    Interrupt::Timer => 0x4,
                    Interrupt::Serial => 0x8,
                    Interrupt::Joypad => 0x10,
                },
        );
    }

    pub fn clear_interrupt(&mut self, memory: &mut Memory, interrupt: Interrupt) {
        memory.write_byte(
            gb::ie,
            memory.read_byte(gb::ie)
                & match interrupt {
                    Interrupt::VBlank => !0x1,
                    Interrupt::LcdStat => !0x2,
                    Interrupt::Timer => !0x4,
                    Interrupt::Serial => !0x8,
                    Interrupt::Joypad => !0x10,
                },
        );
    }

    pub fn check_interrupts(&mut self, memory: &mut Memory) -> Option<Interrupt> {
        let ie = memory.read_byte(gb::ie);
        if ie & 0x1 != 0 {
            self.clear_interrupt(memory, Interrupt::VBlank);
            Some(Interrupt::VBlank)
        } else if ie & 0x2 != 0 {
            self.clear_interrupt(memory, Interrupt::LcdStat);
            Some(Interrupt::LcdStat)
        } else if ie & 0x4 != 0 {
            self.clear_interrupt(memory, Interrupt::Timer);
            Some(Interrupt::Timer)
        } else if ie & 0x8 != 0 {
            self.clear_interrupt(memory, Interrupt::Serial);
            Some(Interrupt::Serial)
        } else if ie & 0x10 != 0 {
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
