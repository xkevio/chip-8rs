use crate::cpu::CPU;
use minifb::{Scale, Window, WindowOptions};

mod cpu;
mod keypad;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Chip8 {
    pub cpu: CPU,
    pub buffer: Vec<u32>,
    pub rom: Vec<u8>,
    pub stop: bool,
}

fn main() {
    let mut chip8 = Chip8 {
        cpu: CPU::new(),
        buffer: vec![0; WIDTH * HEIGHT],
        rom: Vec::<u8>::new(),
        stop: false,
    };

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
    if let Ok(ins) = std::fs::read(rom_path) {
        chip8.rom = ins;
    }

    while window.is_open() {
        if !chip8.stop && !chip8.rom.is_empty() {
            chip8.cpu.run(&chip8.rom, &mut chip8.buffer, &mut window);
            chip8.stop = true;
        }

        window.update();
    }
}
