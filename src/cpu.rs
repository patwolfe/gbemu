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
            pc: 0,
            sp: 0,
        }
    }
    pub fn step(&mut self) {
        let instruction = 0x00;
        self.pc = self.execute(instruction);
    }

    pub fn execute(&mut self, instruction: Instruction) -> u16 {
        self.pc + 1
    }
}
