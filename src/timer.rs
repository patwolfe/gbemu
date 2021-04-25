use std::{fmt, thread, time};

const CYCLE_LENGTH_US: u8 = 1;

pub fn sleep_for_cycles(n: u8) {
    thread::sleep(time::Duration::from_micros((CYCLE_LENGTH_US * n) as u64));
}

#[derive(Debug, PartialEq)]
pub enum Cycles {
    Cycles(u8),
    ConditionalCycles(u8, u8),
}

impl fmt::Display for Cycles {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cycles_string = match self {
            Cycles::Cycles(cycles) => std::format!("{}", cycles),
            Cycles::ConditionalCycles(cycles_taken, cycles_not_taken) => {
                std::format!("{}/{}", cycles_taken, cycles_not_taken)
            }
        };
        write!(f, "{}", cycles_string)
    }
}
