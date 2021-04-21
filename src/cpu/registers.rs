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

    pub fn get_16bit_reg(self: &Registers, pair: RegisterPair) -> u16 {
        match pair {
            RegisterPair::Af => Registers::get_combined_value(self.a, self.f),
            RegisterPair::Bc => Registers::get_combined_value(self.b, self.c),
            RegisterPair::De => Registers::get_combined_value(self.d, self.e),
            RegisterPair::Hl => Registers::get_combined_value(self.h, self.l),
        }
    }

    fn get_combined_value(r1: u8, r2: u8) -> u16 {
        (r1 as u16) << 8 & r2 as u16
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::registers::RegisterPair;
    use crate::cpu::registers::Registers;
    #[test]
    fn it_works() {
        let registers = Registers::new();
        assert_eq!(registers.get_16bit_reg(RegisterPair::Af), 0)
    }
}
