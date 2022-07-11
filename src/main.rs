use crate::cpu::CPU;
use minifb::{Scale, Window, WindowOptions};

mod cpu;
mod keypad;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() {
    let mut cpu = CPU::new();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut raw_instructions: Vec<u8> = Vec::new();
    let mut stop: bool = false;

    let mut window = Window::new(
        "Chip-8rs",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    )
    .expect("Window creation error");

    window.limit_update_rate(None);

    let rom_path = std::env::args().nth(1).expect("No ROM was provided!");
    if let Ok(ins) = std::fs::read(&rom_path) {
        raw_instructions = ins;
    }

    while window.is_open() {
        if !stop && !raw_instructions.is_empty() {
            cpu.run(&raw_instructions, &mut buffer, &mut window);
            stop = true;
        }

        window.update();
    }
}
