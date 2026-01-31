use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet2, KeyCode, KeyState};
use spin::Mutex;
use lazy_static::lazy_static;
use crate::{vga_buffer, serial_println, serial_print};

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Azerty, ScancodeSet2>> =
        Mutex::new(Keyboard::new(
            ScancodeSet2::new(),
            layouts::Azerty,
            HandleControl::Ignore
        ));
}

pub fn init() {
    serial_println!("[KEYBOARD] Driver initialized (AZERTY layout, Set2)");
}

pub fn add_scancode(scancode: u8) {
    match scancode {
        0xFA | 0xFE | 0xAA | 0x00 => return, 
        _ => {} 
    }
    
    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        //  On stocke l'état AVANT que process_keyevent ne consomme le key_event ---
        let state = key_event.state;
        let code = key_event.code;

        // Gestion prioritaire des touches spéciales
        if state == KeyState::Down {
            match code {
                KeyCode::Backspace => {
                    vga_buffer::backspace();
                    return;
                }
                KeyCode::Escape => {
                    vga_buffer::clear_screen();
                    return;
                }
                _ => {}
            }
        }

        // On passe key_event ici (il est "moved")
        if let Some(decoded) = keyboard.process_keyevent(key_event) {
            // On utilise la variable 'state' qu'on a sauvegardée plus haut
            if state == KeyState::Down {
                match decoded {
                    DecodedKey::Unicode(ch) => {
                        if ch != '\u{0008}' {
                            vga_buffer::print_char(ch);
                            serial_print!("{}", ch);
                        }
                    }
                    DecodedKey::RawKey(key) => {
                        match key {
                            KeyCode::Return | KeyCode::NumpadEnter => {
                                vga_buffer::print_char('\n');
                                serial_print!("\n");
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}