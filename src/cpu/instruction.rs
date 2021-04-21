use std::fmt;

pub enum Instruction {
    Nop,
    Stop,
    Halt,
    Di,
    Ei,
    Ld16(Load16Target, Load16Source),
    Ld8(Load8Target, Load8Source),
    Add(ArithmeticTarget),
    Sub(ArithmeticTarget),
    AddCarry(ArithmeticTarget),
    SubCarry(ArithmeticTarget),
    And(ArithmeticTarget),
    Or(ArithmeticTarget),
    Xor(ArithmeticTarget),
    Cp(ArithmeticTarget),
    Increment(ArithmeticTarget),
    Decrement(ArithmeticTarget),
    AddPtr(AddPtrTarget, AddPtrTarget),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let instruction_string = match self {
            Nop => "NOP",
            Stop => "STOP",
            Halt => "HALT",
            Di => "DI",
            Ei => "EI",
            _ => "Not yet",
        };
        write!(f, "")
    }
}

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
            Bc => String::from("BC"),
            De => String::from("DE"),
            Hl => String::from("HL"),
            StackPointer => String::from("SP"),
            Load16Target::Address(data) => data.to_string(),
        };
        write!(f, "")
    }
}

pub enum Load16Source {
    Data(u16),
    SpPlus(i8),
    Hl,
}

pub enum Load8Target {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    Address(u16),
    AtC,
    AtHl,
    AtHli,
    AtHld,
}

pub enum Load8Source {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    Data(u8),
    AtC,
    AtHl,
}

pub enum ArithmeticTarget {
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

pub enum AddPtrTarget {
    Bc,
    De,
    Hl,
    StackPointer,
    Data(i8),
}
