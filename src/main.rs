extern crate minifb;
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

mod cpu;
mod gb;
mod memory;

use crate::cpu::Cpu;

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
    while window.is_open() && !window.is_key_down(Key::Escape) {
        cpu.step();
        // draw_screen(&cpu, &mut buffer);
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, gb::screen_width, gb::screen_height)
            .unwrap();
    }
}

fn draw_screen(cpu: &Cpu, buffer: &mut Vec<u32>) {
    for i in buffer.iter_mut() {
        *i = 1;
    }
}
