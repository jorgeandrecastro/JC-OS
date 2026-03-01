//! Module user : Code destiné à s'exécuter en Ring 3 dans JC-OS.
//! Signé : The Rust Eagle 🦅

/// Affiche une chaîne de caractères via le Syscall ID 1
/// Utilise la convention d'appel System V pour les syscalls x86_64.
pub fn sys_print(s: &str) {
    let ptr = s.as_ptr() as u64;
    let len = s.len() as u64;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 1,      // ID du syscall (SYS_PRINT)
            in("rdi") ptr,    // Argument 1 : Pointeur vers le buffer
            in("rsi") len,    // Argument 2 : Longueur de la chaîne
            // Le CPU écrase RCX (RIP) et R11 (RFLAGS) lors du syscall
            out("rcx") _,     
            out("r11") _,     
            // On informe Rust que ces registres peuvent changer
            clobber_abi("system"), 
        );
    }
}

/// Termine l'exécution du programme via le Syscall ID 0
/// Cette fonction ne retourne JAMAIS au programme appelant.
pub fn sys_exit(code: u64) -> ! {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 0,      // ID du syscall (SYS_EXIT)
            in("rdi") code,   // Argument 1 : Code de sortie (0 = OK)
            options(noreturn) // Indique au compilateur que l'asm ne revient pas
        );
    }
}

/// --- PROGRAMME DE TEST JC-OS ---
/// Ce point d'entrée est appelé par le noyau après le passage en Ring 3.
#[no_mangle]
pub extern "C" fn user_test_program() -> ! {
    // Inline asm to confirm we're executing in user mode
    unsafe {
        core::arch::asm!("mov rax, 1");  // Just a No-op to mark execution point
    }
    
    // Test 1: Call sys_print
    sys_print("Hello from Ring 3!");
    sys_print("JC-OS user mode is working.");
    
    // Test 2: Exit gracefully
    sys_exit(0);
}