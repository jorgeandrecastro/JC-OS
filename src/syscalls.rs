use x86_64::registers::model_specific::{LStar, SFMask, Efer, EferFlags, GsBase, Msr};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;
use core::arch::global_asm;

// Assembleur : Gestion de l'entrée et de la sortie du Ring 3
global_asm!(r#"
.global syscall_entry
.extern syscall_handler

syscall_entry:
    swapgs                  # Bascule GS du mode User vers le mode Kernel
    mov gs:[0x08], rsp      # Sauvegarde le RSP (pile) de l'utilisateur
    mov rsp, gs:[0x00]      # Charge le RSP (pile) du noyau

    # Sauvegarde des registres volatils pour le retour
    push r11                # Sauvegarde RFLAGS
    push rcx                # Sauvegarde RIP de retour (sysret utilise RCX)

    # Rearrange arguments for System V ABI calling convention
    # User passes: rax=ID, rdi=arg1, rsi=arg2
    # We need:    rdi=ID, rsi=arg1, rdx=arg2
    mov rdx, rsi            # rdx = arg2 (was in rsi)
    mov rsi, rdi            # rsi = arg1 (was in rdi)
    mov rdi, rax            # rdi = syscall ID (was in rax)

    # Call the handler with proper arguments
    call syscall_handler

    # Restauration du contexte
    pop rcx                 # Restaure le RIP de retour
    pop r11                 # Restaure les RFLAGS
    
    mov rsp, gs:[0x08]      # Restaure la pile de l'utilisateur
    swapgs                  # Remet le GS utilisateur en place
    sysretq                 # Retour vers le Ring 3
"#);

#[repr(C)]
struct KernelData {
    kernel_stack: u64, // Offset 0x00
    user_stack: u64,   // Offset 0x08
}

// Données statiques pour stocker les piles par CPU
static mut KERNEL_DATA: KernelData = KernelData { kernel_stack: 0, user_stack: 0 };

/// Initialise les mécanismes de Syscall du processeur
pub fn init() {
    let handler_addr = VirtAddr::new(syscall_entry as *const () as u64);
    unsafe {
        // 1. Activer les extensions syscall (EFER)
        Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);

        // 2. Définir l'adresse de l'entrée (LSTAR)
        LStar::write(handler_addr);

        // 3. Configurer les segments (STAR)
        // Format : [User Base Selector + 16] [Kernel Base Selector] [NULL] [NULL]
        let k_code = crate::gdt::get_kernel_code_selector().0 as u64;
        let u_code = (crate::gdt::get_user_code_selector().0 as u64 - 16) | 3;
        
        let mut star_msr = Msr::new(0xC000_0081);
        let val = (u_code << 48) | (k_code << 32);
        star_msr.write(val);

        // 4. Masque de drapeaux (SFMask) : On coupe les interruptions lors du syscall
        SFMask::write(RFlags::INTERRUPT_FLAG);

        // 5. Initialiser GS_BASE pour pointer sur notre structure de données
        init_gs_base();
        
        // 6. Charger la pile noyau par défaut
        set_kernel_stack(crate::gdt::get_tss_stack_ptr().as_u64());
    }
}

pub fn init_gs_base() {
    unsafe {
        let gs_base_addr = &raw const KERNEL_DATA as u64;
        
        // Set both GS_BASE (user mode) and IA32_KERNEL_GS_BASE (kernel mode)
        GsBase::write(VirtAddr::new(gs_base_addr));
        
        // Set IA32_KERNEL_GS_BASE MSR (0xC000_0102)
        let mut kernel_gs_msr = Msr::new(0xC000_0102);
        kernel_gs_msr.write(gs_base_addr);
        
        crate::serial_println!("[SYSCALL] GS_BASE initialized: {:#x}", gs_base_addr);
    }
}

