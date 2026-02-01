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
- RAM-based file system with interactive shell
- Async/await task scheduling with executor

## ✨ Implemented Features

### Display
- **VGA text output** (80×25 characters)
- **16 foreground and background colors**
- **Automatic scrolling** when screen is full
- **Smart backspace** with line wrapping
- **Hardware cursor update** (ports 0x3D4/0x3D5)
- **Color-coded UI elements** with border boxes

### Input
- **PS/2 keyboard** with French AZERTY layout
- **Scancode Set 2** (IBM standard)
- **Complete alphanumeric key mapping**
- **Special keys**: Enter, Backspace, Escape
- **Command buffer** with 256 character capacity

### File System
- **RAM File System (RAMFS)** - In-memory file storage
- **BTreeMap-based organization** for efficient file lookup
- **File operations**: create, read, write, delete, list
- **Statistics tracking**: file count and total size
- **Unicode support** via UTF-8 lossless conversion

### Interactive Shell
- **Command interpreter** with multiple built-in commands
- **File management commands**: touch, cat, rm, edit
- **System information**: info, stats, whoami, neofetch
- **Utility commands**: help, echo, clear, ls

### Task Scheduling
- **Async/await support** with Rust futures
- **Task executor** with round-robin scheduling
- **Task queue** using VecDeque
- **Cooperative multitasking** via yield_now()
- **Task identification** with atomic TaskId
- **Pin-based future pinning** for safe async execution

### System Management
- **GDT** (Global Descriptor Table) - CPU segmentation
- **TSS** (Task State Segment) with Double Fault stack
- **IDT** (Interrupt Descriptor Table) - Interrupt vectors
- **PIC 8259** - Programmable Interrupt Controller
- **Double Fault Handler** protected by IST (Interrupt Stack Table)
- **Timer Interrupt** - Hardware timer (IRQ0) for future scheduling
- **Keyboard Interrupt** - PS/2 keyboard input handling

### Memory Management
- **x86_64 Paging** (4-level page tables)
- **Physical memory mapping** via bootloader info
- **Frame allocator** using UEFI memory map
- **Heap allocation** (100 KiB) with linked-list allocator
- **Virtual to physical address translation**
- **Page-level protection** (PRESENT, WRITABLE flags)
- **Error handling** with Result type and alloc_error_handler

### Debugging
- **COM1 serial output** via UART 16550
- **Complete boot logging**
- **Memory statistics** display (heap start, size, status)
- **Panic error display** via serial
- **Interrupt event logging**

## 🏗️ System Architecture

```
┌─────────────────────────────────────────────────┐
│              JC-OS Kernel v0.2                   │
│              Andre Edition                       │
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
│  │  9. File System    (RAMFS initialization) │  │
│  │  10. Task System    (Executor init)       │  │
│  │  11. Interrupts enabled                   │  │
│  │  12. UI Launch     (shell prompt)         │  │
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
│  Task Scheduling Architecture                    │
│  ┌───────────────────────────────────────────┐  │
│  │  Executor                                  │  │
│  │  ├── Task Queue: VecDeque<Task>           │  │
│  │  ├── spawn(task) → push_back              │  │
│  │  └── run() → poll futures in loop         │  │
│  │                                            │  │
│  │  Task                                      │  │
│  │  ├── id: TaskId (atomic u64)              │  │
│  │  └── future: Pin<Box<dyn Future>>         │  │
│  │                                            │  │
│  │  YieldNow Future                          │  │
│  │  ├── yielded: bool                        │  │
│  │  └── poll() → Pending/Ready               │  │
│  └───────────────────────────────────────────┘  │
├─────────────────────────────────────────────────┤
│  Managed Peripherals:                           │
│  • VGA 0xB8000  - Text screen                  │
│  • COM1 0x3F8   - Serial port                  │
│  • PIC 0x20/0xA0 - Interrupt controller        │
│  • PS/2 0x60/0x64 - Keyboard                   │
│  • PIT 0x40     - Programmable Interval Timer  │
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
│   ├── fs.rs                     # RAM File System (RAMFS)
│   ├── task.rs                   # Task structures + async support
│   ├── executor.rs               # Task executor + scheduler
│   └── drivers/
│       ├── mod.rs                # Drivers module (export)
│       ├── keyboard.rs           # PS/2 AZERTY keyboard driver + shell
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
• Timer (IRQ0)    → Basic handler (for future scheduling)
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
• Returns Result<(), ()> for error handling

Memory Statistics (displayed at boot):
• Heap Start : 0x444444440000
• Heap Size  : 100 KB
• Status     : DYNAMIC ALLOCATION OK
```

