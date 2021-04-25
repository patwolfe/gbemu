mod instruction;
mod registers;

use crate::cpu::instruction::Instruction;
use crate::cpu::registers::Registers;
use crate::memory::Memory;
use crate::ppu::Ppu;

pub struct Cpu {
    registers: Registers,
    pc: u16,
    sp: u16,
    memory: Memory,
    ppu: Ppu,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            pc: 0,
            sp: 0xFFFE,
            memory: Memory::initialize(),
            ppu: Ppu::new(),
        }
    }

    pub fn step(&mut self, _framebuffer: &mut Vec<u32>) {
        let instruction = Instruction::from_bytes(&self.memory, self.pc);
        println!("{}", instruction);
        self.pc = self.execute(&instruction);
    }

    fn execute(&mut self, i: &Instruction) -> u16 {
        let (size, _cycles) = Instruction::size_and_cycles(i);
        self.pc + size as u16
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn execute_nop() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.pc + 1, cpu.execute(&Instruction::Nop));
    }
}
