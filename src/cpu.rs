use minifb::Window;
use rodio::{source::SineWave, OutputStream, OutputStreamHandle, Sink, Source};
use std::time::{Duration, Instant};

use crate::keypad;

pub struct CPU {
    memory: [u8; 4096],
    registers: [u8; 16],

    i_register: u16,
    delay_register: u8,
    sound_register: u8,

    program_counter: u16,
    stack_pointer: u8,
    stack: [u16; 16],
    pc_advance: bool,

    streams: (OutputStream, OutputStreamHandle),
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            memory: [0; 4096],
            registers: [0; 16],

            i_register: 0,
            delay_register: 0,
            sound_register: 0,

            program_counter: 512,
            stack_pointer: 0,
            stack: [0; 16],
            pc_advance: true,

            streams: OutputStream::try_default().expect("Error initializing audio devices!"),
        }
    }

    pub fn run(&mut self, instructions: &[u8], buffer: &mut [u32], window: &mut Window) {
        let mut d_c = 0;
        let mut s_c = 0;

        self.load_sprites();
        self.memory[512..(512 + instructions.len())].copy_from_slice(instructions);

        let ins_start = Instant::now();
        let mut counter = 0;
        let mut ins_c = 0u64;

        let sink = Sink::try_new(&self.streams.1).unwrap();
        sink.pause();

        loop {
            if !window.is_open() {
                break;
            }

            if self.delay_register > 0 {
                if d_c == 8 {
                    self.delay_register -= 1;
                    d_c = 0;
                } else {
                    d_c += 1;
                }
            }

            if self.sound_register > 0 {
                println!("beep!");

                if sink.is_paused() {
                    sink.play();
                    sink.append(
                        SineWave::new(220.0)
                            .take_duration(Duration::from_secs_f32(0.5))
                            .amplify(0.25),
                    );
                }

                if s_c == 8 {
                    self.sound_register -= 1;
                    s_c = 0;
                } else {
                    s_c += 1;
                }
            } else {
                sink.pause();
            }

            self.pc_advance = true;

            let (ins_a, ins_b) = (
                self.memory[self.program_counter as usize],
                self.memory[(self.program_counter + 1) as usize],
            );
            let ins: u16 = ((ins_a as u16) << 8) | ins_b as u16;

            let a = ((ins & 0xF000) >> 12) as u8;
            let b = ((ins & 0x0F00) >> 8) as u8;
            let c = ((ins & 0x00F0) >> 4) as u8;
            let d = ((ins & 0x000F) >> 0) as u8;

            let key_state = keypad::get_key_state(window);
            window.update();

            match (a, b, c, d) {
                (0x0, 0x0, 0xE, 0x0) => self.clear(buffer, window),
                (0x0, 0x0, 0xE, 0xE) => self.ret(),
                (0x1, _, _, _) => self.jmp(b, c, d),
                (0x2, _, _, _) => self.call(b, c, d),
                (0x3, _, _, _) => self.skip(b, c, d),
                (0x4, _, _, _) => self.skip_if_not(b, c, d),
                (0x5, _, _, 0x0) => self.skip_r(b, c),
                (0x6, _, _, _) => self.set_i(b, c, d),
                (0x7, _, _, _) => self.add_i(b, c, d),
                (0x8, _, _, 0x0) => self.store(b, c),
                (0x8, _, _, 0x1) => self.or(b, c),
                (0x8, _, _, 0x2) => self.and(b, c),
                (0x8, _, _, 0x3) => self.xor(b, c),
                (0x8, _, _, 0x4) => self.add(b, c),
                (0x8, _, _, 0x5) => self.sub(b, c),
                (0x8, _, _, 0x6) => self.shr(b, c),
                (0x8, _, _, 0x7) => self.subn(b, c),
                (0x8, _, _, 0xE) => self.shl(b, c),
                (0x9, _, _, 0x0) => self.skip_r_not(b, c),
                (0xA, _, _, _) => self.load_i(b, c, d),
                (0xB, _, _, _) => self.jmp_pc(b, c, d),
                (0xC, _, _, _) => self.rnd(b, c, d),
                (0xD, _, _, _) => self.draw(b, c, d, buffer, window),
                (0xE, _, 0x9, 0xE) => self.skip_if_key(b, window),
                (0xE, _, 0xA, 0x1) => self.skip_if_not_key(b, window),
                (0xF, _, 0x0, 0x7) => self.load_delay(b),
                (0xF, _, 0x0, 0xA) => self.wait_for_key(b, &key_state),
                (0xF, _, 0x1, 0x5) => self.set_delay(b),
                (0xF, _, 0x1, 0x8) => self.set_sound(b),
                (0xF, _, 0x1, 0xE) => self.add_to_i(b),
                (0xF, _, 0x2, 0x9) => self.set_i_to_sprite(b),
                (0xF, _, 0x3, 0x3) => self.store_bcd(b),
                (0xF, _, 0x5, 0x5) => self.reg_to_mem(b),
                (0xF, _, 0x6, 0x5) => self.mem_to_reg(b),
                _ => {
                    println!(
                        "Last op-code, avg: {}",
                        ins_c / ins_start.elapsed().as_secs()
                    );
                    break;
                }
            }

            if self.pc_advance {
                self.program_counter += 2;
            }

            ins_c += 1;

            counter += 1;
            if counter >= 10 {
                counter = 0;
                // spin_sleep::sleep(Duration::from_millis(15));
                let timer = Instant::now();
                spin_sleep::sleep(Duration::from_millis(20));
                println!("Slept for {}ms", timer.elapsed().as_millis());
            }

            // sleep for 2mil ns, freq: 500Hz, TODO
            // std::thread::sleep(Duration::from_micros(2000));
            // std::thread::sleep(Duration::from_micros((2000 - end) as u64));
        }
    }

    #[rustfmt::skip]
    fn load_sprites(&mut self) {
        let font: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0,
            0x20, 0x60, 0x20, 0x20, 0x70,
            0xF0, 0x10, 0xF0, 0x80, 0xF0,
            0xF0, 0x10, 0xF0, 0x10, 0xF0,
            0x90, 0x90, 0xF0, 0x10, 0x10,
            0xF0, 0x80, 0xF0, 0x10, 0xF0,
            0xF0, 0x80, 0xF0, 0x90, 0xF0,
            0xF0, 0x10, 0x20, 0x40, 0x40,
            0xF0, 0x90, 0xF0, 0x90, 0xF0,
            0xF0, 0x90, 0xF0, 0x10, 0xF0,
            0xF0, 0x90, 0xF0, 0x90, 0x90,
            0xE0, 0x90, 0xE0, 0x90, 0xE0,
            0xF0, 0x80, 0x80, 0x80, 0xF0,
            0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0,
            0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];

        self.memory[..font.len()].copy_from_slice(&font);
    }

    fn clear(&mut self, buffer: &mut [u32], window: &mut Window) {
        for b in &mut *buffer {
            *b = 0;
        }

        window.update_with_buffer(buffer, 64, 32).unwrap();
    }

    fn ret(&mut self) {
        self.program_counter = self.stack[self.stack_pointer as usize];
        self.pc_advance = false;
        self.stack_pointer -= 1;
    }

    fn jmp(&mut self, b: u8, c: u8, d: u8) {
        let loc: u16 = ((((b << 4) | c) as u16) << 4) | (d as u16);
        self.pc_advance = false;

        if self.program_counter == loc {
            // dont recurse
            self.memory[self.program_counter as usize] = 0u8;
            self.memory[(self.program_counter + 1) as usize] = 0u8;
        }

        self.program_counter = loc;
    }

    fn call(&mut self, b: u8, c: u8, d: u8) {
        let loc: u16 = ((((b << 4) | c) as u16) << 4) | (d as u16);
        self.stack_pointer += 1;
        self.stack[self.stack_pointer as usize] = self.program_counter + 2;
        self.program_counter = loc;
        self.pc_advance = false;
    }

    fn skip(&mut self, b: u8, c: u8, d: u8) {
        let kk: u8 = (c << 4) | d;
        if self.registers[b as usize] == kk {
            self.program_counter += 2;
        }
    }

    fn skip_if_not(&mut self, b: u8, c: u8, d: u8) {
        let kk: u8 = (c << 4) | d;
        if self.registers[b as usize] != kk {
            self.program_counter += 2;
        }
    }

    fn skip_r(&mut self, b: u8, c: u8) {
        if self.registers[b as usize] == self.registers[c as usize] {
            self.program_counter += 2;
        }
    }

    fn set_i(&mut self, b: u8, c: u8, d: u8) {
        let kk: u8 = (c << 4) | d;
        self.registers[b as usize] = kk;
    }

    fn add_i(&mut self, b: u8, c: u8, d: u8) {
        let kk: u8 = (c << 4) | d;
        self.registers[b as usize] += kk;
    }

    fn store(&mut self, b: u8, c: u8) {
        self.registers[b as usize] = self.registers[c as usize];
    }

    fn or(&mut self, b: u8, c: u8) {
        self.registers[b as usize] |= self.registers[c as usize];
    }

    fn and(&mut self, b: u8, c: u8) {
        self.registers[b as usize] &= self.registers[c as usize];
    }

    fn xor(&mut self, b: u8, c: u8) {
        self.registers[b as usize] ^= self.registers[c as usize];
    }

    fn add(&mut self, b: u8, c: u8) {
        let (result, carry) =
            self.registers[b as usize].overflowing_add(self.registers[c as usize]);
        self.registers[b as usize] = result;
        self.registers[0xF_usize] = carry as u8;
    }

    fn sub(&mut self, b: u8, c: u8) {
        if self.registers[b as usize] > self.registers[c as usize] {
            self.registers[0xF_usize] = 1;
        } else {
            self.registers[0xF_usize] = 0;
        }

        self.registers[b as usize] -= self.registers[c as usize];
    }

    fn shr(&mut self, b: u8, _c: u8) {
        let lsb: bool = (self.registers[b as usize] & 1) == 1;
        self.registers[0xF_usize] = lsb as u8;
        self.registers[b as usize] >>= 1;
    }

    fn subn(&mut self, b: u8, c: u8) {
        if self.registers[c as usize] > self.registers[b as usize] {
            self.registers[0xF_usize] = 1;
        } else {
            self.registers[0xF_usize] = 0;
        }

        self.registers[b as usize] = self.registers[c as usize] - self.registers[b as usize];
    }

    fn shl(&mut self, b: u8, _c: u8) {
        let msb: bool = ((self.registers[b as usize] >> 7) & 1) == 1;
        self.registers[0xF_usize] = msb as u8;
        self.registers[b as usize] <<= 1;
    }

    fn skip_r_not(&mut self, b: u8, c: u8) {
        if self.registers[b as usize] != self.registers[c as usize] {
            self.program_counter += 2;
        }
    }

    fn load_i(&mut self, b: u8, c: u8, d: u8) {
        let loc: u16 = ((((b << 4) | c) as u16) << 4) | (d as u16);
        self.i_register = loc;
    }

    fn jmp_pc(&mut self, b: u8, c: u8, d: u8) {
        let loc: u16 = ((((b << 4) | c) as u16) << 4) | (d as u16);
        self.program_counter = loc + self.registers[0] as u16;
        self.pc_advance = false;
    }

    fn rnd(&mut self, b: u8, c: u8, d: u8) {
        let rnd = rand::random::<u8>();
        self.registers[b as usize] = rnd & ((c << 4) | d);
    }

    fn draw(&mut self, b: u8, c: u8, d: u8, buffer: &mut [u32], window: &mut Window) {
        let x = self.registers[b as usize];
        let y = self.registers[c as usize];

        self.registers[0xF_usize] = 0;

        for off in 0..d {
            let mut bits = [0u8; 8];
            let n = self.memory[(self.i_register + off as u16) as usize];

            for shift in 0..8 {
                bits[shift] = if (n & (128 >> shift)) > 0 { 1 } else { 0 };
            }

            for i in 0..8 {
                let wrap_x = (x as u32 + i) % 64;
                let wrap_y = (y as u32 + off as u32) % 32;

                let prev_pix = buffer[(wrap_x + (wrap_y * 64)) as usize] as u8;
                let prev_to_bit = if prev_pix > 0 { 1 } else { 0 };

                let new_pix = bits[i as usize] ^ prev_to_bit;

                if new_pix < prev_to_bit {
                    self.registers[0xF_usize] = 1;
                }

                buffer[(wrap_x + (wrap_y * 64)) as usize] = if new_pix >= 1 { u32::MAX } else { 0 };
            }
        }

        window.update_with_buffer(buffer, 64, 32).unwrap();
    }

    fn skip_if_key(&mut self, b: u8, window: &Window) {
        let key_code = keypad::convert_reg_to_keys(self.registers[b as usize]);
        if window.is_key_down(key_code) {
            self.program_counter += 2;
        }
    }

    fn skip_if_not_key(&mut self, b: u8, window: &Window) {
        let key_code = keypad::convert_reg_to_keys(self.registers[b as usize]);
        if !window.is_key_down(key_code) {
            self.program_counter += 2;
        }
    }

    fn load_delay(&mut self, b: u8) {
        self.registers[b as usize] = self.delay_register;
    }

    fn wait_for_key(&mut self, b: u8, key_state: &[bool]) {
        // println!("Waiting for key, storing it in register 0x{b:02x}!");

        self.pc_advance = false;
        for i in 0..16 {
            if key_state[i] {
                self.registers[b as usize] = i as u8;
                self.pc_advance = true;
            }
        }
    }

    fn set_delay(&mut self, b: u8) {
        // decrease when non-zero
        self.delay_register = self.registers[b as usize];
    }

    fn set_sound(&mut self, b: u8) {
        // decrease when non-zero
        self.sound_register = self.registers[b as usize];
    }

    fn add_to_i(&mut self, b: u8) {
        self.i_register += self.registers[b as usize] as u16;
    }

    fn set_i_to_sprite(&mut self, b: u8) {
        self.i_register = (self.registers[b as usize] * 5) as u16;
    }

    fn store_bcd(&mut self, b: u8) {
        let mut num = self.registers[b as usize];
        let mut digits: Vec<u8> = Vec::new();

        while num > 0 {
            let digit = num % 10;
            num /= 10;

            digits.push(digit);
        }

        self.memory[self.i_register as usize] = *digits.get(2).unwrap_or(&0);
        self.memory[(self.i_register + 1) as usize] = *digits.get(1).unwrap_or(&0);
        self.memory[(self.i_register + 2) as usize] = *digits.get(0).unwrap_or(&0);
    }

    fn reg_to_mem(&mut self, b: u8) {
        for reg in 0..=b {
            self.memory[(self.i_register + reg as u16) as usize] = self.registers[reg as usize];
        }
    }

    fn mem_to_reg(&mut self, b: u8) {
        for reg in 0..=b {
            self.registers[reg as usize] = self.memory[(self.i_register + reg as u16) as usize];
        }
    }
}
