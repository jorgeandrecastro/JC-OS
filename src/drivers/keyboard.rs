use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet2, KeyCode, KeyState};
use spin::Mutex;
use lazy_static::lazy_static;
use crate::{vga_buffer, serial_println, serial_print, print, println};
use alloc::string::String;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Azerty, ScancodeSet2>> =
        Mutex::new(Keyboard::new(
            ScancodeSet2::new(),
            layouts::Azerty,
            HandleControl::Ignore
        ));

    // Dynamic buffer to store user commands
    static ref COMMAND_BUFFER: Mutex<String> = Mutex::new(String::with_capacity(80));
}

pub fn init() {
    serial_println!("[KEYBOARD] Driver initialized (AZERTY layout, Set2)");
}

/// Parse and execute commands typed by Andre
fn interpret_command(command: &str) {
    match command {
        "aide" | "help" => {
            println!("\nAvailable commands: help, info, stats, clear, neofetch");
        }
        "info" => {
            println!("\nJC-OS v0.2 - Sovereign Rust Kernel");
            println!("Architecture: x86_64 Bare Metal");
            println!("Mode: Memory Protection & Dynamic Allocation");
        }
        "stats" => {
            println!("\n--- JC-OS MEMORY STATS ---");
            println!("Heap Start : 0x{:X}", crate::allocator::heap_start());
            println!("Heap Size  : {} KB", crate::allocator::heap_size() / 1024);
            println!("Status     : DYNAMIC ALLOCATION OK");
            println!("---------------------------");
        }
        "neofetch" => {
            println!("\n    _/_/_/    _/_/_/  ");
            println!("       _/  _/         JC-OS v0.2");
            println!("      _/  _/          Kernel: Rust 64-bit");
            println!("_/_/_/      _/_/_/    Sovereignty: 100%");
            println!("\nSystem ready for critical computations.");
        }
        "clear" => {
            vga_buffer::clear_screen();
            print!(">>> "); // Immediately show the prompt again
            return;
        }
        _ => {
            if !command.is_empty() {
                println!("\nUnknown command: {}", command);
            }
        }
    }
    print!("\n>>> ");
}

pub fn add_scancode(scancode: u8) {
    match scancode {
        0xFA | 0xFE | 0xAA | 0x00 => return, // Ignore ACK, NACK, Self-test, and 0
        _ => {} 
    }
    
    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        let state = key_event.state;
        let code = key_event.code;

        // Handle control keys (Down only)
        if state == KeyState::Down {
            match code {
                KeyCode::Backspace => {
                    let mut cmd = COMMAND_BUFFER.lock();
                    if !cmd.is_empty() {
                        cmd.pop();
                        vga_buffer::backspace();
                    }
                    return;
                }
                KeyCode::Escape => {
                    COMMAND_BUFFER.lock().clear();
                    vga_buffer::clear_screen();
                    print!(">>> ");
                    return;
                }
                _ => {}
            }
        }

        // Decode Unicode characters
        if let Some(decoded) = keyboard.process_keyevent(key_event) {
            if state == KeyState::Down {
                match decoded {
                    DecodedKey::Unicode(ch) => {
                        if ch == '\n' || ch == '\r' {
                            let mut cmd = COMMAND_BUFFER.lock();
                            interpret_command(&cmd);
                            cmd.clear();
                        } else if ch != '\u{0008}' { // Avoid pushing backspace
                            COMMAND_BUFFER.lock().push(ch);
                            vga_buffer::print_char(ch);
                            serial_print!("{}", ch);
                        }
                    }
                    DecodedKey::RawKey(key) => {
                        if key == KeyCode::Return || key == KeyCode::NumpadEnter {
                            let mut cmd = COMMAND_BUFFER.lock();
                            interpret_command(&cmd);
                            cmd.clear();
                        }
                    }
                }
            }
        }
    }
}
