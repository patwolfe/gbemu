use std::{fmt, thread, time};

const MICRO_PER_FRAME: u64 = 16667;

pub fn sleep_to_frame_end(start: time::Instant) {
    let elapsed_micros = (time::Instant::now() - start).as_micros() as u64;
    println!("{} have passed", elapsed_micros);
    thread::sleep(time::Duration::from_micros(
        MICRO_PER_FRAME - elapsed_micros,
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