### 5. RAM File System (`src/fs.rs`)
**Role**: In-memory file storage and management

```
Structure:
• File: name (String) + data (Vec<u8>)
• RamFileSystem: BTreeMap<String, File>
• Global instance protected by Mutex

Features:
• write_file(name, content) - Create/overwrite files
• read_file(name) - Read file as String (returns Option)
• list_files() - Returns Vec<String> of all filenames
• remove_file(name) - Delete file (returns bool)
• get_stats() - Returns (file_count, total_bytes)

Storage:
• In-memory only (volatile)
• Unicode support via UTF-8 lossless conversion
• No persistence (data lost on reboot)
```

### 6. Task Management (`src/task.rs`)
**Role**: Async task structures and cooperative multitasking

```
TaskId:
• Atomic u64 counter for unique identification
• Thread-safe ID generation
• Implements Debug, Clone, Copy, Eq, Ord

Task:
• id: TaskId - Unique task identifier
• future: Pin<Box<dyn Future<Output = ()>>>
  - Pinned future for safe async execution
  - Boxed for heap allocation
  - Sized for task queue storage

YieldNow Future:
• Cooperative multitasking primitive
• yielded: bool flag
• First poll returns Pending, second returns Ready
• Used for task yielding in async contexts

yield_now() Function:
• Creates YieldNow future
• Enables cooperative task switching
• Simple API for async code
```

### 7. Task Executor (`src/executor.rs`)
**Role**: Async task scheduler and runtime

```
Executor Structure:
• task_queue: VecDeque<Task>
  - Double-ended queue for efficient push/pop
  - FIFO ordering for round-robin scheduling
  - Dynamic task storage

Methods:
• new() -> Self
  - Creates empty executor instance
  - Initializes task queue

• spawn(&mut self, task: Task)
  - Adds task to end of queue
  - Task: Future wrapped in Task struct
  - Non-blocking operation

• run(&mut self) -> !
  - Main executor loop
  - Calls run_ready_tasks() repeatedly
  - Uses hlt() for power efficiency
  - Never returns (infinite loop)

• run_ready_tasks(&mut self)
  - Polls all ready tasks
  - Processes tasks in queue order
  - Maintains remaining_tasks counter
  - Re-queues pending tasks

Internal Functions:
• dummy_waker() -> Waker
  - Creates no-op waker for polling
  - RawWaker with minimal VTable
  - Required by Context::from_waker()

Waker Implementation:
• clone: Duplicates RawWaker
• no_op: Empty wake function
• VTable: Static RawWakerVTable
```

### 8. VGA Buffer (`src/vga_buffer.rs`)
**Role**: Text display on VGA screen

```
Specifications:
• Address: 0xB8000
• Size: 80 × 25 = 2000 characters
• Attributes: 1 color byte + 1 character byte

Features:
• 16 ANSI colors (Black → White)
• Automatic scroll with line preservation
• Smart backspace (wraps to previous line)
• Hardware cursor update
• Color-coded output support
```

### 9. PS/2 Keyboard (`src/drivers/keyboard.rs`)
**Role**: Translates scancodes to characters and shell command handling

```
Configuration:
• Layout: French AZERTY
• Scancode Set: 2 (IBM standard)
• Control: Ignore Ctrl (for testing)
• Command Buffer: 256 character capacity

Shell Commands:
• help     - Show available commands
• info     - Display system information
• whoami   - Display current user
• echo     - Print text to screen
• ls       - List files in RAMFS
• touch    - Create new file
• cat      - Read file content
• rm       - Delete file
• edit     - Modify existing file
• stats    - Show filesystem statistics
• neofetch - Display system info (ASCII art)
• clear    - Clear screen
• Esc      - Clear buffer + reset screen

Handled Keys:
• Letters a-z, A-Z (AZERTY layout)
• Digits 0-9
• French accented characters (è, é, ê, ë)
• Special characters (, ; : !)
• Enter, Backspace, Escape
```

### 10. Serial Port (`src/serial.rs`)
**Role**: Debugging via serial connection

