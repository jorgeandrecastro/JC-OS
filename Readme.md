# JC-OS — Bare-Metal Operating System Kernel

A **minimalist operating system kernel** written in Rust, designed to run directly on x86_64 hardware without any underlying operating system.

## 📖 Project Description

JC-OS is a personal hobby kernel project focused on low-level system programming. It runs in a `no_std` environment and is bootstrapped via the Rust `bootloader` crate on x86_64 systems.

This project demonstrates the fundamentals of OS creation:
- Custom boot via UEFI/Legacy bootloader
- Hardware management without system abstraction
- Direct communication with CPU and peripherals
- VGA video memory manipulation
- Hardware interrupt handling
- Virtual memory with paging
- Dynamic memory allocation (heap)

## ✨ Implemented Features

### Display
- **VGA text output** (80×25 characters)
- **16 foreground and background colors**
- **Automatic scrolling** when screen is full
- **Smart backspace** with line wrapping
- **Hardware cursor update** (ports 0x3D4/0x3D5)

### Input
- **PS/2 keyboard** with French AZERTY layout
- **Scancode Set 2** (IBM standard)
- **Complete alphanumeric key mapping**
- **Special keys**: Enter, Backspace, Escape

### System Management
- **GDT** (Global Descriptor Table) - CPU segmentation
- **TSS** (Task State Segment) with Double Fault stack
- **IDT** (Interrupt Descriptor Table) - Interrupt vectors
- **PIC 8259** - Programmable Interrupt Controller
- **Double Fault Handler** protected by IST (Interrupt Stack Table)

### Memory Management
- **x86_64 Paging** (4-level page tables)
- **Physical memory mapping** via bootloader info
- **Frame allocator** using UEFI memory map
- **Heap allocation** (100 KiB) with linked-list allocator
- **Virtual to physical address translation**
- **Page-level protection** (PRESENT, WRITABLE flags)

### Debugging
- **COM1 serial output** via UART 16550
- **Complete boot logging**
- **Memory statistics** display (heap start, size, status)
- **Panic error display** via serial

## 🏗️ System Architecture

```
┌─────────────────────────────────────────────────┐
│              JC-OS Kernel v0.1                   │
├─────────────────────────────────────────────────┤
│  Entry Point: kernel_main()                      │
├─────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────┐  │
│  │           Initialization Order            │  │
│  ├───────────────────────────────────────────┤  │
│  │  1. GDT + TSS     (CPU segmentation)      │  │
│  │  2. IDT           (interrupt table)       │  │
│  │  3. PIC           (interrupt controller)  │  │
│  │  4. PS/2 Controller (keyboard)            │  │
│  │  5. Keyboard Driver (AZERTY Set2)         │  │
│  │  6. Paging Setup  (4-level page tables)   │  │
│  │  7. Frame Allocator (memory map parsing)  │  │
│  │  8. Heap Init      (100 KiB allocator)    │  │
│  │  9. Interrupts enabled                    │  │
│  └───────────────────────────────────────────┘  │
├─────────────────────────────────────────────────┤
│  Memory Layout (Virtual Address Space)          │
│  ┌───────────────────────────────────────────┐  │
│  │  0x0000_0000_0000 - Kernel Code           │  │
│  │  ...                                      │  │
│  │  0x4444_4444_0000 - HEAP START (100 KiB)  │  │
│  │  0x4444_4444_19000 - HEAP END             │  │
│  │  ...                                      │  │
│  │  Higher half kernel (identity mapped)     │  │
│  └───────────────────────────────────────────┘  │
├─────────────────────────────────────────────────┤
│  Managed Peripherals:                           │
│  • VGA 0xB8000  - Text screen                  │
│  • COM1 0x3F8   - Serial port                  │
│  • PIC 0x20/0xA0 - Interrupt controller        │
│  • PS/2 0x60/0x64 - Keyboard                   │
└─────────────────────────────────────────────────┘
```

## 📁 Project Structure

