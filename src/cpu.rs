mod instruction;
mod registers;

use crate::cpu::instruction::*;
use crate::cpu::registers::*;
use crate::gb;
use crate::memory::Memory;
use crate::ppu::Ppu;
use crate::timer;
use crate::timer::Cycles;

pub struct Cpu {
    registers: Registers,
    pc: u16,
    sp: u16,
    pub memory: Memory,
    ppu: Ppu,
    current_instruction: (Instruction, u8),
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            pc: gb::init_pc_value,
            sp: gb::init_sp_value,
            memory: Memory::initialize(),
            ppu: Ppu::new(),
            current_instruction: (Instruction::Nop, 0),
        }
    }

    pub fn step(&mut self, _framebuffer: &mut Vec<u32>) {
        let instruction = Instruction::from_bytes(&self.memory, self.pc);
        println!("{}", instruction);
        self.pc = self.execute(&instruction);
    }

    fn execute(&mut self, i: &Instruction) -> u16 {
        let (size, _cycles) = Instruction::size_and_cycles(i);
        match i {
            Instruction::Nop => {}
            Instruction::Stop => {}
            Instruction::Halt => {}
            Instruction::Load16(operand1, operand2) => {
                match (operand1, operand2) {
                    (Load16Target::Register16(reg), Load16Source::Data(d16)) => {
                        // need to read 2 bytes for d16, currently done in instruciton decode
                        self.registers.set_16bit(reg, *d16)
                    }
                    (Load16Target::Address(a16), Load16Source::StackPointer) => {
                        // need to read 2 bytes for a16, currently done in instruciton decode
                        self.memory.write_2_bytes(*a16, self.sp);
                    }
                    (Load16Target::Register16(reg), Load16Source::SpPlus(s8)) => {
                        // need to read 1 byte for s8, currently done in instruciton decode
                        let sp_plus = (self.sp as i16) + (*s8 as i16);
                        self.registers.set_16bit(reg, self.sp + (sp_plus as u16))
                    }
                    (Load16Target::StackPointer, Load16Source::Hl) => {
                        self.sp = self.registers.get_16bit(&RegisterPair::Hl)
                    }
                    _ => panic!("Enounctered unimplemented load16 instruction: {}", i),
                };
            }
            Instruction::Load8(operand1, operand2) => match (operand1, operand2) {
                (Load8Operand::Register(target_reg), Load8Operand::Register(src_reg)) => {
                    let data = self.registers.get(src_reg);
                    self.registers.set(&target_reg, data);
                }
                (Load8Operand::AtReg16(reg), Load8Operand::Register(src_reg)) => {
                    let data = self.registers.get(src_reg);
                    let address: u16 = self.registers.get_16bit(reg);
                    self.memory.write_byte(address, data);
                }
                _ => panic!("Enounctered unimplemented load8 instruction: {}", i),
            },
            Instruction::Add(operand) => {
                let result = 0;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        let result: u16 =
                            (self.registers.get(&Register::A) + self.registers.get(reg)) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::AtHl => {
                        let result: u16 = (self.registers.get(&Register::A)
                            + self
                                .memory
                                .read_byte(self.registers.get_16bit(&RegisterPair::Hl)))
                            as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        let result: u16 = (self.registers.get(&Register::A) + d8) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Carry, result > 0xFF);
                self.registers.set_flag(Flag::Subtract, false);
            }
            Instruction::Sub(operand) => {
                let result = 0;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        let result: u16 =
                            (self.registers.get(&Register::A) - self.registers.get(reg)) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::AtHl => {
                        let result: u16 = (self.registers.get(&Register::A)
                            - self
                                .memory
                                .read_byte(self.registers.get_16bit(&RegisterPair::Hl)))
                            as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        let result: u16 = (self.registers.get(&Register::A) - d8) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Subtract, true);
            }
            Instruction::And(operand) => {
                let result = 0;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        let result: u16 =
                            (self.registers.get(&Register::A) & self.registers.get(reg)) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::AtHl => {
                        let result: u16 = (self.registers.get(&Register::A)
                            & self
                                .memory
                                .read_byte(self.registers.get_16bit(&RegisterPair::Hl)))
                            as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        let result: u16 = (self.registers.get(&Register::A) & d8) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, false);
                self.registers.set_flag(Flag::Carry, false);
            }
            Instruction::Or(operand) => {
                let result = 0;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        let result: u16 =
                            (self.registers.get(&Register::A) | self.registers.get(reg)) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::AtHl => {
                        let result: u16 = (self.registers.get(&Register::A)
                            | self
                                .memory
                                .read_byte(self.registers.get_16bit(&RegisterPair::Hl)))
                            as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        let result: u16 = (self.registers.get(&Register::A) | d8) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, false);
                self.registers.set_flag(Flag::Carry, false);
            }
            _ => panic!("Enounctered unimplemented instruction: {}", i),
        };
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
    #[test]
    fn execute_load16() {
        let mut cpu = Cpu::new();
        cpu.execute(&Instruction::Load16(
            Load16Target::Register16(RegisterPair::Hl),
            Load16Source::Data(0xAB),
        ));
        assert_eq!(0xAB, cpu.registers.get_16bit(&RegisterPair::Hl));
    }
    #[test]
    fn execute_load16_sp() {
        let mut cpu = Cpu::new();
        cpu.execute(&Instruction::Load16(
            Load16Target::Register16(RegisterPair::Hl),
            Load16Source::Data(0xAB),
        ));
        cpu.execute(&Instruction::Load16(
            Load16Target::StackPointer,
            Load16Source::Hl,
        ));
        assert_eq!(0xAB, cpu.sp);
    }
}
