use std::fmt;

pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    f: u8,
    pc: u16,
    sp: u16,
}
pub enum RegisterPair {
    Af,
    Bc,
    De,
    Hl,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            f: 0,
            pc: 0,
            sp: 0,
        }
    }

    pub fn get_16bit(self: &Registers, reg_pair: RegisterPair) -> u16 {
        match reg_pair {
            RegisterPair::Af => Registers::get_combined_value(self.a, self.f),
            RegisterPair::Bc => Registers::get_combined_value(self.b, self.c),
            RegisterPair::De => Registers::get_combined_value(self.d, self.e),
            RegisterPair::Hl => Registers::get_combined_value(self.h, self.l),
        }
    }

    fn get_combined_value(r1: u8, r2: u8) -> u16 {
        (r1 as u16) << 8 & r2 as u16
    }

    pub fn set_16bit(&mut self, reg_pair: RegisterPair, value: u16) {
        let r1 = ((value & 0xF0) >> 8) as u8;
        let r2 = (value & 0x0F) as u8;

        match reg_pair {
            RegisterPair::Af => {
                self.a = r1;
                self.f = r2;
            }
            RegisterPair::Bc => {
                self.b = r1;
                self.c = r2;
            }
            RegisterPair::De => {
                self.d = r1;
                self.e = r2;
            }
            RegisterPair::Hl => {
                self.h = r1;
                self.l = r2;
            }
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(a: {}, b: {}, c: {}, d: {}, e: {}, h: {}, l: {}, f: {}, pc: {}, sp: {} )",
            self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.f, self.pc, self.sp
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::registers::RegisterPair;
    use crate::cpu::registers::Registers;
    #[test]
    fn init_zero() {
        let registers = Registers::new();
        assert_eq!(registers.get_16bit(RegisterPair::Af), 0)
    }
    #[test]
    fn set_16() {
        let mut registers = Registers::new();
        registers.set_16bit(RegisterPair::Af, 0xAC);
        assert_eq!(registers.a, 0xA);
        assert_eq!(registers.f, 0xC);
    }
}
