use std::{fmt, thread, time};

const CYCLE_LENGTH_US: u64 = 1;

pub fn sleep_to_cycles(n: u64, start: time::Instant) {
    let elapsed_micros = (time::Instant::now() - start).as_micros() as u64;
    thread::sleep(time::Duration::from_micros(
        (CYCLE_LENGTH_US * n) - elapsed_micros,
    ));
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
