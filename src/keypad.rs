use minifb::{Key, Window};

pub fn get_key_state(window: &Window) -> [bool; 16] {
    let mut state = [false; 16];

    for i in 0..16 {
        state[i] = window.is_key_down(convert_reg_to_keys(i as u8));
    }

    state
}

pub fn convert_reg_to_keys(value: u8) -> Key {
    match value {
        0x1 => Key::Key1,
        0x2 => Key::Key2,
        0x3 => Key::Key3,
        0xC => Key::Key4,
        0x4 => Key::Q,
        0x5 => Key::W,
        0x6 => Key::E,
        0xD => Key::R,
        0x7 => Key::A,
        0x8 => Key::S,
        0x9 => Key::D,
        0xE => Key::F,
        0xA => Key::Y,
        0x0 => Key::X,
        0xB => Key::C,
        0xF => Key::V,
        _ => panic!("Wrong key"),
    }
}
