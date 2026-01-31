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

    static ref COMMAND_BUFFER: Mutex<String> = Mutex::new(String::with_capacity(80));
}

pub fn init() {
    serial_println!("[KEYBOARD] Driver initialized (AZERTY layout, Set2)");
}

fn interpret_command(command: &str) {
    let command = command.trim();
    if command.is_empty() { 
        print!("\n>>> ");
        return; 
    }

    println!(""); 

    let mut parts = command.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("");

    match cmd {
        "help" => {
            println!("Commands: help, info, stats, echo, whoami, ls, touch, cat, clear, neofetch");
        },
        "info" => {
            println!("JC-OS v0.2 - Andre Edition");
            println!("Status: Stable");
        },
        "whoami" => {
            println!("Andre");
        },
        "echo" => {
            println!("{}", args);
        },
        // --- NOUVELLES COMMANDES DE SYSTÈME DE FICHIERS ---
        "ls" => {
            let fs = crate::fs::FS.lock();
            let files = fs.list_files();
            if files.is_empty() {
                println!("No files found.");
            } else {
                for f in files {
                    println!("- {}", f);
                }
            }
        },
        "touch" => {
            let mut arg_parts = args.splitn(2, ' ');
            let name = arg_parts.next().unwrap_or("");
            let content = arg_parts.next().unwrap_or("");
            if name.is_empty() {
                println!("Usage: touch <filename> <content>");
            } else {
                crate::fs::FS.lock().write_file(name, content);
                println!("File '{}' saved to RAM.", name);
            }
        },
        "cat" => {
            if let Some(content) = crate::fs::FS.lock().read_file(args.trim()) {
                println!("{}", content);
            } else {
                println!("Error: File '{}' not found.", args.trim());
            }
        },
        // ------------------------------------------------
        "stats" => {
            println!("--- MEMORY STATS ---");
            println!("Heap Start : 0x444444440000");
            println!("Heap Size  : 100 KB");
        },
        "neofetch" => {
            println!("  _/_/   JC-OS v0.2");
            println!(" _/      Kernel: Rust 64-bit");
            println!("_/_/_/   User: Andre");
        },
        "clear" => {
            crate::vga_buffer::clear_screen();
        },
        _ => {
            println!("Unknown command: {}", cmd);
        },
    }
    
    print!("\n>>> ");
}

pub fn add_scancode(scancode: u8) {
    match scancode {
        0xFA | 0xFE | 0xAA | 0x00 => return,
        _ => {} 
    }
    
    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        let state = key_event.state;
        let code = key_event.code;

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

        if let Some(decoded) = keyboard.process_keyevent(key_event) {
            if state == KeyState::Down {
                match decoded {
                    DecodedKey::Unicode(ch) => {
                        if ch == '\n' || ch == '\r' {
                            let mut cmd = COMMAND_BUFFER.lock();
                            interpret_command(&cmd);
                            cmd.clear();
                        } else {
                            COMMAND_BUFFER.lock().push(ch);
                            vga_buffer::print_char(ch);
                            serial_print!("{}", ch);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}