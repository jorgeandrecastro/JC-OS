// src/drivers/mouse.rs
// [Copyright 2026 Andre - Apache 2.0]

use x86_64::instructions::port::Port;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::vga_buffer;

lazy_static! {
    static ref MOUSE_STATE: Mutex<MouseState> = Mutex::new(MouseState::new());
}

struct MouseState {
    phase: u8,
    buffer: [u8; 3],
    x: i32,
    y: i32,
    old_x: i32,
    old_y: i32,
}

impl MouseState {
    fn new() -> Self {
        Self { 
            phase: 0, 
            buffer: [0; 3], 
            x: 40, 
            y: 12,
            old_x: 40,
            old_y: 12,
        }
    }
}

pub fn init() {
    unsafe {
        let mut cmd_port: Port<u8> = Port::new(0x64);
        let mut data_port: Port<u8> = Port::new(0x60);

        // Activer le port auxiliaire
        mouse_wait(1);
        cmd_port.write(0xA8);

        // Activer les interruptions (Command Byte)
        mouse_wait(1);
        cmd_port.write(0x20);
        mouse_wait(0);
        let status = data_port.read() | 2;
        mouse_wait(1);
        cmd_port.write(0x60);
        mouse_wait(1);
        data_port.write(status);

        // Set Defaults
        mouse_write(0xF6);
        let _ = mouse_read();

        // Enable Data Reporting
        mouse_write(0xF4);
        let _ = mouse_read();
    }
}

fn mouse_wait(typ: u8) {
    let mut port: Port<u8> = Port::new(0x64);
    unsafe {
        if typ == 0 {
            while (port.read() & 1) == 0 {}
        } else {
            while (port.read() & 2) != 0 {}
        }
    }
}

fn mouse_write(data: u8) {
    let mut cmd_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);
    mouse_wait(1);
    unsafe {
        cmd_port.write(0xD4);
        mouse_wait(1);
        data_port.write(data);
    }
}

fn mouse_read() -> u8 {
    let mut data_port: Port<u8> = Port::new(0x60);
    mouse_wait(0);
    unsafe { data_port.read() }
}

pub fn add_mouse_data(data: u8) {
    let mut state = MOUSE_STATE.lock();
    
    match state.phase {
        0 => {
            if (data & 0x08) != 0 {
                state.buffer[0] = data;
                state.phase = 1;
            }
        }
        1 => {
            state.buffer[1] = data;
            state.phase = 2;
        }
        2 => {
            state.buffer[2] = data;
            
            // Correction de la syntaxe de décalage avec parenthèses explicites
            let dx = state.buffer[1] as i32 - (((state.buffer[0] as i32) << 4) & 0x100);
            let dy = state.buffer[2] as i32 - (((state.buffer[0] as i32) << 3) & 0x100);

            state.old_x = state.x;
            state.old_y = state.y;

            state.x = (state.x + dx / 2).clamp(0, 79);
            state.y = (state.y - dy / 2).clamp(0, 24);

            draw_cursor(state.x, state.y, state.old_x, state.old_y);
            
            state.phase = 0;
        }
        _ => state.phase = 0,
    }
}

fn draw_cursor(x: i32, y: i32, _old_x: i32, _old_y: i32) {
    // Utilisation de _ pour les variables inutilisées pour éviter les warnings
    let _writer = vga_buffer::WRITER.lock();
    
    // Pour l'instant on ne fait rien pour tester si ça compile
    // On pourra implémenter l'inversion de couleur ici !
}