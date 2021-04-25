use std::fmt;

use crate::cpu::registers::Register;
use crate::cpu::registers::RegisterPair;
use crate::memory::Memory;
use crate::timer;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Nop,
    Stop,
    Halt,
    Load16(Load16Target, Load16Source),
    Load8(Load8Operand, Load8Operand),
    Add(ArithmeticOperand),
    Sub(ArithmeticOperand),
    AddCarry(ArithmeticOperand),
    SubCarry(ArithmeticOperand),
    And(ArithmeticOperand),
    Or(ArithmeticOperand),
    Xor(ArithmeticOperand),
    Compare(ArithmeticOperand),
    Increment(ArithmeticOperand),
    Decrement(ArithmeticOperand),
    AddPtr(PtrArithOperand, PtrArithOperand),
    IncrementPtr(PtrArithOperand),
    DecrementPtr(PtrArithOperand),
    Jump(JumpKind),
    EnableInterrupts,
    DisableInterrupts,
    Pop(RegisterPair),
    Push(RegisterPair),
    Return(ReturnKind),
    Rotate(RotateKind),
    SetCarryFlag,
    DecimalAdjust,
    Restart(u8),
    Call(CallKind),
    Instruction16(Instruction16),
    Complement,
    FlipCarry,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let instruction_string = match self {
            Instruction::Nop => String::from("NOP"),
            Instruction::Stop => String::from("STOP"),
            Instruction::Halt => String::from("HALT"),
            Instruction::Complement => String::from("CPL"),
            Instruction::FlipCarry => String::from("CCF"),
            Instruction::Load16(target, source) => std::format!("LD {},{}", target, source),
            Instruction::Load8(target, source) => std::format!("LD {},{}", target, source),
            Instruction::Add(operand) => std::format!("ADD {}", operand),
            Instruction::Sub(operand) => std::format!("SUB {}", operand),
            Instruction::AddCarry(operand) => std::format!("ADC {}", operand),
            Instruction::SubCarry(operand) => std::format!("SBC {}", operand),
            Instruction::And(operand) => std::format!("AND {}", operand),
            Instruction::Or(operand) => std::format!("OR {}", operand),
            Instruction::Xor(operand) => std::format!("XOR {}", operand),
            Instruction::Compare(operand) => std::format!("CP {}", operand),
            Instruction::Increment(operand) => std::format!("INC {}", operand),
            Instruction::Decrement(operand) => std::format!("DEC {}", operand),
            Instruction::AddPtr(operand1, operand2) => {
                std::format!("ADD {},{}", operand1, operand2)
            }
            Instruction::IncrementPtr(operand) => std::format!("INC {}", operand),
            Instruction::DecrementPtr(operand) => std::format!("DEC {}", operand),
            Instruction::Jump(kind) => std::format!("J{}", kind),
            Instruction::EnableInterrupts => String::from("EI"),
            Instruction::DisableInterrupts => String::from("DI"),
            Instruction::Pop(reg_pair) => std::format!("POP {}", reg_pair),
            Instruction::Push(reg_pair) => std::format!("PUSH {}", reg_pair),
            Instruction::Rotate(kind) => std::format!("R{}A", kind),
            Instruction::SetCarryFlag => String::from("SCF"),
            Instruction::DecimalAdjust => String::from("DAA"),
            Instruction::Call(kind) => std::format!("CALL {}", kind),
            Instruction::Restart(n) => std::format!("RST {}", n),
            Instruction::Return(kind) => std::format!("RET{}", kind),
            Instruction::Instruction16(instruction) => std::format!("{}", instruction),
        };
        write!(f, "{}", instruction_string)
    }
}

