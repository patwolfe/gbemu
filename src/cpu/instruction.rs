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
    AddPtr(AddPtrOperand, AddPtrOperand),
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
            _ => String::from("Not yet"),
        };
        write!(f, "{}", instruction_string)
    }
}

impl Instruction {
    pub fn from_bytes(memory: &Memory, pc: u16) -> Instruction {
        let byte = memory.read_byte(pc);
        let high_bits = byte & 0xF0;
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
                    _ => panic!("Invalid opcode: {}", byte.is_ascii_hexdigit()),
                };
                let data: u16 =
                    (memory.read_byte(pc + 1) as u16) << 8 | memory.read_byte(pc + 2) as u16;
                Instruction::Load16(target, Load16Source::Data(data))
            }
            (0x0..=0x3, 0x2) => {
                let target = match high_bits {
                    0x0 => Load8Operand::AtReg16(RegisterPair::Bc),
                    0x1 => Load8Operand::AtReg16(RegisterPair::De),
                    0x2 => Load8Operand::AtHli,
                    0x3 => Load8Operand::AtHld,
                    _ => panic!("Invalid opcode: {}", byte),
                };
                Instruction::Load8(target, Load8Operand::Register(Register::A))
            }
            _ => panic!(),
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
            Load16Target::Address(data) => data.to_string(),
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
            Load16Source::Data(data) => data.to_string(),
        };
        write!(f, "{}", operand_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum Load8Operand {
    Register(Register),
    Address(u16),
    AtC,
    AtReg16(RegisterPair),
    AtHli,
    AtHld,
}

impl fmt::Display for Load8Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            Load8Operand::Register(reg) => std::format!("{}", reg),
            Load8Operand::AtC => String::from("(C)"),
            Load8Operand::AtReg16(reg_pair) => std::format!("({})", reg_pair),
            Load8Operand::AtHli => String::from("(HL+)"),
            Load8Operand::AtHld => String::from("(HL-)"),
            Load8Operand::Address(a16) => std::format!("({})", a16),
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
pub enum AddPtrOperand {
    Register16(RegisterPair),
    StackPointer,
    Data(i8),
}

impl fmt::Display for AddPtrOperand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            AddPtrOperand::Register16(reg_pair) => std::format!("({})", reg_pair),
            AddPtrOperand::StackPointer => String::from("SP"),
            AddPtrOperand::Data(i8) => std::format!("{}", i8),
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
                Instruction::AddPtr(AddPtrOperand::StackPointer, AddPtrOperand::Data(25))
            ),
            "ADD SP,25"
        );
    }
    #[test]
    fn decode_nop() {
        let memory = Memory::initialize();
        assert_eq!(Instruction::from_bytes(&memory, 0), Instruction::Nop);
    }
    #[test]
    fn decode_nop_fails() {
        let memory = Memory::initialize();
        assert_ne!(Instruction::from_bytes(&memory, 0), Instruction::Stop);
    }
    #[test]
    fn decode_ld8() {
        let mut memory = Memory::initialize();
        memory.bootrom[0] = 0x02;
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
        memory.bootrom[0] = 0x01;
        memory.bootrom[1] = 0xAB;
        memory.bootrom[2] = 0xCD;
        assert_eq!(
            Instruction::from_bytes(&memory, 0),
            Instruction::Load16(
                Load16Target::Register16(RegisterPair::Bc),
                Load16Source::Data(0xABCD)
            )
        );
    }
}