```
Configuration:
• Port: COM1 (0x3F8)
• UART: 16550 standard
• Output: stdout during QEMU debugging

Usage:
• Boot log: "[JC-OS] Kernel starting..."
• System log: "[GDT] Loaded", "[IDT] Loaded"
• Memory stats: "Heap Allocator Ready"
• Panic display
• Serial print for debugging
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

## ⌨️ Shell Commands

### File Management

| Command | Description | Usage |
|---------|-------------|-------|
| `touch` | Create new file | `touch <filename> <content>` |
| `cat` | Read file content | `cat <filename>` |
| `rm` | Delete file | `rm <filename>` |
| `edit` | Modify file | `edit <filename> <new_content>` |
| `ls` | List all files | `ls` |

### System Information

| Command | Description | Output Example |
|---------|-------------|----------------|
| `info` | Display system info | JC-OS v0.2 - Andre Edition |
| `whoami` | Display current user | Andre |
| `stats` | Show filesystem stats | Files: 3, Memory: 256 bytes |
| `neofetch` | ASCII system info | Art + system details |

### Utilities

| Command | Description |
|---------|-------------|
| `help` | Show available commands |
| `echo` | Print text to screen |
| `clear` | Clear the screen |
| `Enter` | Execute command |
| `Backspace` | Delete previous character |
| `Esc` | Clear buffer + reset screen |

## 🔍 Example Session

```
qemu-system-x86_64 -drive format=raw,file=target/x86_64-jc-os/debug/bootimage-jc-os.bin -serial stdio

[JC-OS] Kernel starting...
[GDT] Loaded
[IDT] Interrupt Descriptor Table loaded
[PIC] Initialized - Timer and Keyboard enabled
[PS/2] Keyboard controller initialized
[KEYBOARD] Driver initialized (AZERTY layout, Set2)
[PAGING] 4-Level page tables initialized
[FRAMES] Boot info frame allocator ready
[SYSTEM] Heap Allocator Ready
[FS] RAM File System initialized
[EXECUTOR] Task scheduler ready

╔═══════════════════════════════════════════════════════════════════════╗
║           JC-OS - BARE METAL KERNEL v0.2 - RUST                       ║
╚═══════════════════════════════════════════════════════════════════════╝

Digital Sovereignty System
File System: READY (RAMFS) | Commands examples: touch, ls, cat, rm, edit
Task Scheduling: READY | Async/Await supported

>>> help
Commands: help, info, stats, echo, whoami, ls, touch, cat, rm, edit, clear, neofetch

>>> touch hello.txt "Hello JC-OS!"
File 'hello.txt' saved to RAM.

>>> touch test.txt "This is a test"
File 'test.txt' saved to RAM.

>>> ls
- hello.txt
- test.txt

>>> cat hello.txt
Hello JC-OS!

>>> stats
--- SYSTEM STATS ---
Files stored : 2
Used Memory  : 21 bytes
Heap Size    : 100 KB
Buffer Cap   : 256 chars

>>> neofetch
  _/_/   JC-OS v0.2
 _/      Kernel: Rust 64-bit
_/_/_/   User: Andre

>>> whoami
Andre

>>> clear

>>> 
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
| `alloc` | - | Dynamic memory allocation |

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

### Keyboard not responding
Check AZERTY layout mapping or try with US QWERTY layout.

### File system commands not working
Ensure RAMFS is initialized: check boot log for "[FS] RAM File System initialized"

### Async tasks not running
Verify executor is initialized and run() is called in main loop

## 🔮 Future Improvements

- [ ] **PS/2 Mouse Driver** - On-screen cursor tracking and click events
- [ ] **Page Fault Handler** - Better memory error reporting and debugging
- [ ] **Kernel Heap Expansion** - Dynamic heap growth based on demand
- [ ] **Persistent Storage** - Disk driver with FAT32 reading/writing
- [ ] **Advanced Shell** - Tab completion, command history, environment variables
- [ ] **Preemptive Scheduling** - Timer-based task switching
- [ ] **Multiple Executors** - Multi-core task distribution
- [ ] **Task Priorities** - Priority-based task scheduling
- [ ] **Inter-Task Communication** - Channels, signals, and message passing
- [ ] **Virtual File System** - VFS abstraction layer for multiple file systems
- [ ] **Process Management** - Process creation, termination, and IPC
- [ ] **System Calls** - User-mode to kernel-mode transitions
- [ ] **Memory Protection** - User/kernel memory isolation
- [ ] **Network Support** - Network card driver and basic networking
- [ ] **GUI Subsystem** - Window manager and basic graphics

## 📄 License

This project is licensed under Apache 2.0.

## 🤝 Contributions

Issues and pull requests are welcome to improve the project!

---

**JC-OS v0.2 - Andre Edition**  
A minimalist bare-metal operating system written in Rust