impl Instruction {
    pub fn from_bytes(memory: &Memory, a: u16) -> Instruction {
        let byte = memory.read_byte(a);
        let high_bits = (byte & 0xF0) >> 4;
        let low_bits = byte & 0x0F;
        match (high_bits, low_bits) {
            (0x0, 0x0) => Instruction::Nop,
            (0x1, 0x0) => Instruction::Stop,
            (0x2..=0x3, 0x0) => {
                let offset = memory.read_byte(a + 1);
                match high_bits {
                    0x2 => Instruction::Jump(JumpKind::JumpRelativeConditional(
                        JumpCondition::NonZero,
                        offset,
                    )),
                    0x3 => Instruction::Jump(JumpKind::JumpRelativeConditional(
                        JumpCondition::NonCarry,
                        offset,
                    )),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                }
            }
            (0x0..=0x3, 0x1) => {
                let target = match high_bits {
                    0x0 => Load16Target::Register16(RegisterPair::Bc),
                    0x1 => Load16Target::Register16(RegisterPair::De),
                    0x2 => Load16Target::Register16(RegisterPair::Hl),
                    0x3 => Load16Target::StackPointer,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                let data: u16 =
                    (memory.read_byte(a + 1) as u16) | (memory.read_byte(a + 2) as u16) << 8;
                Instruction::Load16(target, Load16Source::Data(data))
            }
            (0x0..=0x3, 0x2) => {
                let target = match high_bits {
                    0x0 => Load8Operand::AtReg16(RegisterPair::Bc),
                    0x1 => Load8Operand::AtReg16(RegisterPair::De),
                    0x2 => Load8Operand::AtHli,
                    0x3 => Load8Operand::AtHld,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Load8(target, Load8Operand::Register(Register::A))
            }
            (0x0..=0x3, 0x3) => {
                let operand = match high_bits {
                    0x0 => PtrArithOperand::Register16(RegisterPair::Bc),
                    0x1 => PtrArithOperand::Register16(RegisterPair::De),
                    0x2 => PtrArithOperand::Register16(RegisterPair::Hl),
                    0x3 => PtrArithOperand::StackPointer,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::IncrementPtr(operand)
            }
            (0x0..=0x3, 0x4..=0x5) => {
                let operand = match high_bits {
                    0x0 => ArithmeticOperand::Register(Register::B),
                    0x1 => ArithmeticOperand::Register(Register::D),
                    0x2 => ArithmeticOperand::Register(Register::H),
                    0x3 => ArithmeticOperand::AtHl,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                if let 0x4 = low_bits {
                    Instruction::Increment(operand)
                } else {
                    Instruction::Decrement(operand)
                }
            }
            (0x0..=0x3, 0x6) => {
                let data = memory.read_byte(a + 1);
                let target = match high_bits {
                    0x0 => Load8Operand::Register(Register::B),
                    0x1 => Load8Operand::Register(Register::D),
                    0x2 => Load8Operand::Register(Register::H),
                    0x3 => Load8Operand::AtReg16(RegisterPair::Hl),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Load8(target, Load8Operand::Data(data))
            }
            (0x0..=0x1, 0x7) => match high_bits {
                0x0 => Instruction::Rotate(RotateKind::LeftCarry),
                0x1 => Instruction::Rotate(RotateKind::Left),
                _ => panic!("Invalid opcode: {:#x}", byte),
            },
            (0x2..=0x3, 0x7) => match high_bits {
                0x2 => Instruction::DecimalAdjust,
                0x3 => Instruction::SetCarryFlag,
                _ => panic!("Invalid opcode: {:#x}", byte),
            },
            (0x0, 0x8) => {
                let address: u16 =
                    (memory.read_byte(a + 1) as u16) | (memory.read_byte(a + 2) as u16) << 8;
                Instruction::Load16(Load16Target::Address(address), Load16Source::StackPointer)
            }
            (0x1..=0x3, 0x8) => {
                let offset = memory.read_byte(a + 1);
                match high_bits {
                    0x1 => Instruction::Jump(JumpKind::JumpRelative(offset)),
                    0x2 => Instruction::Jump(JumpKind::JumpRelativeConditional(
                        JumpCondition::Zero,
                        offset,
                    )),
                    0x3 => Instruction::Jump(JumpKind::JumpRelativeConditional(
                        JumpCondition::Carry,
                        offset,
                    )),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                }
            }
            (0x0..=0x3, 0x9) => {
                let operand = match high_bits {
                    0x0 => PtrArithOperand::Register16(RegisterPair::Bc),
                    0x1 => PtrArithOperand::Register16(RegisterPair::De),
                    0x2 => PtrArithOperand::Register16(RegisterPair::Hl),
                    0x3 => PtrArithOperand::StackPointer,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::AddPtr(PtrArithOperand::Register16(RegisterPair::Hl), operand)
            }
            (0x0..=0x3, 0xA) => {
                let src = match high_bits {
                    0x0 => Load8Operand::AtReg16(RegisterPair::Bc),
                    0x1 => Load8Operand::AtReg16(RegisterPair::De),
                    0x2 => Load8Operand::AtHli,
                    0x3 => Load8Operand::AtHld,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Load8(Load8Operand::Register(Register::A), src)
            }
            (0x0..=0x3, 0xC..=0xD) => {
                let inc_dec_target = match high_bits {
                    0x0 => ArithmeticOperand::Register(Register::C),
                    0x1 => ArithmeticOperand::Register(Register::E),
                    0x2 => ArithmeticOperand::Register(Register::L),
                    0x3 => ArithmeticOperand::Register(Register::A),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                match low_bits {
                    0xC => Instruction::Increment(inc_dec_target),
                    0xD => Instruction::Decrement(inc_dec_target),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                }
            }
            (0x0..=0x3, 0xE) => {
                let data = memory.read_byte(a + 1);
                let target = match high_bits {
                    0x0 => Load8Operand::Register(Register::C),
                    0x1 => Load8Operand::Register(Register::E),
                    0x2 => Load8Operand::Register(Register::L),
                    0x3 => Load8Operand::Register(Register::A),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Load8(target, Load8Operand::Data(data))
            }
            (0x0..=0x3, 0xB) => {
                let operand = match high_bits {
                    0x0 => PtrArithOperand::Register16(RegisterPair::Bc),
                    0x1 => PtrArithOperand::Register16(RegisterPair::De),
                    0x2 => PtrArithOperand::Register16(RegisterPair::Hl),
                    0x3 => PtrArithOperand::StackPointer,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::DecrementPtr(operand)
            }
            (0x0..=0x1, 0xF) => match high_bits {
                0x0 => Instruction::Rotate(RotateKind::RightCarry),
                0x1 => Instruction::Rotate(RotateKind::Right),
                _ => panic!("Invalid opcode: {:#x}", byte),
            },
            (0x2..=0x3, 0xF) => match high_bits {
                0x2 => Instruction::Complement,
                0x3 => Instruction::FlipCarry,
                _ => panic!("Invalid opcode: {:#x}", byte),
            },
            (0x7, 0x6) => Instruction::Halt,
            (0x4..=0x7, 0x0..=0x7) => {
                let target = match high_bits {
                    0x4 => Load8Operand::Register(Register::B),
                    0x5 => Load8Operand::Register(Register::D),
                    0x6 => Load8Operand::Register(Register::H),
                    0x7 => Load8Operand::AtReg16(RegisterPair::Hl),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                let source = match low_bits {
                    0x0 => Load8Operand::Register(Register::B),
                    0x1 => Load8Operand::Register(Register::C),
                    0x2 => Load8Operand::Register(Register::D),
                    0x3 => Load8Operand::Register(Register::E),
                    0x4 => Load8Operand::Register(Register::H),
                    0x5 => Load8Operand::Register(Register::L),
                    0x6 => Load8Operand::AtReg16(RegisterPair::Hl),
                    0x7 => Load8Operand::Register(Register::A),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Load8(target, source)
            }
            (0x4..=0x7, 0x8..=0xF) => {
                let target = match high_bits {
                    0x4 => Load8Operand::Register(Register::C),
                    0x5 => Load8Operand::Register(Register::E),
                    0x6 => Load8Operand::Register(Register::L),
                    0x7 => Load8Operand::Register(Register::A),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                let source = match low_bits {
                    0x8 => Load8Operand::Register(Register::B),
                    0x9 => Load8Operand::Register(Register::C),
                    0xA => Load8Operand::Register(Register::D),
                    0xB => Load8Operand::Register(Register::E),
                    0xC => Load8Operand::Register(Register::H),
                    0xD => Load8Operand::Register(Register::L),
                    0xE => Load8Operand::AtReg16(RegisterPair::Hl),
                    0xF => Load8Operand::Register(Register::A),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Load8(target, source)
            }
            (0x8..=0xB, 0x0..=0x7) => {
                let operand = match low_bits {
                    0x0 => ArithmeticOperand::Register(Register::B),
                    0x1 => ArithmeticOperand::Register(Register::C),
                    0x2 => ArithmeticOperand::Register(Register::D),
                    0x3 => ArithmeticOperand::Register(Register::E),
                    0x4 => ArithmeticOperand::Register(Register::H),
                    0x5 => ArithmeticOperand::Register(Register::L),
                    0x6 => ArithmeticOperand::AtHl,
                    0x7 => ArithmeticOperand::Register(Register::A),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                match high_bits {
                    0x8 => Instruction::Add(operand),
                    0x9 => Instruction::Sub(operand),
                    0xA => Instruction::And(operand),
                    0xB => Instruction::Or(operand),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                }
            }
            (0x8..=0xB, 0x8..=0xF) => {
                let operand = match low_bits {
                    0x8 => ArithmeticOperand::Register(Register::B),
                    0x9 => ArithmeticOperand::Register(Register::C),
                    0xA => ArithmeticOperand::Register(Register::D),
                    0xB => ArithmeticOperand::Register(Register::E),
                    0xC => ArithmeticOperand::Register(Register::H),
                    0xD => ArithmeticOperand::Register(Register::L),
                    0xE => ArithmeticOperand::AtHl,
                    0xF => ArithmeticOperand::Register(Register::A),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                match high_bits {
                    0x8 => Instruction::AddCarry(operand),
                    0x9 => Instruction::SubCarry(operand),
                    0xA => Instruction::Xor(operand),
                    0xB => Instruction::Compare(operand),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                }
            }
            (0xC..=0xD, 0x0) => {
                let cond = match high_bits {
                    0xC => JumpCondition::NonZero,
                    0xD => JumpCondition::NonCarry,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Return(ReturnKind::ReturnConditional(cond))
            }
            (0xE..=0xF, 0x0) => {
                let a8 = memory.read_byte(a + 1);
                match high_bits {
                    0xE => Instruction::Load8(
                        Load8Operand::AtAddress8(a8),
                        Load8Operand::Register(Register::A),
                    ),
                    0xF => Instruction::Load8(
                        Load8Operand::Register(Register::A),
                        Load8Operand::AtAddress8(a8),
                    ),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                }
            }
            (0xC..=0xF, 0x1) => {
                let reg = match high_bits {
                    0xC => RegisterPair::Bc,
                    0xD => RegisterPair::De,
                    0xE => RegisterPair::Hl,
                    0xF => RegisterPair::Af,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Pop(reg)
            }
            (0xC..=0xD, 0x2) => {
                let a16: u16 =
                    (memory.read_byte(a + 1) as u16) | (memory.read_byte(a + 2) as u16) << 8;
                let cond = match high_bits {
                    0xC => JumpCondition::NonZero,
                    0xD => JumpCondition::NonCarry,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Jump(JumpKind::JumpConditional(cond, a16))
            }
            (0xE..=0xF, 0x2) => match high_bits {
                0xE => Instruction::Load8(Load8Operand::AtC, Load8Operand::Register(Register::A)),
                0xF => Instruction::Load8(Load8Operand::Register(Register::A), Load8Operand::AtC),
                _ => panic!("Invalid opcode: {:#x}", byte),
            },
            (0xC, 0x3) => {
                let a16: u16 =
                    (memory.read_byte(a + 1) as u16) | (memory.read_byte(a + 2) as u16) << 8;
                Instruction::Jump(JumpKind::Jump(a16))
            }
            (0xF, 0x3) => Instruction::DisableInterrupts,
            (0xF, 0xB) => Instruction::EnableInterrupts,
            (0xC..=0xD, 0x4) => {
                let a16: u16 = memory.read_2_bytes(a + 1);
                let cond = match high_bits {
                    0xC => JumpCondition::NonZero,
                    0xD => JumpCondition::NonCarry,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Call(CallKind::CallConditional(a16, cond))
            }
            (0xC..=0xF, 0x5) => {
                let reg = match high_bits {
                    0xC => RegisterPair::Bc,
                    0xD => RegisterPair::De,
                    0xE => RegisterPair::Hl,
                    0xF => RegisterPair::Af,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Push(reg)
            }
            (0xC..=0xF, 0x6) => {
                let d8 = memory.read_byte(a + 1);
                match high_bits {
                    0xC => Instruction::Add(ArithmeticOperand::Data(d8)),
                    0xD => Instruction::Sub(ArithmeticOperand::Data(d8)),
                    0xE => Instruction::And(ArithmeticOperand::Data(d8)),
                    0xF => Instruction::Or(ArithmeticOperand::Data(d8)),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                }
            }
            (0xC..=0xF, 0x7) => match high_bits {
                0xC => Instruction::Restart(0),
                0xD => Instruction::Restart(2),
                0xE => Instruction::Restart(4),
                0xF => Instruction::Restart(6),
                _ => panic!("Invalid opcode: {:#x}", byte),
            },
            (0xC..=0xD, 0x8) => {
                let cond = match high_bits {
                    0xC => JumpCondition::Zero,
                    0xD => JumpCondition::Carry,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Return(ReturnKind::ReturnConditional(cond))
            }
            (0xE, 0x8) => {
                let s8: i8 = memory.read_byte(a + 1) as i8;
                Instruction::AddPtr(PtrArithOperand::StackPointer, PtrArithOperand::Data(s8))
            }
            (0xC..=0xE, 0x9) => match high_bits {
                0xC => Instruction::Return(ReturnKind::Return),
                0xD => Instruction::Return(ReturnKind::ReturnInterrupt),
                0xE => Instruction::Jump(JumpKind::JumpHl),
                _ => panic!("Invalid opcode: {:#x}", byte),
            },
            (0xF, 0x8) => {
                let s8: i8 = memory.read_byte(a + 1) as i8;
                Instruction::Load16(
                    Load16Target::Register16(RegisterPair::Hl),
                    Load16Source::SpPlus(s8),
                )
            }
            (0xF, 0x9) => Instruction::Load16(Load16Target::StackPointer, Load16Source::Hl),
            (0xC..=0xD, 0xA) => {
                let a16: u16 = memory.read_2_bytes(a + 1);
                let cond = match high_bits {
                    0xC => JumpCondition::Zero,
                    0xD => JumpCondition::Carry,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Jump(JumpKind::JumpConditional(cond, a16))
            }
            (0xE..=0xF, 0xA) => {
                let a16: u16 = memory.read_2_bytes(a + 1);
                match high_bits {
                    0xE => Instruction::Load8(
                        Load8Operand::AtAddress16(a16),
                        Load8Operand::Register(Register::A),
                    ),
                    0xF => Instruction::Load8(
                        Load8Operand::Register(Register::A),
                        Load8Operand::AtAddress16(a16),
                    ),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                }
            }
            (0xC..=0xD, 0xC) => {
                let a16: u16 = memory.read_2_bytes(a + 1);
                let cond = match high_bits {
                    0xC => JumpCondition::Zero,
                    0xD => JumpCondition::Carry,
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Call(CallKind::CallConditional(a16, cond))
            }
            (0xC, 0xB) => {
                let suffix: u8 = memory.read_byte(a + 1);
                Instruction::Instruction16(Instruction::decode_i16_suffix(suffix))
            }
            (0xC, 0xD) => {
                let a16 = memory.read_2_bytes(a + 1);
                Instruction::Call(CallKind::Call(a16))
            }
            (0xC..=0xF, 0xE) => {
                let d8 = memory.read_byte(a + 1);
                match high_bits {
                    0xC => Instruction::AddCarry(ArithmeticOperand::Data(d8)),
                    0xD => Instruction::SubCarry(ArithmeticOperand::Data(d8)),
                    0xE => Instruction::Xor(ArithmeticOperand::Data(d8)),
                    0xF => Instruction::Compare(ArithmeticOperand::Data(d8)),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                }
            }
            (0xC..=0xF, 0xF) => match high_bits {
                0xC => Instruction::Restart(1),
                0xD => Instruction::Restart(3),
                0xE => Instruction::Restart(5),
                0xF => Instruction::Restart(7),
                _ => panic!("Invalid opcode: {:#x}", byte),
            },
            _ => panic!("Couldn't match opcode for {:#x}/{:#x}", high_bits, low_bits),
        }
    }
    fn decode_i16_suffix(suffix: u8) -> Instruction16 {
        let high_bits = (suffix & 0xF0) >> 4;
        let low_bits = suffix & 0x0F;
        let reg = match low_bits {
            0x0 | 0x8 => ArithmeticOperand::Register(Register::B),
            0x1 | 0x9 => ArithmeticOperand::Register(Register::C),
            0x2 | 0xA => ArithmeticOperand::Register(Register::D),
            0x3 | 0xB => ArithmeticOperand::Register(Register::E),
            0x4 | 0xC => ArithmeticOperand::Register(Register::H),
            0x5 | 0xD => ArithmeticOperand::Register(Register::L),
            0x6 | 0xE => ArithmeticOperand::AtHl,
            0x7 | 0xF => ArithmeticOperand::Register(Register::A),
            _ => panic!("Invalid 16 bit instruction suffix: {:#x}", suffix),
        };
        match low_bits {
            0x0..=0x7 => match high_bits {
                0x0 => Instruction16::RotateLeftCarry(reg),
                0x1 => Instruction16::RotateLeft(reg),
                0x2 => Instruction16::ShiftLeft(reg),
                0x3 => Instruction16::Swap(reg),
                0x4 => Instruction16::BitwiseComplement(0, reg),
                0x5 => Instruction16::BitwiseComplement(2, reg),
                0x6 => Instruction16::BitwiseComplement(4, reg),
                0x7 => Instruction16::BitwiseComplement(6, reg),
                0x8 => Instruction16::Reset(0, reg),
                0x9 => Instruction16::Reset(2, reg),
                0xA => Instruction16::Reset(4, reg),
                0xB => Instruction16::Reset(6, reg),
                0xC => Instruction16::Set(0, reg),
                0xD => Instruction16::Set(2, reg),
                0xE => Instruction16::Set(4, reg),
                0xF => Instruction16::Set(6, reg),
                _ => panic!("Invalid 16 bit instruction suffix: {:#x}", suffix),
            },
            0x8..=0xF => match high_bits {
                0x0 => Instruction16::RotateRightCarry(reg),
                0x1 => Instruction16::RotateRight(reg),
                0x2 => Instruction16::ShiftRightArithmetic(reg),
                0x3 => Instruction16::ShiftRightLogical(reg),
                0x4 => Instruction16::BitwiseComplement(1, reg),
                0x5 => Instruction16::BitwiseComplement(3, reg),
                0x6 => Instruction16::BitwiseComplement(5, reg),
                0x7 => Instruction16::BitwiseComplement(7, reg),
                0x8 => Instruction16::Reset(1, reg),
                0x9 => Instruction16::Reset(3, reg),
                0xA => Instruction16::Reset(5, reg),
                0xB => Instruction16::Reset(7, reg),
                0xC => Instruction16::Set(1, reg),
                0xD => Instruction16::Set(3, reg),
                0xE => Instruction16::Set(5, reg),
                0xF => Instruction16::Set(7, reg),
                _ => panic!("Invalid 16 bit instruction suffix: {:#x}", suffix),
            },
            _ => panic!("Invalid 16 bit instruction suffix: {:#x}", suffix),
        }
    }

    pub fn size_and_cycles(i: &Instruction) -> (u8, timer::Cycles) {
        match i {
            Instruction::Nop => (1, timer::Cycles::Cycles(1)),
            Instruction::Stop => (2, timer::Cycles::Cycles(1)),
            Instruction::Halt => (1, timer::Cycles::Cycles(1)),
            Instruction::Complement => (1, timer::Cycles::Cycles(1)),
            Instruction::FlipCarry => (1, timer::Cycles::Cycles(1)),
            Instruction::Load16(target, source) => match (target, source) {
                (_, Load16Source::Data(_)) => (3, timer::Cycles::Cycles(3)),
                (Load16Target::Address(_), Load16Source::StackPointer) => {
                    (3, timer::Cycles::Cycles(5))
                }
                (_, Load16Source::SpPlus(_)) => (2, timer::Cycles::Cycles(3)),
                _ => (1, timer::Cycles::Cycles(2)),
            },
            Instruction::Load8(target, source) => match (target, source) {
                (Load8Operand::Register(_), Load8Operand::Register(_)) => {
                    (1, timer::Cycles::Cycles(1))
                }
                (Load8Operand::AtHli, _) | (_, Load8Operand::AtHli) => {
                    (1, timer::Cycles::Cycles(2))
                }
                (Load8Operand::AtHld, _) | (_, Load8Operand::AtHld) => {
                    (1, timer::Cycles::Cycles(2))
                }
                (Load8Operand::Register(_), Load8Operand::Data(_)) => (2, timer::Cycles::Cycles(2)),
                (Load8Operand::AtReg16(RegisterPair::Hl), Load8Operand::Data(_)) => {
                    (2, timer::Cycles::Cycles(3))
                }
                (Load8Operand::AtReg16(_), Load8Operand::Register(_)) => {
                    (1, timer::Cycles::Cycles(2))
                }
                (Load8Operand::Register(_), Load8Operand::AtReg16(_)) => {
                    (1, timer::Cycles::Cycles(2))
                }
                (Load8Operand::AtC, Load8Operand::Register(_)) => (1, timer::Cycles::Cycles(2)),
                (Load8Operand::Register(_), Load8Operand::AtC) => (1, timer::Cycles::Cycles(2)),
                (Load8Operand::AtAddress8(_), _) | (_, Load8Operand::AtAddress8(_)) => {
                    (2, timer::Cycles::Cycles(3))
                }
                (Load8Operand::AtAddress16(_), _) | (_, Load8Operand::AtAddress16(_)) => {
                    (3, timer::Cycles::Cycles(4))
                }
                _ => panic!("Can't figure out size of {}", i),
            },
            // All arithmetic ops share size/cycle properties based on operand
            Instruction::Add(operand)
            | Instruction::Sub(operand)
            | Instruction::AddCarry(operand)
            | Instruction::SubCarry(operand)
            | Instruction::And(operand)
            | Instruction::Or(operand)
            | Instruction::Xor(operand)
            | Instruction::Compare(operand) => match operand {
                ArithmeticOperand::Register(_) => (1, timer::Cycles::Cycles(1)),
                ArithmeticOperand::AtHl => (1, timer::Cycles::Cycles(2)),
                ArithmeticOperand::Data(_) => (2, timer::Cycles::Cycles(2)),
            },
            Instruction::Increment(target) | Instruction::Decrement(target) => match target {
                ArithmeticOperand::Register(_) => (1, timer::Cycles::Cycles(1)),
                ArithmeticOperand::AtHl => (1, timer::Cycles::Cycles(3)),
                _ => panic!("Can't figure out size of {}", i),
            },
            Instruction::AddPtr(operand1, _) => match operand1 {
                PtrArithOperand::Register16(_) => (1, timer::Cycles::Cycles(2)),
                PtrArithOperand::StackPointer => (2, timer::Cycles::Cycles(4)),
                _ => panic!("Can't figure out size of {}", i),
            },
            Instruction::IncrementPtr(_) | Instruction::DecrementPtr(_) => {
                (1, timer::Cycles::Cycles(2))
            }
            Instruction::Jump(kind) => match kind {
                JumpKind::Jump(_) => (3, timer::Cycles::Cycles(4)),
                JumpKind::JumpConditional(_, _) => (3, timer::Cycles::ConditionalCycles(4, 3)),
                JumpKind::JumpRelative(_) => (2, timer::Cycles::Cycles(4)),
                JumpKind::JumpRelativeConditional(_, _) => {
                    (2, timer::Cycles::ConditionalCycles(3, 2))
                }
                JumpKind::JumpHl => (1, timer::Cycles::Cycles(1)),
            },
            Instruction::EnableInterrupts => (1, timer::Cycles::Cycles(1)),
            Instruction::DisableInterrupts => (1, timer::Cycles::Cycles(1)),
            Instruction::Pop(_) => (1, timer::Cycles::Cycles(3)),
            Instruction::Push(_) => (1, timer::Cycles::Cycles(3)),
            Instruction::Rotate(_) => (1, timer::Cycles::Cycles(1)),
            Instruction::SetCarryFlag => (1, timer::Cycles::Cycles(1)),
            Instruction::DecimalAdjust => (1, timer::Cycles::Cycles(1)),
            Instruction::Call(kind) => match kind {
                CallKind::CallConditional(_, _) => (3, timer::Cycles::ConditionalCycles(6, 3)),
                CallKind::Call(_) => (3, timer::Cycles::Cycles(6)),
            },
            Instruction::Restart(_) => (1, timer::Cycles::Cycles(4)),
            Instruction::Return(kind) => match kind {
                ReturnKind::Return | ReturnKind::ReturnConditional(_) => {
                    (1, timer::Cycles::ConditionalCycles(5, 2))
                }
                ReturnKind::ReturnInterrupt => (1, timer::Cycles::Cycles(4)),
            },
            // TODO: Need to change this so (HL) 16 bit instructions take 4 cycles
            Instruction::Instruction16(_) => (2, timer::Cycles::Cycles(2)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Load16Target {
    Register16(RegisterPair),
    StackPointer,
    Address(u16),
}

impl fmt::Display for Load16Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            Load16Target::Register16(reg_pair) => std::format!("{}", reg_pair),
            Load16Target::StackPointer => String::from("SP"),
            Load16Target::Address(data) => std::format!("${:#0x}", data),
        };
        write!(f, "{}", operand_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum Load16Source {
    StackPointer,
    Data(u16),
    SpPlus(i8),
    Hl,
}

impl fmt::Display for Load16Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            Load16Source::Hl => String::from("HL"),
            Load16Source::StackPointer => String::from("SP"),
            Load16Source::SpPlus(r8) => std::format!("SP+{}", r8),
            Load16Source::Data(data) => std::format!("${:#0x}", data),
        };
        write!(f, "{}", operand_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum Load8Operand {
    Register(Register),
    AtAddress16(u16),
    AtAddress8(u8),
    AtC,
    AtReg16(RegisterPair),
    AtHli,
    AtHld,
    Data(u8),
}

impl fmt::Display for Load8Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            Load8Operand::Register(reg) => std::format!("{}", reg),
            Load8Operand::AtC => String::from("(C)"),
            Load8Operand::AtReg16(reg_pair) => std::format!("({})", reg_pair),
            Load8Operand::AtHli => String::from("(HL+)"),
            Load8Operand::AtHld => String::from("(HL-)"),
            Load8Operand::AtAddress16(a16) => std::format!("({:#0x})", a16),
            Load8Operand::AtAddress8(a8) => std::format!("({:#0x})", a8),
            Load8Operand::Data(data) => std::format!("${:#0x}", data),
        };
        write!(f, "{}", operand_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum ArithmeticOperand {
    Register(Register),
    AtHl,
    Data(u8),
}

impl fmt::Display for ArithmeticOperand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            ArithmeticOperand::Register(reg) => std::format!("{}", reg),
            ArithmeticOperand::AtHl => String::from("(HL)"),
            ArithmeticOperand::Data(u8) => std::format!("${}", u8),
        };
        write!(f, "{}", operand_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum PtrArithOperand {
    Register16(RegisterPair),
    StackPointer,
    Data(i8),
}

impl fmt::Display for PtrArithOperand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            PtrArithOperand::Register16(reg_pair) => std::format!("({})", reg_pair),
            PtrArithOperand::StackPointer => String::from("SP"),
            PtrArithOperand::Data(i8) => std::format!("${}", i8),
        };
        write!(f, "{}", operand_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum JumpKind {
    JumpRelative(u8),
    JumpRelativeConditional(JumpCondition, u8),
    Jump(u16),
    JumpConditional(JumpCondition, u16),
    JumpHl,
}

impl fmt::Display for JumpKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind = match self {
            JumpKind::JumpRelative(offset) => std::format!("R {}", offset),
            JumpKind::JumpRelativeConditional(cond, offset) => {
                std::format!("R {} {}", cond, offset)
            }
            JumpKind::Jump(address) => std::format!("P {}", address),
            JumpKind::JumpConditional(cond, address) => std::format!("P {} {}", cond, address),
            JumpKind::JumpHl => String::from("P (HL)"),
        };
        write!(f, "{}", kind)
    }
}

#[derive(Debug, PartialEq)]
pub enum JumpCondition {
    Zero,
    NonZero,
    Carry,
    NonCarry,
}

impl fmt::Display for JumpCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            JumpCondition::Zero => String::from("Z"),
            JumpCondition::NonZero => String::from("NZ"),
            JumpCondition::Carry => String::from("C"),
            JumpCondition::NonCarry => String::from("NC"),
        };
        write!(f, "{}", operand_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum ReturnKind {
    Return,
    ReturnInterrupt,
    ReturnConditional(JumpCondition),
}

impl fmt::Display for ReturnKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind_string = match self {
            ReturnKind::Return => String::from(""),
            ReturnKind::ReturnInterrupt => String::from("I"),
            ReturnKind::ReturnConditional(cond) => std::format!(" {}", cond),
        };
        write!(f, "{}", kind_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum Instruction16 {
    RotateLeftCarry(ArithmeticOperand),
    RotateLeft(ArithmeticOperand),
    RotateRightCarry(ArithmeticOperand),
    RotateRight(ArithmeticOperand),
    ShiftLeft(ArithmeticOperand),
    ShiftRightArithmetic(ArithmeticOperand),
    ShiftRightLogical(ArithmeticOperand),
    Swap(ArithmeticOperand),
    BitwiseComplement(u8, ArithmeticOperand),
    Reset(u8, ArithmeticOperand),
    Set(u8, ArithmeticOperand),
}

impl fmt::Display for Instruction16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind_string = match self {
            Instruction16::RotateLeftCarry(reg) => std::format!("RLC {}", reg),
            Instruction16::RotateLeft(reg) => std::format!("RL {}", reg),
            Instruction16::RotateRightCarry(reg) => std::format!("RRC {}", reg),
            Instruction16::RotateRight(reg) => std::format!("RR {}", reg),
            Instruction16::ShiftLeft(reg) => std::format!("SLA {}", reg),
            Instruction16::ShiftRightArithmetic(reg) => std::format!("SRA {}", reg),
            Instruction16::ShiftRightLogical(reg) => std::format!("SRL {}", reg),
            Instruction16::Swap(reg) => std::format!("SWAP {}", reg),
            Instruction16::BitwiseComplement(val, reg) => std::format!("BIT {} {}", val, reg),
            Instruction16::Reset(val, reg) => std::format!("BIT {} {}", val, reg),
            Instruction16::Set(val, reg) => std::format!("Set {} {}", val, reg),
        };
        write!(f, "{}", kind_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum RotateKind {
    Left,
    LeftCarry,
    Right,
    RightCarry,
}

impl fmt::Display for RotateKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind_string = match self {
            RotateKind::Left => String::from("L"),
            RotateKind::LeftCarry => String::from("LC"),
            RotateKind::Right => String::from("R"),
            RotateKind::RightCarry => String::from("RC"),
        };
        write!(f, "{}", kind_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum CallKind {
    Call(u16),
    CallConditional(u16, JumpCondition),
}

impl fmt::Display for CallKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind_string = match self {
            CallKind::Call(a16) => std::format!("{}", a16),
            CallKind::CallConditional(a16, cond) => std::format!("{} {}", a16, cond),
        };
        write!(f, "{}", kind_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_nop() {
        assert_eq!(std::format!("{}", Instruction::Nop), "NOP");
    }
    #[test]
    fn display_halt() {
        assert_eq!(std::format!("{}", Instruction::Halt), "HALT");
    }
    #[test]
    fn display_load8() {
        assert_eq!(
            std::format!(
                "{}",
                Instruction::Load8(Load8Operand::Register(Register::A), Load8Operand::AtC)
            ),
            "LD A,(C)"
        );
    }
    #[test]
    fn display_load16() {
        assert_eq!(
            std::format!(
                "{}",
                Instruction::Load16(
                    Load16Target::Register16(RegisterPair::Hl),
                    Load16Source::SpPlus(15)
                )
            ),
            "LD HL,SP+15"
        );
    }
    #[test]
    fn display_add() {
        assert_eq!(
            std::format!(
                "{}",
                Instruction::Add(ArithmeticOperand::Register(Register::B))
            ),
            "ADD B"
        );
    }
    #[test]
    fn display_sub() {
        assert_eq!(
            std::format!("{}", Instruction::Sub(ArithmeticOperand::AtHl)),
            "SUB (HL)"
        );
    }
    #[test]
    fn display_adc() {
        assert_eq!(
            std::format!(
                "{}",
                Instruction::AddCarry(ArithmeticOperand::Register(Register::B))
            ),
            "ADC B"
        );
    }
    #[test]
    fn display_sbc() {
        assert_eq!(
            std::format!(
                "{}",
                Instruction::SubCarry(ArithmeticOperand::Register(Register::D))
            ),
            "SBC D"
        );
    }
    #[test]
    fn display_and() {
        assert_eq!(
            std::format!(
                "{}",
                Instruction::And(ArithmeticOperand::Register(Register::B))
            ),
            "AND B"
        );
    }
    #[test]
    fn display_or() {
        assert_eq!(
            std::format!(
                "{}",
                Instruction::Or(ArithmeticOperand::Register(Register::C))
            ),
            "OR C"
        );
    }
    #[test]
    fn display_xor() {
        assert_eq!(
            std::format!("{}", Instruction::Xor(ArithmeticOperand::Data(10))),
            "XOR $10"
        );
    }
    #[test]
    fn display_cp() {
        assert_eq!(
            std::format!("{}", Instruction::Compare(ArithmeticOperand::Data(10))),
            "CP $10"
        );
    }
    #[test]
    fn display_inc() {
        assert_eq!(
            std::format!(
                "{}",
                Instruction::Increment(ArithmeticOperand::Register(Register::A))
            ),
            "INC A"
        );
    }
    #[test]
    fn display_dec() {
        assert_eq!(
            std::format!(
                "{}",
                Instruction::Decrement(ArithmeticOperand::Register(Register::D))
            ),
            "DEC D"
        );
    }
    #[test]
    fn display_addptr() {
        assert_eq!(
            std::format!(
                "{}",
                Instruction::AddPtr(PtrArithOperand::StackPointer, PtrArithOperand::Data(25))
            ),
            "ADD SP,$25"
        );
    }
    #[test]
    fn decode_nop() {
        let mut memory = Memory::initialize();
        memory.rom_bank0[0] = 0;
        assert_eq!(Instruction::from_bytes(&memory, 0), Instruction::Nop);
    }
    #[test]
    fn decode_nop_fails() {
        let memory = Memory::initialize();
        assert_ne!(Instruction::from_bytes(&memory, 0), Instruction::Stop);
    }
    #[test]
    fn decode_stop() {
        let mut memory = Memory::initialize();
        memory.rom_bank0[0] = 0x10;
        assert_eq!(Instruction::from_bytes(&memory, 0), Instruction::Stop);
    }
    #[test]
    fn decode_ld8() {
        let mut memory = Memory::initialize();
        memory.rom_bank0[0] = 0x02;
        assert_eq!(
            Instruction::from_bytes(&memory, 0),
            Instruction::Load8(
                Load8Operand::AtReg16(RegisterPair::Bc),
                Load8Operand::Register(Register::A)
            )
        );
    }
    #[test]
    fn decode_ld16() {
        let mut memory = Memory::initialize();
        memory.rom_bank0[0] = 0x01;
        memory.rom_bank0[1] = 0xCD;
        memory.rom_bank0[2] = 0xAB;
        assert_eq!(
            Instruction::from_bytes(&memory, 0),
            Instruction::Load16(
                Load16Target::Register16(RegisterPair::Bc),
                Load16Source::Data(0xABCD)
            )
        );
    }
    #[test]
    fn decode_inc16() {
        let mut memory = Memory::initialize();
        memory.rom_bank0[0] = 0x23;
        assert_eq!(
            Instruction::from_bytes(&memory, 0),
            Instruction::IncrementPtr(PtrArithOperand::Register16(RegisterPair::Hl))
        );
    }
    #[test]
    fn decode_inc() {
        let mut memory = Memory::initialize();
        memory.rom_bank0[0] = 0x24;
        assert_eq!(
            Instruction::from_bytes(&memory, 0),
            Instruction::Increment(ArithmeticOperand::Register(Register::H))
        );
    }
    #[test]
    fn decode_dec() {
        let mut memory = Memory::initialize();
        memory.rom_bank0[0] = 0x35;
        assert_eq!(
            Instruction::from_bytes(&memory, 0),
            Instruction::Decrement(ArithmeticOperand::AtHl)
        );
    }
    #[test]
    pub fn decode_bootrom() {
        let memory = Memory::initialize();
        assert_eq!(
            Instruction::from_bytes(&memory, 0),
            Instruction::Load16(Load16Target::StackPointer, Load16Source::Data(0xFFFE))
        );
        assert_eq!(
            Instruction::from_bytes(&memory, 3),
            Instruction::Xor(ArithmeticOperand::Register(Register::A))
        );
        assert_eq!(
            Instruction::from_bytes(&memory, 4),
            Instruction::Load16(
                Load16Target::Register16(RegisterPair::Hl),
                Load16Source::Data(0x9FFF)
            )
        );
        assert_eq!(
            Instruction::from_bytes(&memory, 7),
            Instruction::Load8(Load8Operand::AtHld, Load8Operand::Register(Register::A))
        );
        assert_eq!(
            Instruction::from_bytes(&memory, 8),
            Instruction::Instruction16(Instruction16::BitwiseComplement(
                7,
                ArithmeticOperand::Register(Register::H)
            ))
        );
    }

    #[test]
    pub fn load16_metadat() {
        assert_eq!(
            (3, timer::Cycles::Cycles(3)),
            Instruction::size_and_cycles(&Instruction::Load16(
                Load16Target::Register16(RegisterPair::Hl),
                Load16Source::Data(0x9FFF)
            ))
        );
    }
}