/// Make KERNEL_DATA (GS_BASE) accessible from user mode. 
/// MUST be called after memory::init_global()
pub fn make_gs_accessible_from_user() {
    unsafe {
        let gs_base_addr = VirtAddr::new(&raw const KERNEL_DATA as u64);
        crate::serial_println!("[SYSCALL] GS_BASE address: {:#x}", gs_base_addr);
        crate::serial_println!("[SYSCALL] KERNEL_DATA structure size: {} bytes", core::mem::size_of::<KernelData>());
        
        if let Some(phys_offset) = *crate::memory::PHYS_MEM_OFFSET.lock() {
            crate::serial_println!("[SYSCALL] Attempting to unlock GS_BASE page...");
            let success = crate::memory::force_user_access(phys_offset, gs_base_addr);
            if success {
                crate::serial_println!("[SYSCALL] ✓ GS_BASE page unlocked for user mode");
            } else {
                crate::serial_println!("[SYSCALL] ✗ Failed to unlock GS_BASE page!");
            }
        } else {
            crate::serial_println!("[SYSCALL] ✗ PHYS_MEM_OFFSET not available!");
        }
    }
}

pub fn set_kernel_stack(stack_addr: u64) {
    unsafe {
        KERNEL_DATA.kernel_stack = stack_addr;
    }
}

#[no_mangle]
pub extern "C" fn syscall_handler(syscall_id: u64, arg1: u64, arg2: u64) {
    crate::serial_println!("[SYSCALL] Handler: ID={} (0x{:x}), ARG1={:#x}, ARG2={}", syscall_id, syscall_id, arg1, arg2);
    
    match syscall_id {
        // --- SYS_EXIT ---
        0 => { 
            crate::serial_println!("[SYSCALL] SYS_EXIT (code={})", arg1);
            crate::serial_println!("[SYSTEM] User program exited with code {}", arg1);
            loop { x86_64::instructions::hlt(); }
        },

        // --- SYS_PRINT (VGA + SERIAL) ---
        1 => { 
            let ptr = arg1 as *const u8;
            let len = arg2 as usize;
            
            if len > 0 && len <= 1024 {
                let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
                if let Ok(s) = core::str::from_utf8(slice) {
                    // 1. Debug sur le port série
                    crate::serial_println!("[RING 3] {}", s);
                    
                    // 2. Affichage physique sur l'écran (QEMU)
                    use crate::vga_buffer::WRITER;
                    let mut writer = WRITER.lock();
                    writer.write_string(s);
                    writer.write_string("\n"); 
                } else {
                    crate::serial_println!("[SYSCALL] Invalid UTF-8 string");
                }
            } else {
                crate::serial_println!("[SYSCALL] Invalid string length: {}", len);
            }
        },

        _ => crate::serial_println!("[WARN] Unknown syscall ID: {} (0x{:x})", syscall_id, syscall_id),
    }
}

/// Saute littéralement dans l'espace utilisateur
pub unsafe fn jump_to_user(code_ptr: u64, stack_ptr: u64) -> ! {
    let code_sel = (crate::gdt::get_user_code_selector().0 | 0x3) as u64;
    let data_sel = (crate::gdt::get_user_data_selector().0 | 0x3) as u64;

    core::arch::asm!(
        ".intel_syntax noprefix",
        "cli",              // Disable interrupts during switch
        "push {data_sel}",  // SS
        "push {stack_ptr}", // RSP
        "pushf",            // RFLAGS
        "pop rax",          // Get RFLAGS into RAX
        "or rax, 0x200",    // Set IF bit (interrupts enabled)
        "push rax",         // Push modified RFLAGS
        "push {code_sel}",  // CS
        "push {code_ptr}",  // RIP
        "iretq",            // Pop all and jump to user mode
        ".att_syntax",
        data_sel = in(reg) data_sel,
        stack_ptr = in(reg) stack_ptr,
        code_sel = in(reg) code_sel,
        code_ptr = in(reg) code_ptr,
        options(noreturn)
    );
}

extern "C" {
    fn syscall_entry();
}