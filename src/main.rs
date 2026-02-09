// main.rs - JC-OS Kernel Entry Point
// Version 0.2 - Andre Edition

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


mod vga_buffer;
mod serial;
mod interrupts;
mod gdt;
mod drivers;
mod memory;
mod allocator;
mod fs; // Important: link the new file system
mod shell; // Important: link the new shell
mod auth;// Important: link the new authentication module
pub mod task;
pub mod executor;



entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    serial_println!("[JC-OS] Kernel starting...");

    // 1. Core Architecture Setup
    gdt::init();
    interrupts::init_idt();
    interrupts::init_pic();
    
    // 2. Memory Management (Paging & Frame Allocation)
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // 3. Dynamic Memory Allocation (Heap)
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");

    serial_println!("[SYSTEM] Heap Allocator Ready");

    // 4. Input Drivers
    init_keyboard_controller();
    drivers::keyboard::init();
    
    // 5. Activation
    x86_64::instructions::interrupts::enable();
    
    // 6. UI Launch
    display_screen();

    // --- ICI LE CHANGEMENT POUR LE MULTITACHE ---
    let mut executor = Executor::new();

    // 1. On lance le Shell (il va afficher le prompt et gérer les commandes)")
    executor.spawn(Task::new(shell::run_shell()));
    
    // 2. On garde l'horloge
    executor.spawn(Task::new(clock_task()));   

    // 3. On peut garder les autres tâches de fond
    executor.spawn(Task::new(example_task())); 

    serial_println!("[SYSTEM] Multitasking Executor Started");
    
    executor.run();
    
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Alloc Error: {:?}", layout)
}

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
    writer.set_color_code(ColorCode::new(Color::LightCyan, Color::Black));
    writer.write_string("╔════════════════════════════════════════════════════════════════╗\n");
    writer.write_string("║           JC-OS - BARE METAL KERNEL v0.4 - RUST              ║\n");
    writer.write_string("╚════════════════════════════════════════════════════════════════╝\n\n");
    writer.set_color_code(ColorCode::new(Color::White, Color::Black));
    writer.write_string("Digital Sovereignty System \n");
    writer.write_string("File System: READY (RAMFS) | Commands exemples: touch, ls, cat, rm, edit\n\n");

}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("\n[PANIC] {}", info);
    loop { x86_64::instructions::hlt(); }
}

async fn example_task() {
    let mut count: u64 = 0;
    loop {
        count += 1;
        if count % 1000000 == 0 {
            // En l'utilisant ici, le warning disparaît
            serial_println!("[TASK] Compteur : {}", count);
        }
        crate::task::yield_now().await;
    }
}

// Tâche pour afficher l'horloge en temps réel
async fn clock_task() {
    let mut last_second = 255;

    loop {
        // Changement du chemin vers crate::rtc::get_time()
        let time = crate::drivers::rtc::get_time(); // <--- VERIFIE CE CHEMIN
        
        if time.seconds != last_second {
            let mut writer = crate::vga_buffer::WRITER.lock();
            writer.write_clock(time.hours, time.minutes, time.seconds);
            last_second = time.seconds;
        }

        crate::task::yield_now().await;
    }
}