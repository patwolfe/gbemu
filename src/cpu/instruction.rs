use std::fmt;

use crate::memory::Memory;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Nop,
    Stop,
    Halt,
    Di,
    Ei,
    Ld16(Load16Target, Load16Source),
    Ld8(Load8Operand, Load8Operand),
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
            Instruction::Ld16(target, source) => std::format!("LD {},{}", target, source),
            Instruction::Ld8(target, source) => std::format!("LD {},{}", target, source),
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
        match byte {
            0x00 => Instruction::Nop,
            0x10 => Instruction::Stop,
            _ => panic!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Load16Target {
    Bc,
    De,
    Hl,
    StackPointer,
    Address(u16),
}

impl fmt::Display for Load16Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            Load16Target::Bc => String::from("BC"),
            Load16Target::De => String::from("DE"),
            Load16Target::Hl => String::from("HL"),
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
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    Address(u16),
    AtC,
    AtBc,
    AtDe,
    AtHl,
    AtHli,
    AtHld,
}

impl fmt::Display for Load8Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            Load8Operand::A => String::from("A"),
            Load8Operand::B => String::from("B"),
            Load8Operand::C => String::from("C"),
            Load8Operand::D => String::from("D"),
            Load8Operand::E => String::from("E"),
            Load8Operand::H => String::from("H"),
            Load8Operand::L => String::from("L"),
            Load8Operand::AtC => String::from("(C)"),
            Load8Operand::AtBc => String::from("(BC)"),
            Load8Operand::AtDe => String::from("(DE)"),
            Load8Operand::AtHl => String::from("(HL)"),
            Load8Operand::AtHli => String::from("(HL+)"),
            Load8Operand::AtHld => String::from("(HL-)"),
            Load8Operand::Address(a16) => std::format!("({})", a16),
        };
        write!(f, "{}", operand_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum ArithmeticOperand {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    AtHl,
    Data(u8),
}

impl fmt::Display for ArithmeticOperand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            ArithmeticOperand::A => String::from("A"),
            ArithmeticOperand::B => String::from("B"),
            ArithmeticOperand::C => String::from("C"),
            ArithmeticOperand::D => String::from("D"),
            ArithmeticOperand::E => String::from("E"),
            ArithmeticOperand::H => String::from("H"),
            ArithmeticOperand::L => String::from("L"),
            ArithmeticOperand::AtHl => String::from("(HL)"),
            ArithmeticOperand::Data(u8) => std::format!("${}", u8),
        };
        write!(f, "{}", operand_string)
    }
}

#[derive(Debug, PartialEq)]
pub enum AddPtrOperand {
    Bc,
    De,
    Hl,
    StackPointer,
    Data(i8),
}

impl fmt::Display for AddPtrOperand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let operand_string = match self {
            AddPtrOperand::Bc => String::from("BC"),
            AddPtrOperand::De => String::from("DE"),
            AddPtrOperand::Hl => String::from("HL"),
            AddPtrOperand::StackPointer => String::from("SP"),
            AddPtrOperand::Data(i8) => std::format!("{}", i8),
        };
        write!(f, "{}", operand_string)
    }
}

mod tests {
    use crate::cpu::instruction::AddPtrOperand;
    use crate::cpu::instruction::ArithmeticOperand;
    use crate::cpu::instruction::Instruction;
    use crate::cpu::instruction::Load16Source;
    use crate::cpu::instruction::Load16Target;
    use crate::cpu::instruction::Load8Operand;
    use crate::memory::Memory;
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
            std::format!("{}", Instruction::Ld8(Load8Operand::A, Load8Operand::AtC)),
            "LD A,(C)"
        );
    }
    #[test]
    fn display_load16() {
        assert_eq!(
            std::format!(
                "{}",
                Instruction::Ld16(Load16Target::Hl, Load16Source::SpPlus(15))
            ),
            "LD HL,SP+15"
        );
    }
    #[test]
    fn display_add() {
        assert_eq!(
            std::format!("{}", Instruction::Add(ArithmeticOperand::B)),
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
            std::format!("{}", Instruction::AddCarry(ArithmeticOperand::B)),
            "ADC B"
        );
    }
    #[test]
    fn display_sbc() {
        assert_eq!(
            std::format!("{}", Instruction::SubCarry(ArithmeticOperand::D)),
            "SBC D"
        );
    }
    #[test]
    fn display_and() {
        assert_eq!(
            std::format!("{}", Instruction::And(ArithmeticOperand::B)),
            "AND B"
        );
    }
    #[test]
    fn display_or() {
        assert_eq!(
            std::format!("{}", Instruction::Or(ArithmeticOperand::C)),
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
            std::format!("{}", Instruction::Increment(ArithmeticOperand::A)),
            "INC A"
        );
    }
    #[test]
    fn display_dec() {
        assert_eq!(
            std::format!("{}", Instruction::Decrement(ArithmeticOperand::D)),
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
}
