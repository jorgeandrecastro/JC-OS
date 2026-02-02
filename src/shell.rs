use crate::{print, println, vga_buffer};
use alloc::string::String;
use pc_keyboard::{DecodedKey, KeyCode};
use crate::drivers::keyboard::KEY_QUEUE; 

fn print_prompt() {
    let auth = crate::auth::AUTH.lock();
    let username = auth.get_current_username();
    print!("{}@jc-os:~$ ", username);
}

pub async fn run_shell() {
    vga_buffer::clear_screen();
    println!(" JC-OS - BARE METAL KERNEL v0.3 - RUST EDITION ");

    loop {
        let mut user = String::new();
        let mut pass = String::new();

        println!("\n--- LOGIN REQUIRED ---");
        print!("Username: ");
        read_line(&mut user, false).await;
        
        print!("Password: ");
        read_line(&mut pass, true).await; 

        // .trim() est important pour enlever les résidus de \n ou \r
        let success = crate::auth::AUTH.lock().login(user.trim(), pass.trim());

        if success {
            println!("\nWelcome back, {}!", user.trim());
            break; 
        } else {
            println!("\n[ERROR] Invalid credentials.");
        }
    }

    let mut command_buffer = String::with_capacity(256);
    print_prompt();

    loop {
        if let Some(key) = KEY_QUEUE.pop() {
            match key {
                DecodedKey::Unicode(ch) => {
                    match ch {
                        '\n' | '\r' => {
                            println!("");
                            interpret_command(&command_buffer);
                            command_buffer.clear();
                            print_prompt();
                        }
                        // Support du Backspace en mode Unicode (0x08 ou 0x7F)
                        '\u{8}' | '\u{7f}' => {
                            if !command_buffer.is_empty() {
                                command_buffer.pop();
                                vga_buffer::backspace();
                            }
                        }
                        // Accepte tous les caractères imprimables (AZERTY/Shift ok)
                        c if c >= ' ' => {
                            command_buffer.push(c);
                            print!("{}", c);
                        }
                        _ => {}
                    }
                }
                DecodedKey::RawKey(code) => {
                    match code {
                        KeyCode::Backspace => {
                            if !command_buffer.is_empty() {
                                command_buffer.pop();
                                vga_buffer::backspace();
                            }
                        }
                        KeyCode::Escape => {
                            command_buffer.clear();
                            vga_buffer::clear_screen();
                            print_prompt();
                        }
                        _ => {} 
                    }
                }
            }
        }
        crate::task::yield_now().await;
    }
}

async fn read_line(buffer: &mut String, mask: bool) {
    buffer.clear();
    loop {
        if let Some(key) = KEY_QUEUE.pop() {
            match key {
                DecodedKey::Unicode(ch) => {
                    match ch {
                        '\n' | '\r' => {
                            println!("");
                            return;
                        }
                        '\u{8}' | '\u{7f}' => {
                            if !buffer.is_empty() {
                                buffer.pop();
                                vga_buffer::backspace();
                            }
                        }
                        c if c >= ' ' => {
                            buffer.push(c);
                            if mask { print!("*"); } else { print!("{}", c); }
                        }
                        _ => {}
                    }
                }
                DecodedKey::RawKey(code) => {
                    if code == KeyCode::Backspace {
                        if !buffer.is_empty() {
                            buffer.pop();
                            vga_buffer::backspace();
                        }
                    }
                }
            }
        }
        crate::task::yield_now().await;
    }
}

pub fn interpret_command(command: &str) {
    let command = command.trim();
    if command.is_empty() { return; }

    let mut parts = command.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("");

    match cmd {
        "help" => {
            println!("Available commands:");
            println!("  help, info, stats, whoami, clear, neofetch");
            println!("  ls, touch <file> <text>, cat <file>, edit <file> <text>, rm <file>");
        },
        "info" => {
            println!("JC-OS v0.3 - Andre Edition");
            println!("Architecture: x86_64 Bare Metal");
            println!("Kernel Status: Stable (Multitasking Enabled)");
        },
        "whoami" => {
            let auth = crate::auth::AUTH.lock();
            println!("{}", auth.get_current_username());
        },
        "clear" => {
            vga_buffer::clear_screen();
        },
        "ls" => {
            let fs = crate::fs::FS.lock();
            let files = fs.list_files();
            if files.is_empty() {
                println!("No files found in RAM.");
            } else {
                for f in files { println!("- {}", f); }
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
                println!("File '{}' created.", name);
            }
        },
        "cat" => {
            let filename = args.trim();
            if let Some(content) = crate::fs::FS.lock().read_file(filename) {
                println!("{}", content);
            } else {
                println!("Error: File '{}' not found.", filename);
            }
        },
        "edit" => {
            let mut arg_parts = args.splitn(2, ' ');
            let name = arg_parts.next().unwrap_or("");
            let new_content = arg_parts.next().unwrap_or("");
            if name.is_empty() {
                println!("Usage: edit <filename> <new_content>");
            } else {
                let mut fs = crate::fs::FS.lock();
                if fs.read_file(name).is_some() {
                    fs.write_file(name, new_content);
                    println!("File '{}' updated.", name);
                } else {
                    println!("Error: File '{}' does not exist.", name);
                }
            }
        },
        "rm" => {
            let filename = args.trim();
            if crate::fs::FS.lock().remove_file(filename) {
                println!("File '{}' deleted.", filename);
            } else {
                println!("Error: File not found.");
            }
        },
        "stats" => {
            let (file_count, total_bytes) = crate::fs::FS.lock().get_stats();
            println!("--- SYSTEM STATS ---");
            println!("Stored Files : {}", file_count);
            println!("Used FS Mem  : {} bytes", total_bytes);
            println!("Active Tasks : 3 (Shell, Clock, Keyboard)");
        },
        "neofetch" => {
            println!("   _/_/    JC-OS v0.3");
            println!("  _/       OS: Rust Bare Metal");
            let user = crate::auth::AUTH.lock().get_current_username();
            println!(" _/_/_/    User: {}", user);
            println!("           Mode: Ring 0 (Pre-User)");
        },
        _ => {
            println!("Unknown command: {}", cmd);
        },
    }
}