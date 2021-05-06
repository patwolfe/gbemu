use std::process;

mod instruction;
mod registers;

use crate::cpu::instruction::*;
use crate::cpu::registers::*;
use crate::gb;
use crate::memory::Memory;
use crate::timer::Cycles;

pub struct Cpu {
    registers: Registers,
    pc: u16,
    sp: u16,
    pub memory: Memory,
    _current_instruction: (Instruction, u8),
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut memory = Memory::initialize();
        memory.write_byte(gb::lcd_stat, 0x02);
        Cpu {
            registers: Registers::new(),
            pc: 0, //gb::init_pc_value,
            sp: 0,
            memory,
            _current_instruction: (Instruction::Nop, 0),
        }
    }

    pub fn step(&mut self) -> u8 {
        let instruction = Instruction::from_bytes(&self.memory, self.pc);
        // println!("{:#0x}: {}, {}", self.pc, instruction, self.registers);
        self.execute(&instruction)
    }

    fn execute(&mut self, i: &Instruction) -> u8 {
        let (size, mut cycles) = Instruction::size_and_cycles(i);
        self.pc += size as u16;
        match i {
            Instruction::Nop => {}
            Instruction::Stop => {
                println!("Stopping program with instruction {}", i);
                process::exit(1);
            }
            Instruction::Halt => {
                println!("Halting program with instruction {}", i);
                process::exit(0);
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
                    (Load16Target::StackPointer, Load16Source::Data(d16)) => self.sp = *d16,
                    _ => panic!("Enounctered unimplemented load16 instruction: {}", i),
                };
            }
            Instruction::Load8(operand1, operand2) => match (operand1, operand2) {
                (Load8Operand::Register(target_reg), Load8Operand::Register(src_reg)) => {
                    let data = self.registers.get(src_reg);
                    self.registers.set(&target_reg, data);
                }
                (Load8Operand::Register(target_reg), Load8Operand::Data(d8)) => {
                    self.registers.set(&target_reg, *d8);
                }
                (Load8Operand::AtReg16(reg), Load8Operand::Register(src_reg)) => {
                    let data = self.registers.get(src_reg);
                    let address: u16 = self.registers.get_16bit(reg);
                    self.memory.write_byte(address, data);
                }
                (Load8Operand::Register(target_reg), Load8Operand::AtReg16(reg)) => {
                    let address: u16 = self.registers.get_16bit(reg);
                    let data = self.memory.read_byte(address);
                    self.registers.set(target_reg, data);
                }
                (Load8Operand::AtC, Load8Operand::Register(Register::A))
                | (Load8Operand::AtAddress8(_), Load8Operand::Register(Register::A))
                | (Load8Operand::AtAddress16(_), Load8Operand::Register(Register::A)) => {
                    let data = self.registers.get(&Register::A);
                    let address: u16 = if let Load8Operand::AtC = operand1 {
                        self.registers.get(&Register::C) as u16 + 0xFF00
                    } else if let Load8Operand::AtAddress8(a8) = operand1 {
                        *a8 as u16 + 0xFF00
                    } else if let Load8Operand::AtAddress16(a16) = operand1 {
                        *a16
                    } else {
                        panic!("Should not happen")
                    };
                    self.memory.write_byte(address, data);
                }
                (Load8Operand::Register(Register::A), Load8Operand::AtC)
                | (Load8Operand::Register(Register::A), Load8Operand::AtAddress8(_))
                | (Load8Operand::Register(Register::A), Load8Operand::AtAddress16(_)) => {
                    let address: u16 = if let Load8Operand::AtC = operand2 {
                        self.registers.get(&Register::C) as u16 + 0xFF00
                    } else if let Load8Operand::AtAddress8(a8) = operand2 {
                        *a8 as u16 + 0xFF00
                    } else if let Load8Operand::AtAddress16(a16) = operand2 {
                        *a16
                    } else {
                        panic!("Should not happen")
                    };
                    let data = self.memory.read_byte(address);
                    self.registers.set(&Register::A, data);
                }
                (Load8Operand::AtHld, Load8Operand::Register(reg))
                | (Load8Operand::AtHli, Load8Operand::Register(reg)) => {
                    let data = self.registers.get(reg);
                    let address: u16 = self.registers.get_16bit(&RegisterPair::Hl);
                    match operand1 {
                        Load8Operand::AtHld => {
                            self.registers.set_16bit(&RegisterPair::Hl, address - 1)
                        }
                        Load8Operand::AtHli => {
                            self.registers.set_16bit(&RegisterPair::Hl, address + 1)
                        }
                        _ => panic!("This can't happen"),
                    }
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
                let is_zero;
                let half_carry;
                let carry;
                match operand {
                    ArithmeticOperand::Register(reg) => {
                        is_zero = self.registers.get(&Register::A) == self.registers.get(reg);
                        half_carry = (self.registers.get(&Register::A) & 0xF)
                            < (self.registers.get(reg) & 0xF);
                        carry = self.registers.get(reg) > (self.registers.get(&Register::A));
                    }
                    ArithmeticOperand::AtHl => {
                        let at_hl = self
                            .memory
                            .read_byte(self.registers.get_16bit(&RegisterPair::Hl));
                        is_zero = self.registers.get(&Register::A) == at_hl;
                        half_carry = (self.registers.get(&Register::A) & 0xF) < (at_hl & 0xF);
                        carry = at_hl > self.registers.get(&Register::A);
                    }
                    ArithmeticOperand::Data(d8) => {
                        // need to read 1 byte for d8
                        is_zero = self.registers.get(&Register::A) == *d8;
                        half_carry = (self.registers.get(&Register::A) & 0xF) < (*d8 & 0xF);
                        carry = *d8 > self.registers.get(&Register::A);
                    }
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, is_zero);
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
                        let (diff, _) = self.registers.get(&reg).overflowing_sub(1);
                        result = diff;
                        half_carry = (self.registers.get(&reg) ^ 1 ^ result) & 0x10 == 0x10;
                        self.registers.set(&reg, result);
                    }
                    ArithmeticOperand::AtHl => {
                        // this takes 1 cycle
                        let hl_val = self
                            .memory
                            .read_byte(self.registers.get_16bit(&RegisterPair::Hl));
                        // 1 cycle
                        let (diff, _) = hl_val.overflowing_sub(1);
                        result = diff;
                        half_carry = (hl_val ^ 1 ^ result) & 0x10 == 0x10;
                        // 1 cycle
                        self.memory
                            .write_byte(self.registers.get_16bit(&RegisterPair::Hl), result)
                    }
                    _ => panic!("Enounctered malformed decrement instruction: {}", i),
                };
                // Set flags based on result
                self.registers.set_flag(Flag::Zero, result == 0);
                self.registers.set_flag(Flag::Subtract, true);
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
            Instruction::Rotate(kind) => match kind {
                RotateKind::LeftCircular => {
                    self.rotate(true, false, &ArithmeticOperand::Register(Register::A))
                }
                RotateKind::Left => {
                    self.rotate(true, true, &ArithmeticOperand::Register(Register::A))
                }
                RotateKind::RightCircular => {
                    self.rotate(false, false, &ArithmeticOperand::Register(Register::A))
                }
                RotateKind::Right => {
                    self.rotate(false, true, &ArithmeticOperand::Register(Register::A))
                }
            },
            Instruction::DecimalAdjust => {
                // doc here -> https://ehaskins.com/2018-01-30%20Z80%20DAA/
                let a = self.registers.get(&Register::A);
                let mut nybble_1 = a & 0xF;
                let mut nybble_2 = ((a & 0xF0) >> 4) & 0xF;
                let a_adjusted;
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
                JumpKind::JumpRelative(a8) => {
                    self.pc = (self.pc as i16 + i16::from(*a8)) as u16;
                }
                JumpKind::JumpRelativeConditional(cond, a8) => match cond {
                    // TODO: Refactor this part out into its own helper
                    // jump relative conditional / jump conditional fn
                    JumpCondition::Zero => {
                        if self.registers.get_flag(Flag::Zero) {
                            self.pc = (self.pc as i16 + i16::from(*a8)) as u16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::NonZero => {
                        if !self.registers.get_flag(Flag::Zero) {
                            self.pc = (self.pc as i16 + i16::from(*a8)) as u16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::Carry => {
                        if self.registers.get_flag(Flag::Carry) {
                            self.pc = (self.pc as i16 + i16::from(*a8)) as u16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::NonCarry => {
                        if !self.registers.get_flag(Flag::Carry) {
                            self.pc = (self.pc as i16 + i16::from(*a8)) as u16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                },
                JumpKind::Jump(a16) => self.pc = *a16,
                JumpKind::JumpConditional(cond, a16) => match cond {
                    JumpCondition::Zero => {
                        if self.registers.get_flag(Flag::Zero) {
                            self.pc = *a16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::NonZero => {
                        if !self.registers.get_flag(Flag::Zero) {
                            self.pc = *a16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::Carry => {
                        if self.registers.get_flag(Flag::Carry) {
                            self.pc = *a16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::NonCarry => {
                        if !self.registers.get_flag(Flag::Carry) {
                            self.pc = *a16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                },
                JumpKind::JumpHl => self.pc = self.registers.get_16bit(&RegisterPair::Hl),
            },
            Instruction::Return(kind) => match kind {
                ReturnKind::Return => self.pop(&Load16Target::StackPointer),
                ReturnKind::ReturnConditional(cond) => match cond {
                    JumpCondition::Zero => {
                        if self.registers.get_flag(Flag::Zero) {
                            self.pop(&Load16Target::StackPointer);
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::NonZero => {
                        if !self.registers.get_flag(Flag::Zero) {
                            self.pop(&Load16Target::StackPointer);
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::Carry => {
                        if self.registers.get_flag(Flag::Carry) {
                            self.pop(&Load16Target::StackPointer);
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::NonCarry => {
                        if !self.registers.get_flag(Flag::Carry) {
                            self.pop(&Load16Target::StackPointer);
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                },
                ReturnKind::ReturnInterrupt => {
                    self.pop(&Load16Target::StackPointer);
                    // TODO: Enable IME here
                }
            },
            Instruction::Pop(reg_pair) => self.pop(&Load16Target::Register16(*reg_pair)),
            Instruction::Push(reg_pair) => self.push(self.registers.get_16bit(reg_pair)),
            Instruction::DisableInterrupts => { /* TODO */ }
            Instruction::EnableInterrupts => { /* TODO */ }
            Instruction::Call(kind) => match kind {
                CallKind::Call(a16) => {
                    self.push(self.pc);
                    self.pc = *a16;
                }
                CallKind::CallConditional(a16, cond) => match cond {
                    JumpCondition::Zero => {
                        if self.registers.get_flag(Flag::Zero) {
                            self.push(self.pc);
                            self.pc = *a16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::NonZero => {
                        if !self.registers.get_flag(Flag::Zero) {
                            self.push(self.pc);
                            self.pc = *a16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::Carry => {
                        if self.registers.get_flag(Flag::Carry) {
                            self.push(self.pc);
                            self.pc = *a16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                    JumpCondition::NonCarry => {
                        if !self.registers.get_flag(Flag::Carry) {
                            self.push(self.pc);
                            self.pc = *a16;
                            cycles = Cpu::update_cycles(cycles, true)
                        } else {
                            cycles = Cpu::update_cycles(cycles, false)
                        }
                    }
                },
            },
            Instruction::Restart(offset) => {
                self.push(self.sp);
                self.pc = *offset as u16;
            }
            Instruction::Instruction16(in16) => self.execute_instruction_16(in16),
            _ => panic!("Enounctered unimplemented instruction: {}", i),
        };
        match cycles {
            Cycles::Cycles(n) => n,
            Cycles::ConditionalCycles(n, _) => n,
        }
    }

    fn pop(&mut self, target: &Load16Target) {
        let low_byte = self.memory.read_byte(self.sp + 1);
        let high_byte = self.memory.read_byte(self.sp + 2);
        self.sp += 2;
        let val = low_byte as u16 | ((high_byte as u16) << 8);
        if let Load16Target::StackPointer = target {
            self.pc = val;
        } else if let Load16Target::Register16(reg_pair) = target {
            self.registers.set_16bit(reg_pair, val);
        } else {
            panic!("Bad pop target: {}", target);
        }
    }
    fn push(&mut self, val: u16) {
        let low_byte = val & 0xFF;
        let high_byte = (val >> 8) & 0xFF;
        self.memory.write_byte(self.sp, high_byte as u8);
        self.memory.write_byte(self.sp - 1, low_byte as u8);
        self.sp -= 2;
    }

    fn execute_instruction_16(&mut self, instruction: &Instruction16) {
        match instruction {
            Instruction16::RotateLeftCircular(operand) => self.rotate(true, false, operand),
            Instruction16::RotateRightCircular(operand) => self.rotate(false, false, operand),
            Instruction16::RotateLeft(operand) => self.rotate(true, true, operand),
            Instruction16::RotateRight(operand) => self.rotate(false, true, operand),
            Instruction16::ShiftLeft(operand) => self.shift(true, false, operand),
            Instruction16::ShiftRightArithmetic(operand) => self.shift(false, false, operand),
            Instruction16::ShiftRightLogical(operand) => self.shift(false, true, operand),
            Instruction16::Swap(operand) => {
                let value = match operand {
                    ArithmeticOperand::Register(reg) => self.registers.get(reg),
                    ArithmeticOperand::AtHl => self
                        .memory
                        .read_byte(self.registers.get_16bit(&RegisterPair::Hl)),
                    _ => panic!("Bad operand for swap instruction {}", instruction),
                };
                let high_bits = (value >> 4) & 0xF;
                let low_bits = value & 0xF;
                let swapped_value = (low_bits << 4) | high_bits;
                match operand {
                    ArithmeticOperand::Register(reg) => self.registers.set(reg, swapped_value),
                    ArithmeticOperand::AtHl => self
                        .memory
                        .write_byte(self.registers.get_16bit(&RegisterPair::Hl), swapped_value),
                    _ => panic!("Bad operand for swap instruction {}", instruction),
                };

                self.registers.set_flag(Flag::Zero, value == 0);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, false);
                self.registers.set_flag(Flag::Carry, false);
            }
            Instruction16::BitComplement(index, operand) => {
                let value = match operand {
                    ArithmeticOperand::Register(reg) => self.registers.get(reg),
                    ArithmeticOperand::AtHl => self
                        .memory
                        .read_byte(self.registers.get_16bit(&RegisterPair::Hl)),
                    _ => panic!("Bad operand for bit instruction {}", instruction),
                };
                let bit = ((value & (0x1 << index)) >> index) & 0x1;
                self.registers.set_flag(Flag::Zero, bit == 0);
                self.registers.set_flag(Flag::Subtract, false);
                self.registers.set_flag(Flag::HalfCarry, true);
            }
            Instruction16::Reset(index, operand) => self.set_bit(*index, operand, 0),
            Instruction16::Set(index, operand) => self.set_bit(*index, operand, 1),
            _ => panic!("Bad instruciton16: {}", instruction),
        }
    }

    fn rotate(&mut self, left: bool, through_carry: bool, operand: &ArithmeticOperand) {
        let mut value = if let ArithmeticOperand::AtHl = operand {
            let addr = self.registers.get_16bit(&RegisterPair::Hl);
            self.memory.read_byte(addr)
        } else if let ArithmeticOperand::Register(reg) = operand {
            self.registers.get(reg)
        } else {
            panic!("Malformed rorate got to helper function")
        };

        let shifted_bit = if left {
            (value >> 7) & 0x1
        } else {
            value & 0x1
        };

        if left {
            value <<= 1;
        } else {
            value >>= 1;
        };

        let bit = if through_carry {
            self.registers.get_flag(Flag::Carry) as u8
        } else {
            shifted_bit
        };

        value |= if left { 0x01 & bit } else { 0x80 & (bit << 7) };

        if let ArithmeticOperand::AtHl = operand {
            let addr = self.registers.get_16bit(&RegisterPair::Hl);
            self.memory.write_byte(addr, value)
        } else if let ArithmeticOperand::Register(reg) = operand {
            self.registers.set(reg, value);
        } else {
            panic!("Malformed rotate got to helper function")
        };

        self.registers.set_flag(Flag::Zero, value == 0);
        self.registers.set_flag(Flag::Subtract, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, shifted_bit == 1);
    }

    fn shift(&mut self, left: bool, logical: bool, operand: &ArithmeticOperand) {
        let mut value = if let ArithmeticOperand::Register(reg) = operand {
            self.registers.get(reg)
        } else if let ArithmeticOperand::AtHl = operand {
            let addr = self.registers.get_16bit(&RegisterPair::Hl);
            self.memory.read_byte(addr)
        } else {
            panic!("Malformed shift in helper function");
        };
        let shifted_bit = if left {
            (value >> 7) & 0x1
        } else {
            value & 0x1
        };
        if left {
            value <<= 7;
        } else {
            let mask = if logical { 0x80 } else { 0x0 };
            value >>= 7;
            value &= mask;
        }

        if let ArithmeticOperand::Register(reg) = operand {
            self.registers.set(reg, value);
        } else if let ArithmeticOperand::AtHl = operand {
            let addr = self.registers.get_16bit(&RegisterPair::Hl);
            self.memory.write_byte(addr, value);
        };
        self.registers.set_flag(Flag::Zero, value == 0);
        self.registers.set_flag(Flag::Subtract, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, shifted_bit == 1);
    }

    fn set_bit(&mut self, index: u8, operand: &ArithmeticOperand, bit: u8) {
        let mut value = if let ArithmeticOperand::Register(reg) = operand {
            self.registers.get(reg)
        } else if let ArithmeticOperand::AtHl = operand {
            let addr = self.registers.get_16bit(&RegisterPair::Hl);
            self.memory.read_byte(addr)
        } else {
            panic!("Malformed shift in helper function");
        };
        value &= bit << index;
        if let ArithmeticOperand::AtHl = operand {
            let addr = self.registers.get_16bit(&RegisterPair::Hl);
            self.memory.write_byte(addr, value)
        } else if let ArithmeticOperand::Register(reg) = operand {
            self.registers.set(reg, value);
        }
    }

    fn update_cycles(cycles: Cycles, branch_taken: bool) -> Cycles {
        match cycles {
            Cycles::Cycles(n) => cycles,
            Cycles::ConditionalCycles(n, m) => {
                if branch_taken {
                    Cycles::Cycles(n)
                } else {
                    Cycles::Cycles(m)
                }
            }
        }
    }

    fn dump(&self) {
        println!("ROM:");
        println!("{:?}", self.memory.rom_bank0.iter().enumerate());
        println!("VRAM:");
        println!("{:?}", self.memory.vram);
    }
}

#[cfg(test)]
mod test {
    use super::*;
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
    #[test]
    fn check_flags_after_bit() {
        let mut cpu = Cpu::new();
        cpu.registers.set(&Register::A, 0x47);
        cpu.execute(&Instruction::Load16(
            Load16Target::Register16(RegisterPair::Hl),
            Load16Source::Data(0x9fff),
        ));
        cpu.execute(&Instruction::Instruction16(Instruction16::BitComplement(
            7,
            ArithmeticOperand::Register(Register::H),
        )));
        assert_eq!(false, cpu.registers.get_flag(Flag::Zero));
        assert_eq!(false, cpu.registers.get_flag(Flag::Subtract));
        assert_eq!(true, cpu.registers.get_flag(Flag::HalfCarry));
    }
    #[test]
    fn rotate_left_logical() {
        let mut cpu = Cpu::new();
        cpu.registers.set(&Register::C, 0xCE);
        cpu.execute(&Instruction::Instruction16(Instruction16::RotateLeft(
            ArithmeticOperand::Register(Register::C),
        )));
        assert_eq!(0x9C, cpu.registers.get(&Register::C));
        assert_eq!(true, cpu.registers.get_flag(Flag::Carry));
    }
    #[test]
    fn rotate_left_arithmetic() {
        let mut cpu = Cpu::new();
        cpu.registers.set_flag(Flag::Carry, true);
        cpu.registers.set(&Register::A, 0xCE);
        cpu.execute(&Instruction::Rotate(RotateKind::Left));
        assert_eq!(0x9D, cpu.registers.get(&Register::A));
        assert_eq!(true, cpu.registers.get_flag(Flag::Carry));
    }
    #[test]
    fn execute_push_pop() {
        let mut cpu = Cpu::new();
        cpu.sp = 0xFFFC;
        cpu.execute(&Instruction::Load16(
            Load16Target::Register16(RegisterPair::Hl),
            Load16Source::Data(0xABCD),
        ));
        cpu.execute(&Instruction::Push(RegisterPair::Hl));
        cpu.execute(&Instruction::Load16(
            Load16Target::Register16(RegisterPair::Hl),
            Load16Source::Data(0x0000),
        ));
        cpu.execute(&Instruction::Pop(RegisterPair::Hl));

        assert_eq!(0xABCD, cpu.registers.get_16bit(&RegisterPair::Hl));
    }

    #[test]
    pub fn output_bootrom() {
        let cpu = Cpu::new();
        let mut bytes_read = 0;
        while bytes_read < 0x00a7 {
            let instruction = Instruction::from_bytes(&cpu.memory, bytes_read);
            let (size, _cycles) = Instruction::size_and_cycles(&instruction);
            bytes_read += size as u16;
            println!("{}", instruction);
        }
        bytes_read = 0x00e0;
        while bytes_read < 0x0100 {
            let instruction = Instruction::from_bytes(&cpu.memory, bytes_read);
            let (size, _cycles) = Instruction::size_and_cycles(&instruction);
            bytes_read += size as u16;
            println!("{}", instruction);
        }
    }
}
