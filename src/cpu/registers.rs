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
}

pub enum Flag {
    Zero,
    Subtract,
    HalfCarry,
    Carry,
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
        };
        write!(f, "{}", register_string)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
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
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            f: 0,
        }
    }

    pub fn get(&self, reg: &Register) -> u8 {
        match reg {
            Register::A => self.a,
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
        }
    }

    pub fn set(&mut self, reg: &Register, value: u8) {
        match reg {
            Register::A => self.a = value,
            Register::B => self.b = value,
            Register::C => self.c = value,
            Register::D => self.d = value,
            Register::E => self.e = value,
            Register::H => self.h = value,
            Register::L => self.l = value,
        }
    }

    pub fn get_16bit(self: &Registers, reg_pair: &RegisterPair) -> u16 {
        match reg_pair {
            RegisterPair::Af => Registers::get_combined_value(self.a, self.f),
            RegisterPair::Bc => Registers::get_combined_value(self.b, self.c),
            RegisterPair::De => Registers::get_combined_value(self.d, self.e),
            RegisterPair::Hl => Registers::get_combined_value(self.h, self.l),
        }
    }

    fn get_combined_value(r1: u8, r2: u8) -> u16 {
        ((r1 as u16) << 8) | r2 as u16
    }

    pub fn set_16bit(&mut self, reg_pair: &RegisterPair, value: u16) {
        let r1 = (value >> 8) as u8;
        let r2 = value as u8;

        match reg_pair {
            RegisterPair::Af => {
                self.a = r1;
                // only upper 4 bits of F are used
                self.f = r2 & 0xF0;
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

    pub fn get_flag(&self, flag: Flag) -> bool {
        let value = match flag {
            Flag::Zero => self.f & 0x80,
            Flag::Subtract => self.f & 0x40,
            Flag::HalfCarry => self.f & 0x20,
            Flag::Carry => self.f & 0x10,
        };
        value != 0
    }
    pub fn set_flag(&mut self, flag: Flag, val: bool) {
        if val {
            match flag {
                Flag::Zero => self.f |= 0x80,
                Flag::Subtract => self.f |= 0x40,
                Flag::HalfCarry => self.f |= 0x20,
                Flag::Carry => self.f |= 0x10,
            };
        } else {
            match flag {
                Flag::Zero => self.f &= !0x80,
                Flag::Subtract => self.f &= !0x40,
                Flag::HalfCarry => self.f &= !0x20,
                Flag::Carry => self.f &= !0x10,
            };
        };
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[ a: {:#0x}, b: {:#0x}, c: {:#0x}, d: {:#0x}, e: {:#0x}, h: {:#0x}, l: {:#0x}, Z: {} N: {} H: {} C: {} ]",
            self.a, self.b, self.c, self.d, self.e, self.h, self.l, if self.get_flag(Flag::Zero) {1} else {0}, if self.get_flag(Flag::Subtract)  {1} else {0}, if self.get_flag(Flag::HalfCarry)  {1} else {0}, if self.get_flag(Flag::Carry)  {1} else {0}
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // #[test]
    // fn init_registers() {
    //     let registers = Registers::new();
    //     assert_eq!(registers.get_16bit(&RegisterPair::Af), 0x01B0)
    // }
    #[test]
    fn set_16() {
        let mut registers = Registers::new();
        registers.set_16bit(&RegisterPair::Hl, 0x0A0C);
        assert_eq!(registers.h, 0x0A);
        assert_eq!(registers.l, 0x0C);
    }
    #[test]
    fn set_16_then_get_16() {
        let mut registers = Registers::new();
        registers.set_16bit(&RegisterPair::Hl, 0x0A0C);
        assert_eq!(registers.get_16bit(&RegisterPair::Hl), 0x0A0C);
    }
}
