mod instruction;
mod registers;
use crate::cpu::instruction::Instruction;
use crate::cpu::registers::Registers;

struct Cpu {
    registers: Registers,
    pc: u16,
    sp: u16,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            pc: 0x0100,
            sp: 0xFFFE,
        }
    }
    pub fn step(&mut self) {
        self.pc = self.execute(Instruction::Nop);
    }

    pub fn execute(&mut self, instruction: Instruction) -> u16 {
        self.pc + 1
    }
}
