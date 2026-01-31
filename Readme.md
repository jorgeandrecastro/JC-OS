# JC-OS — Bare-Metal Operating System Kernel

A minimal **x86_64 bare-metal operating system kernel** written in Rust, designed to run directly on hardware without any underlying OS.

## Overview

JC-OS is a personal **hobby kernel** focused on low-level system programming.  
It runs in a `no_std` environment and is bootstrapped using the Rust `bootloader` crate on x86_64 systems.

The kernel provides a VGA text-mode interface with color support, PS/2 keyboard input, basic interrupt handling, and serial output for debugging.  
No attempt is made to be production-ready.

## Features

- **Bare-metal boot** using the Rust `bootloader` crate (x86_64, legacy platform)
- **VGA text-mode output** (80×25) with 16 foreground/background colors
- **PS/2 keyboard driver** (French AZERTY, Scancode Set 2)
- **Interrupt handling** via IDT and PIC 8259
- **Double Fault handler** protected with IST (Interrupt Stack Table)
- **Serial output (UART 16550)** for debugging
- **Automatic screen scrolling** and intelligent backspace handling

## Architecture

┌─────────────────────────────────────────┐
│ User Applications │ <- Not implemented
├─────────────────────────────────────────┤
│ Shell / CLI │ <- VGA input area
├─────────────────────────────────────────┤
│ Keyboard Driver │ <- PS/2 AZERTY
├─────────────────────────────────────────┤
│ Interrupt Handling (IDT/PIC) │
├─────────────────────────────────────────┤
│ Memory Setup (GDT / TSS) │
├─────────────────────────────────────────┤
│ VGA Text Buffer │
├─────────────────────────────────────────┤
│ Bootloader │
└─────────────────────────────────────────

## Core Components

| Module | File | Responsibility |
|------|------|----------------|
| GDT / TSS | `src/gdt.rs` | CPU segmentation, TSS setup |
| IDT | `src/interrupts.rs` | Interrupt descriptor table |
| VGA | `src/vga_buffer.rs` | Text-mode VGA driver |
| Serial | `src/serial.rs` | COM1 serial output |
| Keyboard | `src/drivers/keyboard.rs` | PS/2 keyboard driver |

## Requirements

- **Rust nightly** with target `x86_64-unknown-none`
- **QEMU** (x86_64 emulator)
- **bootimage** (`cargo install bootimage`)
- **llvm-tools-preview** (`rustup component add llvm-tools-preview`)

## Setup

```bash
rustup target add x86_64-unknown-none
cargo install bootimage
rustup component add llvm-tools-preview


##Build and Run

# Debug
cargo run

# Release
cargo run --release

# Bootable image + QEMU
cargo bootimage
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-jc-os/release/bootimage-jc-os.bin \
  -serial stdio


##Keyboard Shortcuts

| Key         | Action           |
| ----------- | ---------------- |
| AZERTY keys | Text input       |
| Enter       | New line         |
| Backspace   | Delete character |
| Escape      | Clear screen     |
| Ctrl+C      | Insert "^C"      |


##Project Layout

jc-os/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── gdt.rs
│   ├── interrupts.rs
│   ├── vga_buffer.rs
│   ├── serial.rs
│   └── drivers/
│       └── keyboard.rs
├── target/
│   └── x86_64-jc-os/
└── x86_64-jc-os.json
##Dependencies
| Crate       | Version | Purpose                   |
| ----------- | ------- | ------------------------- |
| bootloader  | 0.9.23  | Kernel bootstrapping      |
| x86_64      | 0.14    | CPU structures            |
| spin        | 0.9     | Lock-free synchronization |
| pc-keyboard | 0.7.0   | Scancode parsing          |
| pic8259     | 0.10.1  | Legacy PIC                |
| uart_16550  | 0.2.0   | Serial I/O                |
| lazy_static | 1.4.0   | Deferred statics          |
| volatile    | 0.2.6   | Volatile memory access    |
License

Apache License 2.0.

Contributing

Issues and pull requests are welcome.