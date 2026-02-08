use crate::{print, println, vga_buffer};
use alloc::string::{String, ToString}; // On garde les deux finalement !
use pc_keyboard::{DecodedKey, KeyCode};
use crate::drivers::keyboard::KEY_QUEUE;
use crate::fs::NodeType; 
use alloc::format;

fn print_prompt() {
    let auth = crate::auth::AUTH.lock();
    let fs = crate::fs::FS.lock();
    
    let username = auth.get_current_username();
    
    // Construction du chemin CWD (Current Working Directory)
    let path = if fs.cwd.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", fs.cwd.join("/"))
    };

    // Prompt style Linux : andre@jc-os:/home$
    print!("{}@jc-os:{}$ ", username, path);
}

pub async fn run_shell() {
    vga_buffer::clear_screen();
    println!(" JC-OS - BARE METAL KERNEL v0.4 - RUST EDITION ");

    let mut command_buffer = String::with_capacity(256);

    loop { // BOUCLE PRINCIPALE
        // 1. Vérification : est-on connecté ?
        let is_logged_in = crate::auth::AUTH.lock().current_user.is_some();

        if !is_logged_in {
            // --- PHASE DE LOGIN ---
            let mut user = String::new();
            let mut pass = String::new();

            println!("\n--- LOGIN REQUIRED ---");
            print!("Username: ");
            read_line(&mut user, false).await;
            
            print!("Password: ");
            read_line(&mut pass, true).await; 

            if crate::auth::AUTH.lock().login(user.trim(), pass.trim()) {
                println!("\nWelcome back, {}!", user.trim());
                command_buffer.clear();
                print_prompt();
            } else {
                println!("\n[ERROR] Invalid credentials.");
            }
            // On retourne au début du loop pour vérifier à nouveau is_logged_in
            continue; 
        }

        // 2. PHASE DE COMMANDES (Si connecté)
        if let Some(key) = KEY_QUEUE.pop() {
            match key {
                DecodedKey::Unicode(ch) => {
                    match ch {
                        '\n' | '\r' => {
                            println!("");
                            interpret_command(&command_buffer);
                            command_buffer.clear();
                            
                            // Si la commande était "logout", on ne print pas de prompt
                            if crate::auth::AUTH.lock().current_user.is_some() {
                                print_prompt();
                            }
                        }
                        // Support du Backspace en mode Unicode (0x08 ou 0x7F)
                        '\u{8}' | '\u{7f}' => {
                            if !command_buffer.is_empty() {
                                command_buffer.pop();
                                vga_buffer::backspace();
                            }
                        }
                        // Accepte tous les caractères imprimables
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

async fn read_line(buffer: &mut String, mask: bool)  {
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

    // On récupère l'UID actuel pour les créations de fichiers/dossiers
   let current_uid = crate::auth::AUTH.lock().get_current_uid();// À remplacer par auth.get_current_uid() plus tard

    match cmd {
        "help" => {
            println!("Commands: help, info, whoami, clear, stats, neofetch");
            println!("FS: look, open <dir>, room <name>, where, note <file> <text>, read <file>, drop <file>");
        },

       "useradd" => {
    // --- VÉRIFICATION DE SÉCURITÉ ---
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
            // 1. Ajouter l'utilisateur dans le système d'authentification
            let mut auth = crate::auth::AUTH.lock();
            match auth.add_user(new_username, new_password) {
                Ok(new_uid) => {
                    println!("[AUTH] User '{}' created with UID {}.", new_username, new_uid);
                    
                    // 2. Créer automatiquement son dossier Home
                    let mut fs = crate::fs::FS.lock();
                    
                    // On s'assure que /home existe
                    let _ = fs.room("home", 0); 
                    
                    let old_cwd = fs.cwd.clone();
                    if fs.open("/home").is_ok() {
                        if let Err(e) = fs.room(new_username, new_uid) {
                            println!("[FS ERROR] Could not create home directory: {}", e);
                        } else {
                            println!("[FS] Home directory /home/{} created.", new_username);
                        }
                    }
                    fs.cwd = old_cwd;
                },
                Err(e) => println!("[ERROR] {}", e),
            }
        }
    }
},

       "userdel" => {
    // --- VÉRIFICATION DE SÉCURITÉ ---
    let is_admin = {
        let auth = crate::auth::AUTH.lock();
        auth.current_user.as_ref().map(|u| u.role == crate::auth::Role::Admin).unwrap_or(false)
    };

    if !is_admin {
        println!("[PERMISSION DENIED] Only administrators can delete users.");
    } else {
        let username_to_del = args.trim();
        if username_to_del.is_empty() {
            println!("Usage: userdel <username>");
        } else {
            // 1. Supprimer du système d'authentification
            let mut auth = crate::auth::AUTH.lock();
            match auth.delete_user(username_to_del) {
                Ok(_) => {
                    println!("[AUTH] User '{}' deleted.", username_to_del);
                    
                    // 2. Supprimer son home directory
                    let mut fs = crate::fs::FS.lock();
                    let old_cwd = fs.cwd.clone();
                    if fs.open("/home").is_ok() {
                        if fs.remove_file(username_to_del) {
                            println!("[FS] Home directory /home/{} removed.", username_to_del);
                        }
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
            // Note: Le loop principal du shell va nous redemander le login au prochain tour
            return; 
        },

        "edit" => {
        let mut arg_parts = args.splitn(2, ' ');
        let file_name = arg_parts.next().unwrap_or("");
        let new_content = arg_parts.next().unwrap_or("");

    if file_name.is_empty() {
        println!("Usage: edit <filename> <text>");
    } else {
        let current_uid = crate::auth::AUTH.lock().get_current_uid();
        // On réutilise write_file qui écrase le contenu existant
        let mut fs = crate::fs::FS.lock();
        match fs.write_file(file_name, new_content, current_uid) {
            Ok(_) => println!("File '{}' updated.", file_name),
            Err(e) => println!("[ERROR] Could not edit file: {}", e),
        }
    }
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
                    // Utilisation directe du type importé
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
            } else {
                if let Err(e) = crate::fs::FS.lock().open(args) {
                    println!("Error: {}", e);
                }
            }
        },

        "room" => {
            if args.is_empty() {
                println!("Usage: room <name>");
            } else {
                // On passe bien 2 arguments : le nom et l'UID
                if let Err(e) = crate::fs::FS.lock().room(args, current_uid) {
                    println!("Error: {}", e);
                }
            }
        },

       "note" => {
            let mut arg_parts = args.splitn(2, ' ');
            let name = arg_parts.next().unwrap_or("");
            let content = arg_parts.next().unwrap_or("");
            if name.is_empty() {
                println!("Usage: note <filename> <content>");
            } else {
                // On utilise le Result et on passe l'UID
                if let Err(e) = crate::fs::FS.lock().write_file(name, content, current_uid) {
                    println!("Error: {}", e);
                } else {
                    println!("File '{}' created.", name);
                }
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
                    println!("Error: Could not find or remove '{}'.", filename);
                }
            }
        },
        "read" => {
            let filename = args.trim();
            // Attention : read_file dans le FS pro doit être mis à jour pour chercher dans le CWD
            // Pour l'instant, on utilise la logique simplifiée
            if let Some(content) = crate::fs::FS.lock().read_file(filename) {
                println!("{}", content);
            } else {
                println!("Error: File '{}' not found.", filename);
            }
        },

        "whoami" => {
            println!("{}", crate::auth::AUTH.lock().get_current_username());
        },

        "clear" => vga_buffer::clear_screen(),

        "stats" => {
            let (file_count, total_bytes) = crate::fs::FS.lock().get_stats();
            println!("Files/Folders : {}", file_count);
            println!("Used Space    : {} bytes", total_bytes);
        },

        "neofetch" => {
            println!("   _/_/    JC-OS v0.4 Pro");
            println!("  _/       User: {}", crate::auth::AUTH.lock().get_current_username());
            println!(" _/_/_/    FS  : Hierarchical RAMFS");
        },

        _ => println!("Unknown command: {}", cmd),
    }
}