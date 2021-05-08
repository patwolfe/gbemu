use crate::memory::Memory;

pub enum Interrupt {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

pub struct InterruptHandler {
    ime: bool
}

pub fun interrupts_enabled(&self) -> bool {
    self.ime
}

pub fun enable_interrupts(&mut self) {
    self.ime = true;
}

pub fun disable_interrupts(&mut self) {
    self.ime = false;
}

