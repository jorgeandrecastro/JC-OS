// main.rs - JC-OS Kernel Entry Point (Memory & Keyboard Shell)
// Version 0.2 - Andre Edition

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)] // Required to handle memory errors

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

mod vga_buffer;
mod serial;
mod interrupts;
mod gdt;
mod drivers;
mod memory;
mod allocator;

// NOTE: No need to "use crate::print;" anymore since #[macro_export] 
// makes the macro available everywhere automatically. Manual import caused conflicts.

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    serial_println!("[JC-OS] Booting...");

    // 1. Initialize GDT and IDT (CPU Architecture)
    gdt::init();
    interrupts::init_idt();
    interrupts::init_pic();
    
    // 2. Initialize Memory (Paging)
    // FIX: On Bootloader 0.9.33, physical_memory_offset is a direct u64.
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // 3. Initialize the allocator (Heap)
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");

    serial_println!("[SYSTEM] Memory management & Allocator ready");

    // 4. Initialize drivers
    init_keyboard_controller();
    drivers::keyboard::init();
    
    // 5. Enable interrupts
    x86_64::instructions::interrupts::enable();
    serial_println!("[SYSTEM] Interrupts enabled");

    // 6. Display the interface
    display_screen();

    // MEMORY TEST: Check if allocations work
    let test_box = Box::new(42);
    serial_println!("[MEM] Test Box: {} at {:p}", *test_box, test_box);

    serial_println!("[KERNEL] System stable. Waiting for input...");

    loop {
        x86_64::instructions::hlt();
    }
}

/// Handle errors if the heap is full
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

fn init_keyboard_controller() {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut cmd: Port<u8> = Port::new(0x64);
        let mut data: Port<u8> = Port::new(0x60);
        
        for _ in 0..16 {
            if (cmd.read() & 0x01) != 0 { let _ = data.read(); }
        }
        
        cmd.write(0xAD); // Disable keyboard
        cmd.write(0xA7); // Disable mouse
        
        cmd.write(0x20);
        let mut config = data.read();
        config |= 0x01;   // Enable IRQ 1
        config &= !0x02;  // Disable IRQ 12 (mouse)
        config &= !0x40;  // Disable translation
        
        cmd.write(0x60);
        data.write(config);
        cmd.write(0xAE); // Enable keyboard
    }
    serial_println!("[PS/2] Keyboard controller initialized");
}

fn display_screen() {
    use vga_buffer::{WRITER, ColorCode, Color};
    let mut writer = WRITER.lock();
    writer.clear_screen();
    writer.set_color_code(ColorCode::new(Color::LightCyan, Color::Black));
    writer.write_string("╔════════════════════════════════════════════════════════════════╗\n");
    writer.write_string("║           JC-OS - BARE METAL KERNEL v0.2 - RUST              ║\n");
    writer.write_string("╚════════════════════════════════════════════════════════════════╝\n\n");
    writer.set_color_code(ColorCode::new(Color::White, Color::Black));
    writer.write_string("Digital Sovereignty System \n");
    writer.write_string("Dynamic Memory: ACTIVE\n\n");
    writer.write_string(">>> ");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("\n[PANIC] {}", info);
    // You can call print! here without import since it is exported globally
    // print!("\n!!! KERNEL PANIC !!!\n{}", info); 
    loop { x86_64::instructions::hlt(); }
}
