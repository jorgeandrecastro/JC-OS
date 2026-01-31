// src/drivers/mouse.rs

use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;
use crate::serial_println;

lazy_static! {
    static ref MOUSE_STATE: Mutex<MouseState> = Mutex::new(MouseState::new());
}

#[derive(Copy, Clone, Debug)]
struct MouseState {
    packet_index: usize,
    packet: [u8; 3],
    x_position: i32,
    y_position: i32,
    initialized: bool,
}

impl MouseState {
    const fn new() -> Self {
        MouseState { 
            packet_index: 0, 
            packet: [0; 3],
            x_position: 40,
            y_position: 12,
            initialized: false,
        }
    }
}

pub fn init() {
    serial_println!("[MOUSE] Initializing PS/2 mouse...");
    
    unsafe {
        let mut command_port = Port::new(0x64);
        let mut data_port = Port::new(0x60);

        wait_controller(&mut command_port);
        command_port.write(0xA8);
        serial_println!("[MOUSE] Auxiliary port enabled");

        for _ in 0..10000 { core::hint::spin_loop(); }

        wait_controller(&mut command_port);
        command_port.write(0x20);
        wait_data(&mut command_port);
        let mut config = data_port.read();
        
        serial_println!("[MOUSE] Current config byte: 0x{:02x}", config);
        
        config |= 0b0000_0010;
        config &= !0b0010_0000;
        
        wait_controller(&mut command_port);
        command_port.write(0x60);
        wait_controller(&mut command_port);
        data_port.write(config);
        
        serial_println!("[MOUSE] Config byte updated to: 0x{:02x}", config);

        if !write_mouse_command(0xFF, &mut command_port, &mut data_port) {
            serial_println!("[MOUSE] WARNING: Reset command failed");
        } else {
            serial_println!("[MOUSE] Reset successful");
            
            for _ in 0..3 {
                if wait_data_timeout(&mut command_port, 100000) {
                    let response = data_port.read();
                    serial_println!("[MOUSE] Reset response: 0x{:02x}", response);
                }
            }
        }

        if !write_mouse_command(0xF6, &mut command_port, &mut data_port) {
            serial_println!("[MOUSE] WARNING: Set defaults failed");
        } else {
            serial_println!("[MOUSE] Defaults set");
        }

        if !write_mouse_command(0xF4, &mut command_port, &mut data_port) {
            serial_println!("[MOUSE] ERROR: Enable data reporting failed!");
        } else {
            serial_println!("[MOUSE] Data reporting enabled");
        }
    }

    MOUSE_STATE.lock().initialized = true;
    
    serial_println!("[MOUSE] ✓ PS/2 mouse initialized successfully (IRQ12 enabled)");
    serial_println!("[MOUSE] Note: Make sure IRQ12 is unmasked in PIC configuration!");
}

pub fn handle_mouse_packet(byte: u8) {
    let mut state = MOUSE_STATE.lock();

    if !state.initialized {
        serial_println!("[MOUSE] Received byte 0x{:02x} but mouse not initialized", byte);
        return;
    }

    let index = state.packet_index;

    if index >= 3 {
        serial_println!("[MOUSE] WARNING: packet_index overflow, resetting");
        state.packet_index = 0;
        return;
    }

    if index == 0 {
        if (byte & 0x08) == 0 {
            serial_println!("[MOUSE] Invalid packet start (bit 3 not set): 0x{:02x}, discarding", byte);
            state.packet_index = 0;
            return;
        }
    }

    state.packet[index] = byte;
    state.packet_index = index + 1;

    if state.packet_index == 3 {
        let packet = state.packet;

        let flags = packet[0];
        let left_btn   = (flags & 0x01) != 0;
        let right_btn  = (flags & 0x02) != 0;
        let middle_btn = (flags & 0x04) != 0;
        let x_sign     = (flags & 0x10) != 0;
        let y_sign     = (flags & 0x20) != 0;
        let x_overflow = (flags & 0x40) != 0;
        let y_overflow = (flags & 0x80) != 0;

        let mut x_delta = packet[1] as i16;
        let mut y_delta = packet[2] as i16;

        if x_sign {
            x_delta |= 0xFF00u16 as i16;
        }
        if y_sign {
            y_delta |= 0xFF00u16 as i16;
        }

        let y_delta = -y_delta;

        if x_overflow || y_overflow {
            serial_println!("[MOUSE] ⚠ OVERFLOW detected - packet discarded");
            state.packet_index = 0;
            return;
        }

        state.x_position = (state.x_position + x_delta as i32).clamp(0, 79);
        state.y_position = (state.y_position + y_delta as i32).clamp(0, 24);

        serial_println!(
            "[MOUSE] Pos({:2},{:2}) | Δ({:+4},{:+4}) | Btn L:{} M:{} R:{} | Raw:[0x{:02x} 0x{:02x} 0x{:02x}]",
            state.x_position,
            state.y_position,
            x_delta,
            y_delta,
            if left_btn {"●"} else {"○"},
            if middle_btn {"●"} else {"○"},
            if right_btn {"●"} else {"○"},
            packet[0],
            packet[1],
            packet[2]
        );

        state.packet_index = 0;
    }
}

unsafe fn wait_controller(cmd_port: &mut Port<u8>) {
    let mut timeout = 100000;
    while timeout > 0 && (cmd_port.read() & 0x02) != 0 {
        timeout -= 1;
        core::hint::spin_loop();
    }
    if timeout == 0 {
        serial_println!("[MOUSE] WARNING: wait_controller timeout");
    }
}

unsafe fn wait_data(cmd_port: &mut Port<u8>) {
    let mut timeout = 100000;
    while timeout > 0 && (cmd_port.read() & 0x01) == 0 {
        timeout -= 1;
        core::hint::spin_loop();
    }
    if timeout == 0 {
        serial_println!("[MOUSE] WARNING: wait_data timeout");
    }
}

unsafe fn wait_data_timeout(cmd_port: &mut Port<u8>, timeout: u32) -> bool {
    let mut count = timeout;
    while count > 0 && (cmd_port.read() & 0x01) == 0 {
        count -= 1;
        core::hint::spin_loop();
    }
    count > 0
}

unsafe fn write_mouse_command(command: u8, cmd_port: &mut Port<u8>, data_port: &mut Port<u8>) -> bool {
    wait_controller(cmd_port);
    cmd_port.write(0xD4);
    wait_controller(cmd_port);
    data_port.write(command);
    
    if wait_data_timeout(cmd_port, 100000) {
        let response = data_port.read();
        if response == 0xFA {
            serial_println!("[MOUSE] Command 0x{:02x} acknowledged", command);
            return true;
        } else {
            serial_println!("[MOUSE] Command 0x{:02x} got response 0x{:02x} (expected 0xFA)", command, response);
            return false;
        }
    } else {
        serial_println!("[MOUSE] Command 0x{:02x} timeout waiting for ACK", command);
        return false;
    }
}

#[allow(dead_code)]
pub fn get_position() -> (i32, i32) {
    let state = MOUSE_STATE.lock();
    (state.x_position, state.y_position)
}

#[allow(dead_code)]
pub fn is_button_pressed(button: MouseButton) -> bool {
    let state = MOUSE_STATE.lock();
    if state.packet_index > 0 {
        let flags = state.packet[0];
        match button {
            MouseButton::Left   => (flags & 0x01) != 0,
            MouseButton::Right  => (flags & 0x02) != 0,
            MouseButton::Middle => (flags & 0x04) != 0,
        }
    } else {
        false
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}