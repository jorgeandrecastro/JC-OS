// main.rs - JC-OS Kernel Entry Point (Keyboard Only)

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

mod vga_buffer;
mod serial;
mod interrupts;
mod gdt;
mod drivers;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    serial_println!("[JC-OS] Booting...");

    // 1. GDT + TSS
    gdt::init();
    
    // 2. IDT
    interrupts::init_idt();
    
    // 3. PIC
    interrupts::init_pic();
    
    // 4. PS/2 Keyboard
    init_keyboard_controller();
    
    // 5. Keyboard driver
    drivers::keyboard::init();
    
    // 6. Enable interrupts
    x86_64::instructions::interrupts::enable();
    serial_println!("[SYSTEM] Interrupts enabled");

    // Display screen
    display_screen();

    serial_println!("[KERNEL] Ready. Type something!");

    loop {
        x86_64::instructions::hlt();
    }
}

fn init_keyboard_controller() {
    use x86_64::instructions::port::Port;
    
    unsafe {
        let mut cmd: Port<u8> = Port::new(0x64);
        let mut data: Port<u8> = Port::new(0x60);
        
        // Flush output buffer
        for _ in 0..16 {
            if (cmd.read() & 0x01) != 0 {
                let _ = data.read();
            }
        }
        
        // Disable devices
        cmd.write(0xAD); // Disable keyboard
        cmd.write(0xA7); // Disable mouse
        
        // Flush again
        if (cmd.read() & 0x01) != 0 {
            let _ = data.read();
        }
        
        // Get config byte
        cmd.write(0x20);
        for _ in 0..100 { core::hint::spin_loop(); }
        let mut config = data.read();
        
        // Enable keyboard interrupt (bit 0), disable mouse (bit 1), disable translation (bit 6)
        config |= 0x01;   // Enable keyboard interrupt
        config &= !0x02;  // Disable mouse interrupt
        config &= !0x40;  // Disable translation
        
        // Set config byte
        cmd.write(0x60);
        for _ in 0..100 { core::hint::spin_loop(); }
        data.write(config);
        
        // Enable keyboard
        cmd.write(0xAE);
        
        // Final flush
        for _ in 0..100 { core::hint::spin_loop(); }
        if (cmd.read() & 0x01) != 0 {
            let _ = data.read();
        }
    }
    
    serial_println!("[PS/2] Keyboard controller initialized");
}

fn display_screen() {
    use vga_buffer::{WRITER, ColorCode, Color};
    
    let mut writer = WRITER.lock();
    writer.clear_screen();
    writer.set_color_code(ColorCode::new(Color::LightCyan, Color::Black));
    writer.write_string("╔════════════════════════════════════════════════════════════════╗\n");
    writer.write_string("║              JC-OS - BARE METAL KERNEL v0.1                    ║\n");
    writer.write_string("╚════════════════════════════════════════════════════════════════╝\n\n");
    writer.set_color_code(ColorCode::new(Color::White, Color::Black));
    writer.write_string("Keyboard active. Start typing...\n\n");
    writer.write_string(">>> ");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("\n[PANIC] {}", info);
    loop { x86_64::instructions::hlt(); }
}