```
jc-os/
├── Cargo.toml                    # Rust project configuration
├── Readme.md                     # This file
├── x86_64-jc-os.json             # Custom target spec
├── src/
│   ├── main.rs                   # Entry point + initialization
│   ├── gdt.rs                    # GDT + TSS (segmentation)
│   ├── interrupts.rs             # IDT + PIC handling + handlers
│   ├── vga_buffer.rs             # Color VGA text driver
│   ├── serial.rs                 # COM1 serial output (UART 16550)
│   ├── memory.rs                 # Paging + frame allocator
│   ├── allocator.rs              # Heap allocator (linked-list)
│   └── drivers/
│       ├── mod.rs                # Drivers module (export)
│       ├── keyboard.rs           # PS/2 AZERTY keyboard driver
│       └── mouse.rs              # PS/2 mouse driver (in development)
└── target/
    └── x86_64-jc-os/             # Compiled binaries
```

## 🔧 Detailed Components

### 1. GDT (`src/gdt.rs`)
**Role**: CPU memory segmentation configuration

```
• Kernel Code Segment (64-bit execution)
• TSS (Task State Segment) for:
  - Double Fault Handler stack
  - IST Index 0: 5 stack pages (20KB)
```

### 2. IDT (`src/interrupts.rs`)
**Role**: Routes interrupts to appropriate handlers

```
Configured Vectors:
• Double Fault (CPU Exception) → Isolated stack
• Timer (IRQ0)    → Basic handler
• Keyboard (IRQ1) → Keyboard driver

PIC Configuration:
• Master: Timer + Keyboard enabled (0xF8)
• Slave:  All disabled (0xFF)
```

### 3. Memory Management (`src/memory.rs`)
**Role**: Paging and physical memory allocation

```
Features:
• 4-Level Paging (PML4 → PDP → PD → PT)
• CR3 register read for active page table
• OffsetPageTable for higher-half mapping
• BootInfoFrameAllocator uses UEFI memory map

Memory Map Parsing:
• Iterates through bootloader memory regions
• Filters for Usable memory type
• Allocates 4KiB frames for page mapping
• Tracks next available frame index
```

**Memory Map Entry Example:**
```
Region types:
• Usable RAM          → Can be allocated
• Reserved            → Not available
• ACPI Reclaimable    → Can be used after ACPI
• EFI Runtime         → Reserved for firmware
```

### 4. Heap Allocator (`src/allocator.rs`)
**Role**: Dynamic memory allocation for kernel

```
Configuration:
• Heap Start:  0x4444_4444_0000 (virtual)
• Heap Size:   100 KiB
• Allocator:   linked_list_allocator::LockedHeap
• Page Flags:  PRESENT | WRITABLE

Initialization:
• Maps 25 pages (25 × 4KiB = 100 KiB)
• Initializes LockedHeap with start pointer
• Provides heap_start() and heap_size() queries

Memory Statistics (displayed at boot):
• Heap Start : 0x444444440000
• Heap Size  : 100 KB
• Status     : DYNAMIC ALLOCATION OK
```

### 5. VGA Buffer (`src/vga_buffer.rs`)
**Role**: Text display on VGA screen

```
Specifications:
• Address: 0xB8000
• Size: 80 × 25 = 2000 characters
• Attributes: 1 color byte + 1 character byte

Features:
• 16 ANSI colors (Black → White)
• Automatic scroll with line preservation
• Smart backstack (wraps to previous line)
• Hardware cursor update
```

### 6. PS/2 Keyboard (`src/drivers/keyboard.rs`)
**Role**: Translates scancodes to characters

```
Configuration:
• Layout: French AZERTY
• Scancode Set: 2 (IBM standard)
• Control: Ignore Ctrl (for testing)

Handled Keys:
• Letters a-z, digits 0-9
• AZERTY special characters
• Enter, Backspace, Escape
```

### 7. Serial Port (`src/serial.rs`)
**Role**: Debugging via serial connection

```
Configuration:
• Port: COM1 (0x3F8)
• UART: 16550 standard
• Output: stdout during QEMU debugging

Usage:
• Boot log: "[JC-OS] Booting..."
• System log: "[GDT] Loaded", "[IDT] Loaded"
• Memory stats: "Heap Start: 0x..."
• Panic display
```

## 🚀 Installation and Compilation

### Prerequisites

