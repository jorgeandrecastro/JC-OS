use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use crate::drivers::keyboard;
use crate::serial_println;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer    = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self as u8)
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
    serial_println!("[IDT] Interrupt Descriptor Table loaded");
}

pub fn init_pic() {
    unsafe {
        PICS.lock().initialize();
        
        // Master: Active Timer (IRQ0) et Keyboard (IRQ1)
        // Slave: Désactive tout pour l'instant
        PICS.lock().write_masks(0xF8, 0xFF);
        
        serial_println!("[PIC] Initialized - Timer and Keyboard enabled");
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("\n[EXCEPTION] DOUBLE FAULT");
    serial_println!("{:#?}", stack_frame);
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // On ne fait rien pour l'instant, mais on doit notifier la fin de l'interruption
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    // Port 0x60 est le port de données du contrôleur PS/2
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    // On envoie le scancode au driver qui contient la machine à états (pc-keyboard)
    keyboard::add_scancode(scancode);

    // Notifier le PIC que l'interruption est terminée AVANT de sortir
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}