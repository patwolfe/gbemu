mod instruction;
mod registers;

use crate::cpu::instruction::Instruction;
use crate::cpu::registers::Registers;
use crate::memory::Memory;

pub struct Cpu {
    registers: Registers,
    pc: u16,
    sp: u16,
    memory: Memory,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            pc: 0x0100,
            sp: 0xFFFE,
            memory: Memory::initialize(),
        }
    }

    pub fn step(&mut self) {
        let instruction = Instruction::from_bytes(&self.memory, self.pc);
        self.pc = self.execute(instruction);
    }

    fn execute(&mut self, _instruction: Instruction) -> u16 {
        self.pc + 1
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn execute_nop() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.pc + 1, cpu.execute(Instruction::Nop));
    }
    #[test]
    fn step_nop() {
        let mut cpu = Cpu::new();
        cpu.pc = 0;
        let old_pc = cpu.pc;
        cpu.step();
        assert_eq!(cpu.pc, old_pc + 1);
    }
}
