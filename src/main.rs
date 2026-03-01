#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)] 

extern crate alloc;
use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use crate::executor::Executor;
use crate::task::Task;

// --- DÉCLARATION DES MODULES ---
mod vga_buffer;
mod serial;
mod interrupts;
mod gdt;
mod drivers;
mod memory;
mod allocator;
mod fs; 
mod shell;
mod auth;
pub mod task;
pub mod executor;
mod syscalls; // Nouveau : Gestion des interruptions logicielles
mod user;     // Nouveau : Espace utilisateur (Ring 3)

// --- CONFIGURATION DE LA PILE UTILISATEUR ---
entry_point!(kernel_main);

// On utilise une struct pour forcer l'alignement sur une page (4096 bytes)

#[repr(C, align(4096))]
pub struct UserStack {
    pub data: [u8; 16384],
}
#[no_mangle]
pub static mut USER_STACK: UserStack = UserStack { data: [0; 16384] };

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 1. Setup Architecture (GDT, IDT, Syscalls)
    gdt::init();
    interrupts::init_idt();
   
    syscalls::init_gs_base();
    interrupts::init_pic();
    unsafe { crate::syscalls::init(); }

   // 2. Memory & Heap
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    
    // Initialisation du mapper global (dans memory.rs)
    unsafe { memory::init_global(phys_mem_offset) };

    // On récupère le verrou pour initialiser le Heap
    {
        let mut mapper_lock = memory::MAPPER.lock();
        let mapper = mapper_lock.as_mut().expect("Mapper global non initialisé");
        let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
        
        allocator::init_heap(mapper, &mut frame_allocator)
            .expect("Heap Initialization Failed");
    }

    // Make GS_BASE (KERNEL_DATA) accessible from user mode for syscalls
    syscalls::make_gs_accessible_from_user();

    // 3. Setup pile Kernel
    let tss_stack = gdt::get_tss_stack_ptr().as_u64();
    syscalls::set_kernel_stack(tss_stack);

    // 4. Drivers
    drivers::keyboard::init();
    x86_64::instructions::interrupts::enable();

    display_screen();

    // --- L'EXECUTOR ---
    let mut executor = Executor::new();

    // Tâche 1 : L'horloge (Ring 0) - Elle tournera toujours en fond
    executor.spawn(Task::new(clock_task())); 

    // Tâche 2 : Le Shell (Ring 0)
    // C'est lui qui affichera le prompt et te permettra de taper "run"
    executor.spawn(Task::new(crate::shell::run_shell())); 

    serial_println!("[SYSTEM] JC-OS Ready. Executor running.");
    
    // 5. Lancement
    executor.run();
}
// --- GESTION DES ERREURS & PANIC ---

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Alloc Error: {:?}", layout)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("\n[PANIC] {}", info);
    loop { x86_64::instructions::hlt(); }
}

// --- FONCTIONS AUXILIAIRES ---

fn init_keyboard_controller() {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut cmd: Port<u8> = Port::new(0x64);
        let mut data: Port<u8> = Port::new(0x60);
        while (cmd.read() & 0x01) != 0 { let _ = data.read(); }
        cmd.write(0xAD); cmd.write(0xA7);
        cmd.write(0x20);
        let mut config = data.read();
        config |= 0x01; config &= !0x42;
        cmd.write(0x60); data.write(config);
        cmd.write(0xAE);
    }
}

fn display_screen() {
    use vga_buffer::{WRITER, ColorCode, Color};
    let mut writer = WRITER.lock();
    writer.clear_screen();
    
    // On garde ça ultra simple et pro
    writer.set_color_code(ColorCode::new(Color::LightCyan, Color::Black));
    writer.write_string("JC-OS Rust Kernel\n");
    writer.write_string("-----------------\n");
    
    writer.set_color_code(ColorCode::new(Color::White, Color::Black));
    writer.write_string("Ready.\n\n");
}

// --- TÂCHES ASYNCHRONES ---

async fn example_task() {
    let mut count: u64 = 0;
    loop {
        count += 1;
        if count % 1000000 == 0 {
            // serial_println!("[TASK] Compteur : {}", count);
        }
        crate::task::yield_now().await;
    }
}

async fn clock_task() {
    let mut last_second = 255;
    loop {
        let time = crate::drivers::rtc::get_time(); 
        if time.seconds != last_second {
            let mut writer = crate::vga_buffer::WRITER.lock();
            writer.write_clock(time.hours, time.minutes, time.seconds);
            last_second = time.seconds;
        }
        crate::task::yield_now().await;
    }
}