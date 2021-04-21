use std::time;

struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    f: u8,
    pc: u16,
    sp: u16,
}

struct CPU {
    registers: Registers,
}