```bash
# Rust nightly with bare-metal target
rustup target add x86_64-unknown-none

# Bootable image creation tool
cargo install bootimage

# Required LLVM components
rustup component add llvm-tools-preview

# QEMU emulator
# Ubuntu/Debian: sudo apt install qemu-system-x86
# Arch: sudo pacman -S qemu
# macOS: brew install qemu
```

### Compilation and Execution

```bash
# Debug mode (fast, with asserts)
cargo run

# Release mode (optimized, faster)
cargo run --release

# Create bootable image only
cargo bootimage

# Run bootable image with QEMU
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-jc-os/release/bootimage-jc-os.bin \
  -serial stdio
```

## ⌨️ Keyboard Commands

| Key | Action |
|-----|--------|
| `a` - `z` | Lowercase letter input |
| `A` - `Z` | Uppercase letter input |
| `0` - `9` | Digits |
| `è` `é` `ê` `ë` | French accented characters |
| `,` `;` `:` `!` | Special characters |
| `Enter` | New line + carriage return |
| `Backspace` | Delete previous character |
| `Esc` | Clear entire screen |

## 🔍 Example Session

```
qemu-system-x86_64 -drive format=raw,file=target/x86_64-jc-os/debug/bootimage-jc-os.bin -serial stdio

[JC-OS] Booting...
[GDT] Loaded
[IDT] Interrupt Descriptor Table loaded
[PIC] Initialized - Timer and Keyboard enabled
[PS/2] Keyboard controller initialized
[KEYBOARD] Driver initialized (AZERTY layout, Set2)
[PAGING] 4-Level page tables initialized
[FRAMES] Boot info frame allocator ready
[HEAP] Heap initialized at 0x444444440000 (100 KiB)
[SYSTEM] Interrupts enabled

--- JC-OS MEMORY STATS ---
Heap Start : 0x444444440000
Heap Size  : 100 KB
Status     : DYNAMIC ALLOCATION OK

╔════════════════════════════════════════════════════════════════════════╗
║              JC-OS - BARE METAL KERNEL v0.1                            ║
╚════════════════════════════════════════════════════════════════════════╝

Keyboard active. Start typing...

>>> Hello JC-OS!
```

## 📦 Cargo Dependencies

| Crate | Version | Usage |
|-------|---------|-------|
| `bootloader` | 0.9.23 | Kernel bootstrapping + memory map |
| `x86_64` | 0.14 | x86_64 CPU structures + paging |
| `spin` | 0.9 | Lock-free synchronization |
| `pc-keyboard` | 0.7.0 | PS/2 scancode parsing |
| `pic8259` | 0.10.1 | 8259 PIC controller |
| `uart_16550` | 0.2.0 | COM1 serial port |
| `lazy_static` | 1.4.0 | Deferred static initialization |
| `volatile` | 0.2.6 | VGA volatile memory access |
| `linked_list_allocator` | 0.10 | Heap allocation algorithm |

## 🐛 Troubleshooting

### QEMU not found
```bash
# Check installation
which qemu-system-x86_64

# Install if needed
sudo apt install qemu-system-x86  # Debian/Ubuntu
```

### "target not found" compilation error
```bash
# Add x86_64-unknown-none target
rustup target add x86_64-unknown-none
```

### No VGA output
Verify VGA graphics mode is enabled in QEMU with `-vga std`.

### No serial output
Use `-serial stdio` parameter to redirect COM1 to the terminal.

### Heap allocation failed
Ensure enough physical memory is available (QEMU default: 128MiB).
Increase with: `-m 256M`

## 🔮 Future Improvements

- [ ] **PS/2 Mouse Driver** - On-screen cursor tracking
- [ ] **Page Fault Handler** - Better memory error reporting
- [ ] **Kernel Heap Expansion** - Dynamic heap growth
- [ ] **File System** - FAT32 reading
- [ ] **Interactive Shell** - User commands
- [ ] **Multi-tasking Support** - Preemptive scheduling
- [ ] **Virtual File System** - VFS abstraction layer


## 📄 License

This project is licensed under Apache 2.0.

## 🤝 Contributions

Issues and pull requests are welcome to improve the project!

