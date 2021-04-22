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
}

#[derive(Debug, PartialEq)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    F,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let register_string = match self {
            Register::A => String::from("A"),
            Register::B => String::from("B"),
            Register::C => String::from("C"),
            Register::D => String::from("D"),
            Register::E => String::from("E"),
            Register::H => String::from("H"),
            Register::L => String::from("L"),
            Register::F => String::from("F"),
        };
        write!(f, "{}", register_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum RegisterPair {
    Af,
    Bc,
    De,
    Hl,
}

impl fmt::Display for RegisterPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let register_string = match self {
            RegisterPair::Af => String::from("AF"),
            RegisterPair::Bc => String::from("BC"),
            RegisterPair::De => String::from("DE"),
            RegisterPair::Hl => String::from("HL"),
        };
        write!(f, "{}", register_string)
    }
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            f: 0xB0,
        }
    }

    pub fn get(&self, reg: Register) -> u8 {
        match reg {
            Register::A => self.a,
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::F => self.f,
        }
    }

    pub fn set(&mut self, reg: Register, value: u8) {
        match reg {
            Register::A => self.a = value,
            Register::B => self.b = value,
            Register::C => self.c = value,
            Register::D => self.d = value,
            Register::E => self.e = value,
            Register::H => self.h = value,
            Register::gL => self.l = value,
            Register::F => self.f = value,
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
        (r1 as u16) << 8 | r2 as u16
    }

    pub fn set_16bit(&mut self, reg_pair: RegisterPair, value: u16) {
        let r1 = (value >> 8) as u8;
        let r2 = value as u8;

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
            "[ a: {}, b: {}, c: {}, d: {}, e: {}, h: {}, l: {}, f: {} ]",
            self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.f,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::registers::RegisterPair;
    use crate::cpu::registers::Registers;
    #[test]
    fn init_registers() {
        let registers = Registers::new();
        assert_eq!(registers.get_16bit(RegisterPair::Af), 0x01B0)
    }
    #[test]
    fn set_16() {
        let mut registers = Registers::new();
        registers.set_16bit(RegisterPair::Af, 0x0A0C);
        assert_eq!(registers.a, 0x0A);
        assert_eq!(registers.f, 0x0C);
    }
    #[test]
    fn set_16_then_get_16() {
        let mut registers = Registers::new();
        registers.set_16bit(RegisterPair::Af, 0x0A0C);
        assert_eq!(registers.get_16bit(RegisterPair::Af), 0x0A0C);
    }
}
