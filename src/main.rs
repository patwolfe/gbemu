extern crate minifb;
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use std::time::Instant;

mod cpu;
mod gb;
mod memory;
mod ppu;
mod timer;

use crate::cpu::Cpu;
use crate::ppu::Ppu;

fn main() {
    let mut window = Window::new(
        "Test - ESC to exit",
        gb::screen_width,
        gb::screen_height,
        WindowOptions {
            borderless: false,
            title: true,
            resize: true,
            scale: Scale::X4,
            scale_mode: ScaleMode::UpperLeft,
            topmost: true,
            transparency: false,
            none: false,
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    let mut buffer: Vec<u32> = vec![0; gb::total_pixels];

    let mut cpu = Cpu::new();
    let mut ppu = Ppu::new();
    let mut cycles_taken = 0;
    #[allow(clippy::never_loop)]
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start_time = Instant::now();
        while cycles_taken < gb::cycles_per_frame {
            let cycles_instruction = cpu.step() as u32;
            ppu.step(cycles_instruction, &mut cpu.memory, &mut buffer);
            cycles_taken += cycles_instruction;
        }
        cycles_taken %= gb::cycles_per_frame;
        window
            .update_with_buffer(&buffer, gb::screen_width, gb::screen_height)
            .unwrap();

        timer::sleep_to_frame_end(start_time);
    }
}
