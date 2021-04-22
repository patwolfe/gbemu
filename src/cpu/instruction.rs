use std::fmt;

use crate::cpu::registers::Register;
use crate::cpu::registers::RegisterPair;
use crate::memory::Memory;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Nop,
    Stop,
    Halt,
    Di,
    Ei,
    Load16(Load16Target, Load16Source),
    Load8(Load8Operand, Load8Operand),
    Add(ArithmeticOperand),
    Sub(ArithmeticOperand),
    AddCarry(ArithmeticOperand),
    SubCarry(ArithmeticOperand),
    And(ArithmeticOperand),
    Or(ArithmeticOperand),
    Xor(ArithmeticOperand),
    Cp(ArithmeticOperand),
    Increment(ArithmeticOperand),
    Decrement(ArithmeticOperand),
    AddPtr(PtrArithOperand, PtrArithOperand),
    IncrementPtr(PtrArithOperand),
    DecrementPtr(PtrArithOperand),
    Jump(JumpKind),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let instruction_string = match self {
            Instruction::Nop => String::from("NOP"),
            Instruction::Stop => String::from("STOP"),
            Instruction::Halt => String::from("HALT"),
            Instruction::Di => String::from("DI"),
            Instruction::Ei => String::from("EI"),
            Instruction::Load16(target, source) => std::format!("LD {},{}", target, source),
            Instruction::Load8(target, source) => std::format!("LD {},{}", target, source),
            Instruction::Add(operand) => std::format!("ADD {}", operand),
            Instruction::Sub(operand) => std::format!("SUB {}", operand),
            Instruction::AddCarry(operand) => std::format!("ADC {}", operand),
            Instruction::SubCarry(operand) => std::format!("SBC {}", operand),
            Instruction::And(operand) => std::format!("AND {}", operand),
            Instruction::Or(operand) => std::format!("OR {}", operand),
            Instruction::Xor(operand) => std::format!("XOR {}", operand),
            Instruction::Cp(operand) => std::format!("CP {}", operand),
            Instruction::Increment(operand) => std::format!("INC {}", operand),
            Instruction::Decrement(operand) => std::format!("DEC {}", operand),
            Instruction::AddPtr(operand1, operand2) => {
                std::format!("ADD {},{}", operand1, operand2)
            }
            Instruction::IncrementPtr(operand) => std::format!("INC {}", operand),
            Instruction::DecrementPtr(operand) => std::format!("DEC {}", operand),
            Instruction::Jump(kind) => std::format!("J{}", kind),
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
            (0x0..=0x1, 0x7) => {
                let target = match high_bits {
                    0x0 => panic!("Haven't implemented RLCA yet"),
                    0x1 => panic!("Haven't implemented RLCA yet"),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
            }
            (0x2..=0x3, 0x7) => {
                let data = memory.read_byte(a + 1);
                let target = match high_bits {
                    0x2 => Load8Operand::Register(Register::H),
                    0x3 => Load8Operand::AtReg16(RegisterPair::Hl),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                };
                Instruction::Load8(target, Load8Operand::Data(data))
            }
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
                    0xB => Instruction::Cp(operand),
                    _ => panic!("Invalid opcode: {:#x}", byte),
                }
            }
            _ => panic!("Couldn't match opcode for {:#x}/{:#x}", high_bits, low_bits),
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
    Data(u16),
    SpPlus(i8),
    Hl,
}

impl fmt::Display for Load16Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            Load16Source::Hl => String::from("HL"),
            Load16Source::SpPlus(r8) => std::format!("SP+{}", r8),
            Load16Source::Data(data) => std::format!("${:#0x}", data),
        };
        write!(f, "{}", operand_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum Load8Operand {
    Register(Register),
    AtAddress(u16),
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
            Load8Operand::AtAddress(a16) => std::format!("({:#0x})", a16),
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
            std::format!("{}", Instruction::Cp(ArithmeticOperand::Data(10))),
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
        println!("{:?}", memory.rom_bank0);
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
    }
}
