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

    pub fn step(&mut self) {
        let instruction = Instruction::from_bytes(&self.memory, self.pc);
        println!("{}", instruction);
        self.pc = self.execute(&instruction);
    }

    fn execute(&mut self, i: &Instruction) -> u16 {
        let (size, _cycles) = Instruction::size_and_cycles(i);
        match i {
            Instruction::Nop => {}
            Instruction::Stop => {
                println!("Stopping program with instruction {}", i);
            }
            Instruction::Halt => {
                println!("Halting program with instruction {}", i);
            }
            Instruction::SetCarryFlag => {
                self.registers.set_flag(Flag::Carry, true);
            }
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
                let result;
                let half_carry;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        result =
                            (self.registers.get(&Register::A) + self.registers.get(reg)) as u16;
                        half_carry = (self.registers.get(&Register::A) & 0xF)
                            + (self.registers.get(reg) & 0xF)
                            > 0xF;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::AtHl => {
                        let at_hl = self
                            .memory
                            .read_byte(self.registers.get_16bit(&RegisterPair::Hl))
                            as u16;
                        result = self.registers.get(&Register::A) as u16 + at_hl;
                        half_carry =
                            (self.registers.get(&Register::A) & 0xF) as u16 + (at_hl & 0xF) > 0xF;
                        self.registers.set(&Register::A, result as u8);
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        result = (self.registers.get(&Register::A) + d8) as u16;
                        half_carry = (self.registers.get(&Register::A) & 0xF) + (d8 & 0xF) > 0xF;
                        self.registers.set(&Register::A, result as u8);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Carry, result > 0xFF);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, half_carry);
            }
            Instruction::Sub(operand) => {
                let result;
                let half_carry;
                let carry;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        result =
                            (self.registers.get(&Register::A) - self.registers.get(reg)) as u16;
                        half_carry = (self.registers.get(&Register::A) & 0xF)
                            < (self.registers.get(reg) & 0xF);
                        carry = self.registers.get(reg) > (self.registers.get(&Register::A));
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::AtHl => {
                        let at_hl = self
                            .memory
                            .read_byte(self.registers.get_16bit(&RegisterPair::Hl));
                        result = (self.registers.get(&Register::A) - at_hl) as u16;
                        half_carry = (self.registers.get(&Register::A) & 0xF) < (at_hl & 0xF);
                        carry = at_hl > self.registers.get(&Register::A);
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        result = (self.registers.get(&Register::A) - *d8) as u16;
                        half_carry = (self.registers.get(&Register::A) & 0xF) < (*d8 & 0xF);
                        carry = *d8 > self.registers.get(&Register::A);
                        self.registers.set(&Register::A, result as u8);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::HalfCarry, half_carry);
                self.registers.set_flag(Flag::Carry, carry);
                self.registers.set_flag(Flag::Subtract, true);
            }
            Instruction::AddCarry(operand) => {
                let result: u16;
                let half_carry;
                let carry_bit = if let true = self.registers.get_flag(Flag::Carry) {
                    1
                } else {
                    0
                };
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        result = (self.registers.get(&Register::A) + self.registers.get(reg))
                            as u16
                            + carry_bit;
                        half_carry = (self.registers.get(&Register::A) & 0xF)
                            + (self.registers.get(reg) & 0xF)
                            > 0xF;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::AtHl => {
                        let at_hl = self
                            .memory
                            .read_byte(self.registers.get_16bit(&RegisterPair::Hl))
                            as u16;
                        result = self.registers.get(&Register::A) as u16 + at_hl + carry_bit;
                        half_carry =
                            (self.registers.get(&Register::A) & 0xF) as u16 + (at_hl & 0xF) > 0xF;
                        self.registers.set(&Register::A, result as u8);
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        result = (self.registers.get(&Register::A) + d8) as u16 + carry_bit;
                        half_carry = (self.registers.get(&Register::A) & 0xF) + (d8 & 0xF) > 0xF;
                        self.registers.set(&Register::A, result as u8);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Carry, result > 0xFF);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, half_carry);
            }
            Instruction::SubCarry(operand) => {
                let result;
                let half_carry;
                let carry;
                let carry_bit = if let true = self.registers.get_flag(Flag::Carry) {
                    1
                } else {
                    0
                };
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        result = (self.registers.get(&Register::A) - self.registers.get(reg))
                            as u16
                            - carry_bit;
                        half_carry = (self.registers.get(&Register::A) & 0xF)
                            - (self.registers.get(reg) & 0xF)
                            - carry_bit as u8
                            >= 0x10;
                        carry = self.registers.get(reg) as u16 + carry_bit
                            > (self.registers.get(&Register::A)) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::AtHl => {
                        let at_hl = self
                            .memory
                            .read_byte(self.registers.get_16bit(&RegisterPair::Hl));
                        result = (self.registers.get(&Register::A) - at_hl) as u16 - carry_bit;
                        half_carry = (self.registers.get(&Register::A) & 0xF)
                            - (at_hl & 0xF)
                            - carry_bit as u8
                            >= 0x10;
                        carry = at_hl as u16 + carry_bit > self.registers.get(&Register::A) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        result = (self.registers.get(&Register::A) - *d8) as u16 - carry_bit;
                        half_carry = (self.registers.get(&Register::A) & 0xF)
                            - (*d8 & 0xF)
                            - carry_bit as u8
                            >= 0x10;
                        carry = *d8 as u16 + carry_bit > self.registers.get(&Register::A) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::HalfCarry, half_carry);
                self.registers.set_flag(Flag::Carry, carry);
                self.registers.set_flag(Flag::Subtract, true);
            }
            Instruction::And(operand) => {
                let result;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        result =
                            (self.registers.get(&Register::A) & self.registers.get(reg)) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::AtHl => {
                        result = (self.registers.get(&Register::A)
                            & self
                                .memory
                                .read_byte(self.registers.get_16bit(&RegisterPair::Hl)))
                            as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        result = (self.registers.get(&Register::A) & d8) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, true);
                self.registers.set_flag(Flag::Carry, false);
            }
            Instruction::Or(operand) => {
                let result;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        result =
                            (self.registers.get(&Register::A) | self.registers.get(reg)) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::AtHl => {
                        result = (self.registers.get(&Register::A)
                            | self
                                .memory
                                .read_byte(self.registers.get_16bit(&RegisterPair::Hl)))
                            as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        result = (self.registers.get(&Register::A) | d8) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, false);
                self.registers.set_flag(Flag::Carry, false);
            }
            Instruction::Xor(operand) => {
                let result;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        result =
                            (self.registers.get(&Register::A) ^ self.registers.get(reg)) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::AtHl => {
                        result = (self.registers.get(&Register::A)
                            ^ self
                                .memory
                                .read_byte(self.registers.get_16bit(&RegisterPair::Hl)))
                            as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        result = (self.registers.get(&Register::A) ^ d8) as u16;
                        self.registers.set(&Register::A, result as u8);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, false);
                self.registers.set_flag(Flag::Carry, false);
            }
            Instruction::Compare(operand) => {
                let result;
                let half_carry;
                let carry;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        result =
                            (self.registers.get(&Register::A) - self.registers.get(reg)) as u16;
                        half_carry = (self.registers.get(&Register::A) & 0xF)
                            - (self.registers.get(reg) & 0xF)
                            >= 0x10;
                        carry = self.registers.get(reg) > (self.registers.get(&Register::A));
                    }
                    ArithmeticOperand::AtHl => {
                        let at_hl = self
                            .memory
                            .read_byte(self.registers.get_16bit(&RegisterPair::Hl));
                        result = (self.registers.get(&Register::A) - at_hl) as u16;
                        half_carry =
                            (self.registers.get(&Register::A) & 0xF) - (at_hl & 0xF) >= 0x10;
                        carry = at_hl > self.registers.get(&Register::A);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        result = (self.registers.get(&Register::A) - *d8) as u16;
                        half_carry = (self.registers.get(&Register::A) & 0xF) - (*d8 & 0xF) >= 0x10;
                        carry = *d8 > self.registers.get(&Register::A);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::HalfCarry, half_carry);
                self.registers.set_flag(Flag::Carry, carry);
                self.registers.set_flag(Flag::Subtract, true);
            }
            Instruction::Increment(operand) => {
                let result;
                let half_carry;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        result = self.registers.get(&reg) + 1;
                        half_carry = (self.registers.get(&reg) ^ 1 ^ result) & 0x10 == 0x10;
                        self.registers.set(&reg, result);
                    }
                    ArithmeticOperand::AtHl => {
                        // this takes 1 cycle
                        let hl_val = self
                            .memory
                            .read_byte(self.registers.get_16bit(&RegisterPair::Hl));
                        // 1 cycle
                        result = hl_val + 1;
                        half_carry = (hl_val ^ 1 ^ result) & 0x10 == 0x10;
                        // 1 cycle
                        self.memory
                            .write_byte(self.registers.get_16bit(&RegisterPair::Hl), result)
                    }
                    _ => panic!("Enounctered malformed increment instruction: {}", i),
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, half_carry);
            }
            Instruction::Decrement(operand) => {
                let result;
                let half_carry;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        result = self.registers.get(&reg) + 1;
                        half_carry = (self.registers.get(&reg) ^ 1 ^ result) & 0x10 == 0x10;
                        self.registers.set(&reg, result);
                    }
                    ArithmeticOperand::AtHl => {
                        // this takes 1 cycle
                        let hl_val = self
                            .memory
                            .read_byte(self.registers.get_16bit(&RegisterPair::Hl));
                        // 1 cycle
                        result = hl_val + 1;
                        half_carry = (hl_val ^ 1 ^ result) & 0x10 == 0x10;
                        // 1 cycle
                        self.memory
                            .write_byte(self.registers.get_16bit(&RegisterPair::Hl), result)
                    }
                    _ => panic!("Enounctered malformed decrement instruction: {}", i),
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, half_carry);
            }
            Instruction::IncrementPtr(operand) => {
                let result;
                match operand {
                    PtrArithOperand::Register16(reg) => {
                        result = self.registers.get_16bit(&reg) + 1;
                        self.registers.set_16bit(&reg, result);
                    }
                    PtrArithOperand::StackPointer => self.sp += 1,
                    _ => panic!("Enounctered malformed 16bit increment instruction: {}", i),
                };
            }
            Instruction::DecrementPtr(operand) => {
                let result;
                match operand {
                    PtrArithOperand::Register16(reg) => {
                        result = self.registers.get_16bit(&reg) - 1;
                        self.registers.set_16bit(&reg, result);
                    }
                    PtrArithOperand::StackPointer => self.sp -= 1,
                    _ => panic!("Enounctered malformed 16bit decrement instruction: {}", i),
                };
            }
            Instruction::Rotate(kind) => {
                let bit;
                match kind {
                    RotateKind::LeftCarry => {
                        let mut a = self.registers.get(&Register::A);
                        bit = ((a & 0x80) >> 7) & 0x01;
                        a = ((a << 1) & 0xFE) | bit;
                        self.registers.set(&Register::A, a);
                    }
                    RotateKind::Left => {
                        let mut a = self.registers.get(&Register::A);
                        bit = ((a & 0x80) >> 7) & 0x01;
                        let carry_bit = if self.registers.get_flag(Flag::Carry) {
                            1
                        } else {
                            0
                        };
                        a = ((a << 1) & 0xFE) | carry_bit;
                        self.registers.set(&Register::A, a);
                    }
                    RotateKind::RightCarry => {
                        let mut a = self.registers.get(&Register::A);
                        bit = a & 0x01;
                        a = ((a >> 1) & 0x8F) | (bit << 7);
                        self.registers.set(&Register::A, a);
                    }
                    RotateKind::Right => {
                        let mut a = self.registers.get(&Register::A);
                        bit = a & 0x01;
                        let carry_bit = if self.registers.get_flag(Flag::Carry) {
                            1
                        } else {
                            0
                        };
                        a = ((a >> 1) & 0x8F) | (carry_bit << 7);
                        self.registers.set(&Register::A, a);
                    }
                    _ => panic!("Encountered malformed rotate instruction: {}", i),
                }
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, false);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, false);
                self.registers.set_flag(Flag::Carry, bit == 1);
            }
            Instruction::DecimalAdjust => {
                // doc here -> https://ehaskins.com/2018-01-30%20Z80%20DAA/
                let a = self.registers.get(&Register::A);
                let mut nybble_1 = a & 0xF;
                let mut nybble_2 = ((a & 0xF0) >> 4) & 0xF;
                let mut a_adjusted;
                println!("Adjusting A: {:#x}", a);
                if self.registers.get_flag(Flag::Subtract) {
                    if nybble_1 > 0x9 || self.registers.get_flag(Flag::HalfCarry) {
                        nybble_1 -= 0x6;
                    }
                    if nybble_2 > 0x9 || self.registers.get_flag(Flag::Carry) {
                        nybble_2 -= 0x6;
                    }
                } else {
                    if nybble_1 > 0x9 || self.registers.get_flag(Flag::HalfCarry) {
                        nybble_1 += 0x6
                    }
                    if nybble_2 > 0x9 || self.registers.get_flag(Flag::Carry) {
                        nybble_2 += 0x6
                    }
                }
                a_adjusted = nybble_1 | (nybble_2 << 4);
                self.registers.set(&Register::A, a_adjusted);
                self.registers.set_flag(Flag::Carry, a_adjusted > 0x99);
                self.registers.set_flag(Flag::HalfCarry, false);
                self.registers.set_flag(Flag::Zero, a_adjusted == 0x0);
            }
            Instruction::Complement => {
                let a = self.registers.get(&Register::A);
                self.registers.set(&Register::A, !a);
                self.registers.set_flag(Flag::HalfCarry, true);
                self.registers.set_flag(Flag::Subtract, true);
            }
            Instruction::FlipCarry => {
                self.registers.set_flag(Flag::HalfCarry, false);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers
                    .set_flag(Flag::Carry, !self.registers.get_flag(Flag::Carry));
            }
            Instruction::Jump(kind) => match kind {
                JumpKind::JumpRelative(a8) => self.pc += *a8 as u16,
                JumpKind::JumpRelativeConditional(cond, a8) => match cond {
                    JumpCondition::Zero => {
                        if self.registers.get_flag(Flag::Zero) {
                            self.sp += *a8 as u16
                        }
                    }
                    JumpCondition::NonZero => {
                        if !self.registers.get_flag(Flag::Zero) {
                            self.sp += *a8 as u16
                        }
                    }
                    JumpCondition::Carry => {
                        if self.registers.get_flag(Flag::Carry) {
                            self.sp += *a8 as u16
                        }
                    }
                    JumpCondition::NonCarry => {
                        if !self.registers.get_flag(Flag::Carry) {
                            self.sp += *a8 as u16
                        }
                    }
                },
                JumpKind::Jump(a16) => self.pc = *a16,
                JumpKind::JumpConditional(cond, a16) => match cond {
                    JumpCondition::Zero => {
                        if self.registers.get_flag(Flag::Zero) {
                            self.pc = *a16
                        }
                    }
                    JumpCondition::NonZero => {
                        if !self.registers.get_flag(Flag::Zero) {
                            self.pc = *a16
                        }
                    }
                    JumpCondition::Carry => {
                        if self.registers.get_flag(Flag::Carry) {
                            self.pc = *a16
                        }
                    }
                    JumpCondition::NonCarry => {
                        if !self.registers.get_flag(Flag::Carry) {
                            self.pc = *a16
                        }
                    }
                },
                JumpKind::JumpHl => self.pc = self.registers.get_16bit(&RegisterPair::Hl),
            },
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
    #[test]
    fn execute_daa_after_sub() {
        let mut cpu = Cpu::new();
        cpu.registers.set(&Register::A, 0x47);
        cpu.registers.set(&Register::D, 0x28);
        cpu.execute(&Instruction::Sub(ArithmeticOperand::Register(Register::D)));
        cpu.execute(&Instruction::DecimalAdjust);
        assert_eq!(0x19, cpu.registers.get(&Register::A));
    }
    #[test]
    fn execute_daa_after_add() {
        let mut cpu = Cpu::new();
        cpu.registers.set(&Register::A, 0x47);
        cpu.registers.set(&Register::D, 0x28);
        cpu.execute(&Instruction::Add(ArithmeticOperand::Register(Register::D)));
        cpu.execute(&Instruction::DecimalAdjust);
        assert_eq!(0x75, cpu.registers.get(&Register::A));
    }
}
