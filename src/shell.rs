use crate::{print, println, vga_buffer};
use alloc::string::{String, ToString};
use pc_keyboard::{DecodedKey, KeyCode};
use crate::drivers::keyboard::KEY_QUEUE;
use crate::fs::NodeType; 
use alloc::format;
use crate::serial_println;
use x86_64::VirtAddr;
use crate::USER_STACK;

fn print_prompt() {
    let auth = crate::auth::AUTH.lock();
    let fs = crate::fs::FS.lock();
    let username = auth.get_current_username();
    
    let path = if fs.cwd.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", fs.cwd.join("/"))
    };

    print!("{}@jc-os:{}$ ", username, path);
}

pub async fn run_shell() {
    vga_buffer::clear_screen();
    println!(" JC-OS - BARE METAL KERNEL v0.4 - RUST EDITION ");

    let mut command_buffer = String::with_capacity(256);

    loop {
        let is_logged_in = crate::auth::AUTH.lock().current_user.is_some();

        if !is_logged_in {
            let mut user = String::new();
            let mut pass = String::new();
            while let Some(_) = KEY_QUEUE.pop() {}

            println!("\n--- LOGIN REQUIRED ---");
            print!("Username: ");
            read_line(&mut user, false).await; // ASYNC READ
            println!(""); //
            
            print!("Password: ");
            read_line(&mut pass, true).await;  // ASYNC READ

            if crate::auth::AUTH.lock().login(user.trim(), pass.trim()) {
                println!("\nWelcome back, {}!", user.trim());
                while let Some(_) = KEY_QUEUE.pop() {}
                command_buffer.clear();
                print_prompt();
            } else {
                println!("\n[ERROR] Invalid credentials.");
            }
            continue; 
        }

        if let Some(key) = KEY_QUEUE.pop() {
            match key {
                DecodedKey::Unicode(ch) => {
                    match ch {
                        '\n' | '\r' => {
                            println!("");
                            interpret_command(&command_buffer).await; // AJOUT DU .AWAIT ICI
                            command_buffer.clear();
                            
                            if crate::auth::AUTH.lock().current_user.is_some() {
                                print_prompt();
                            }
                        }
                        '\u{8}' | '\u{7f}' => {
                            if !command_buffer.is_empty() {
                                command_buffer.pop();
                                vga_buffer::backspace();
                            }
                        }
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

pub async fn interpret_command(command: &str) {
    let command = command.trim();
    if command.is_empty() { return; }

    let mut parts = command.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("");

    let current_uid = crate::auth::AUTH.lock().get_current_uid();

    match cmd {
        "help" => {
            println!("Commands: help, info, whoami, clear, stats, neofetch, run, ia");
            println!("FS: look, open <dir>, room <name>, where, note <file> <text>, read <file>, drop <file>, type <file>");
        },

        "type" => {
            let file_name = args.trim();
            if file_name.is_empty() {
                println!("Usage: type <filename>");
            } else {
                let old_state = crate::vga_buffer::WRITER.lock().save_screen();
                crate::vga_buffer::clear_screen();
                
                {
                    let mut writer = crate::vga_buffer::WRITER.lock();
                    writer.set_color_code(crate::vga_buffer::ColorCode::new(
                        crate::vga_buffer::Color::DarkGray, 
                        crate::vga_buffer::Color::Black
                    ));
                    
                    let header = format!(" TYPE : {}  (Ctrl+S to save, Ctrl+Q to exit)", file_name);
                    writer.write_string_at(&header, 0, 0);
                    
                    writer.set_color_code(crate::vga_buffer::ColorCode::new(
                        crate::vga_buffer::Color::White, 
                        crate::vga_buffer::Color::Black
                    ));
                    writer.row_position = 2;
                    writer.column_position = 0;
                    writer.update_cursor();
                }

                let mut content = crate::fs::FS.lock().read_file(file_name).unwrap_or_else(|| String::new());
                print!("{}", content);

                loop {
                    if let Some(key) = crate::drivers::keyboard::KEY_QUEUE.pop() {
                        match key {
                            pc_keyboard::DecodedKey::Unicode('\u{0011}') => { // Ctrl+Q
                                crate::vga_buffer::WRITER.lock().restore_screen(&old_state);
                                break; 
                            }
                            pc_keyboard::DecodedKey::Unicode('\u{0013}') => { // Ctrl+S
                                let uid = crate::auth::AUTH.lock().get_current_uid();
                                let mut fs = crate::fs::FS.lock();
                                let _ = fs.write_file(file_name, &content, uid);
                            }
                            pc_keyboard::DecodedKey::Unicode('\u{08}') | pc_keyboard::DecodedKey::Unicode('\u{7f}') => {
                                if !content.is_empty() {
                                    content.pop();
                                    crate::vga_buffer::backspace();
                                }
                            }
                            pc_keyboard::DecodedKey::Unicode(c) => {
                                print!("{}", c);
                                content.push(c);
                            }
                            _ => {}
                        }
                    }
                    crate::task::yield_now().await;
                }
            }
        },

        "useradd" => {
            let is_admin = {
                let auth = crate::auth::AUTH.lock();
                auth.current_user.as_ref().map(|u| u.role == crate::auth::Role::Admin).unwrap_or(false)
            };

            if !is_admin {
                println!("[PERMISSION DENIED] Only administrators can add users.");
            } else {
                let mut arg_parts = args.splitn(2, ' ');
                let new_username = arg_parts.next().unwrap_or("");
                let new_password = arg_parts.next().unwrap_or("").trim();

                if new_username.is_empty() || new_password.is_empty() {
                    println!("Usage: useradd <username> <password>");
                } else {
                    let mut auth = crate::auth::AUTH.lock();
                    match auth.add_user(new_username, new_password) {
                        Ok(new_uid) => {
                            println!("[AUTH] User '{}' created with UID {}.", new_username, new_uid);
                            let mut fs = crate::fs::FS.lock();
                            let _ = fs.room("home", 0); 
                            let old_cwd = fs.cwd.clone();
                            if fs.open("/home").is_ok() {
                                let _ = fs.room(new_username, new_uid);
                            }
                            fs.cwd = old_cwd;
                        },
                        Err(e) => println!("[ERROR] {}", e),
                    }
                }
            }
        },

        "logout" => {
            crate::auth::AUTH.lock().logout();
            println!("Logged out.");
            return; 
        },

        "where" => {
            let fs = crate::fs::FS.lock();
            println!("/{}", fs.cwd.join("/"));
        },

        "look" => {
            let fs = crate::fs::FS.lock();
            let entries = fs.look();
            if entries.is_empty() {
                println!("Empty directory.");
            } else {
                for (name, node_type) in entries {
                    match node_type {
                        NodeType::Directory => println!("{}/", name),
                        NodeType::File => println!("{}", name),
                    }
                }
            }
        },

        "open" => {
            if args.is_empty() {
                println!("Usage: open <directory>");
            } else if let Err(e) = crate::fs::FS.lock().open(args) {
                println!("Error: {}", e);
            }
        },

        "room" => {
            if args.is_empty() {
                println!("Usage: room <name>");
            } else if let Err(e) = crate::fs::FS.lock().room(args, current_uid) {
                println!("Error: {}", e);
            }
        },

        "note" => {
            let mut arg_parts = args.splitn(2, ' ');
            let name = arg_parts.next().unwrap_or("");
            let content = arg_parts.next().unwrap_or("");
            if name.is_empty() {
                println!("Usage: note <filename> <content>");
            } else if let Err(e) = crate::fs::FS.lock().write_file(name, content, current_uid) {
                println!("Error: {}", e);
            } else {
                println!("File '{}' created.", name);
            }
        },

        "drop" => {
            let filename = args.trim();
            if filename.is_empty() {
                println!("Usage: drop <filename>");
            } else {
                let mut fs = crate::fs::FS.lock();
                if fs.remove_file(filename) {
                    println!("File '{}' removed.", filename);
                } else {
                    println!("Error: Could not find '{}'.", filename);
                }
            }
        },

        "read" => {
            let filename = args.trim();
            if let Some(content) = crate::fs::FS.lock().read_file(filename) {
                println!("{}", content);
            } else {
                println!("Error: File '{}' not found.", filename);
            }
        },
"run" => {
    let auth = crate::auth::AUTH.lock();
    let is_admin = auth.current_user.as_ref()
        .map(|u| u.role == crate::auth::Role::Admin)
        .unwrap_or(false);

    if !is_admin {
        println!("[PERMISSION DENIED] Admin only.");
    } else {
        // 1. Récupération de l'offset mémoire (sauvegardé lors de l'init_global)
        let phys_offset = *crate::memory::PHYS_MEM_OFFSET.lock();
        
        if let Some(offset) = phys_offset {
            println!("--- Preparing Ring 3 (Hierarchical Access) ---");

            // Localisation du code et de la structure de pile alignée
            let code_addr = VirtAddr::new(crate::user::user_test_program as *const () as u64);
            let stack_start = VirtAddr::from_ptr(unsafe { &raw const crate::USER_STACK.data });

            println!("[DEBUG] Code address: {:#x}", code_addr);
            println!("[DEBUG] Stack start:  {:#x}", stack_start);

            let mut all_pages_ok = true;
            unsafe {
                // 2. On déverrouille le chemin P4 -> P1 pour l'adresse du code
                // Déverrouiller 10 pages autour du code pour inclure .text et .rodata
                for i in 0..10 {
                    let page_addr = (code_addr.as_u64() / 4096) * 4096 + (i * 4096u64);
                    if !crate::memory::force_user_access(offset, VirtAddr::new(page_addr)) {
                        println!("[WARN] Could not unlock code page {}", i);
                        // Don't fail completely, some pages might be unmapped
                    }
                }
                
                // 3. On déverrouille toute la pile (16 KB = 4 pages)
                for i in 0..5 {
                    let page_addr = stack_start + (i * 4096u64);
                    if !crate::memory::force_user_access(offset, page_addr) {
                        println!("[ERROR] Could not unlock stack page {}", i);
                        all_pages_ok = false;
                        break;
                    }
                }
            }

            if all_pages_ok {
                println!("--- Jumping to Ring 3 ---");
                println!("[DEBUG] Function pointer: {:#x}", code_addr);
                println!("[DEBUG] Stack pointer: {:#x}", stack_start.as_u64() + 16384);
                
                // Calcul du sommet de la pile (rappel : la pile descend vers le bas)
                let stack_top = stack_start.as_u64() + 16384;
                
                crate::serial_println!("[SHELL] About to jump into user mode at {:#x}", code_addr);
                
                unsafe { 
                    // Appel de la fonction de saut avec les sélecteurs GDT
                    // User Code = 27 (0x1B), User Data = 35 (0x23)
                    crate::syscalls::jump_to_user(code_addr.as_u64(), stack_top); 
                }
            } else {
                println!("[ERROR] Failed to unlock all required stack pages");
            }
        } else {
            println!("[ERROR] Physical Memory Offset unknown! Check memory.rs.");
        }
    }
},
        "whoami" => println!("{}", crate::auth::AUTH.lock().get_current_username()),
        "clear" => vga_buffer::clear_screen(),
        "stats" => {
            let (file_count, total_bytes) = crate::fs::FS.lock().get_stats();
            println!("Items: {} | Usage: {} bytes", file_count, total_bytes);
        },

        "ia" => {
            if args.is_empty() {
                println!("Usage: ia <question>");
            } else {
                print!("Interrogating JC-AI...");
                serial_println!("AI_REQ:{}", args); 
                let response = crate::serial::read_line(); // Note: Devrait idéalement être async aussi
                println!("\r"); 
                if response.contains("[[CLEAR]]") {
                    vga_buffer::clear_screen(); 
                    println!("[JC-AI]: Ecran nettoye, Andre !");
                } else {
                    println!("[JC-AI]: {}", response);
                }
            }
        },

        "neofetch" => {
            let time = crate::drivers::rtc::get_time();
            println!("   _/_/    JC-OS v0.4 - Rust Edition");
            println!("  _/       User : {}", crate::auth::AUTH.lock().get_current_username());
            println!(" _/_/_/    CPU  : x86_64 Bare Metal");
            println!("           Time : {:02}:{:02}:{:02}", time.hours, time.minutes, time.seconds);
        },

        _ => println!("Unknown command: {}", cmd),
    }
}

// FONCTION INDISPENSABLE POUR LE LOGIN ASYNC
async fn read_line(target: &mut String, mask: bool) {
    loop {
        if let Some(key) = KEY_QUEUE.pop() {
            match key {
                DecodedKey::Unicode(ch) => {
                    match ch {
                        '\n' | '\r' => break,
                        '\u{8}' | '\u{7f}' => {
                            if !target.is_empty() {
                                target.pop();
                                vga_buffer::backspace();
                            }
                        }
                        c if c >= ' ' => {
                            target.push(c);
                            if mask { print!("*"); } else { print!("{}", c); }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        crate::task::yield_now().await;
    }
}