use std::{thread, time};

static const CYCLE_LENGTH_US = 1;

pub fn sleep_for_cycles(n: u8) {
    thread::sleep(time::Duration::from_micros(CYCLE_LENGTH_US));
}
