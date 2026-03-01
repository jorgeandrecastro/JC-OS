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
- Hierarchical file system with directories and permissions
- Interactive shell with real-time clock display
- Async/await task scheduling with executor
- User management with UID system
- Automatic timezone support (Europe/France)
- PS/2 mouse driver (in development)

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
- **Hierarchical RAM File System** - Multi-level directory structure
- **Inode-based design** with UID and permissions
- **Current Working Directory (CWD)** navigation
- **Directory operations**: look, open, room (create directory)
- **File operations**: touch, cat, read, edit, note, drop
- **Path navigation**: absolute and relative paths
- **Automatic home directory creation** for new users
- **BTreeMap-based organization** for efficient lookup
- **Statistics tracking**: file count and total size
- **Unicode support** via UTF-8 lossless conversion

### Interactive Shell
- **Command interpreter** with multiple built-in commands
- **File management commands**: touch, cat, rm, edit
- **System information**: info, stats, whoami, neofetch, date
- **Utility commands**: help, echo, clear, ls
- **AI Assistant**: `ia` command for interactive AI queries via local LLM
- **Secure login system** with authentication

### User Authentication & Management
- **Role-based access control** with Admin and Standard roles
- **User management** with login/logout functionality
- **Dynamic user creation** with `useradd` command (Admin only)
- **User deletion** with `userdel` command (Admin only)
- **UID system** for user identification
- **Session tracking** with current user identification
- **Password authentication** with credential validation
- **Automatic home directory creation** for new users
- **Default admin account**: username "andre", password "admin123"

### Real-Time Clock (RTC)
- **CMOS RTC access** via ports 0x70/0x71
- **BCD to decimal conversion** for accurate time reading
- **Time struct** with hours, minutes, seconds
- **Automatic timezone adjustment** for France (UTC+1/+2)
- **Daylight Saving Time (DST)** support with European rules
- **Non-volatile time keeping** independent of system power

### PS/2 Mouse Driver
- **PS/2 mouse interface** via ports 0x60/0x64
- **3-byte packet protocol** for movement and button data
- **Movement delta calculation** with sign extension
- **Cursor position tracking** with screen boundary clamping
- **Mouse state management** with phase-based packet decoding
- **Auxiliary port enablement** for mouse device
- **Data reporting activation** for real-time input

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
- **IDT** (Interrupt Descriptor Table) - Interrupt vectors with exception handlers
- **PIC 8259** - Programmable Interrupt Controller
- **Double Fault Handler** with proper IST stack protection
- **Page Fault Handler** with detailed error reporting
- **General Protection Fault Handler** for CPU violations
- **Invalid Opcode Handler** for instruction errors
- **Timer Interrupt** - Hardware timer (IRQ0) for real-time clock display
- **Keyboard Interrupt** - PS/2 keyboard input handling
- **Mouse Interrupt** - PS/2 mouse input handling (IRQ12)

### User Mode & Syscalls (Ring 3)
- **Privilege Level Switching** - Kernel Ring 0 ↔ User Ring 3
- **Syscall Interface** - SYSCALL/SYSRET AMD64 mechanism
- **User Stack Isolation** - Separate stack for user programs
- **GS_BASE Register Setup** - Kernel data structure access from user mode
- **IA32_KERNEL_GS_BASE MSR** - Proper swapgs initialization
- **System Call Handler** - Kernel-side syscall entry point
- **Argument Marshalling** - System V AMD64 ABI compliance
- **Page Accessibility** - User-accessible kernel memory for syscall operations
- **User Program Support** - Ring 3 test programs with syscalls
- **SYS_PRINT** - Print to serial from user mode
- **SYS_EXIT** - Clean program termination from user mode

### Memory Management
- **x86_64 Paging** (4-level page tables)
- **Physical memory mapping** via bootloader info
- **Frame allocator** using UEFI memory map
- **Heap allocation** (100 KiB) with linked-list allocator
- **Virtual to physical address translation**
- **Page-level protection** (PRESENT, WRITABLE, USER_ACCESSIBLE flags)
- **Error handling** with Result type and alloc_error_handler
- **User-Accessible Memory** - Kernel structures marked for user mode access
- **Safe Page Table Traversal** - Presence checks prevent cascading page faults
- **Force User Access** - Hierarchical page table permission unlocking

### Debugging
- **COM1 serial output** via UART 16550
- **Complete boot logging**
- **Memory statistics** display (heap start, size, status)
- **Panic error display** via serial
- **Interrupt event logging**

## 🏗️ System Architecture

```
┌─────────────────────────────────────────────────┐
│              JC-OS Kernel v0.4                   │
│              Andre Edition                       │
├─────────────────────────────────────────────────┤
│  Entry Point: kernel_main()                     │
├─────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────┐  │
│  │           Initialization Order            │  │
│  ├───────────────────────────────────────────┤  │
│  │  1. GDT + TSS     (CPU segmentation)      │  │
│  │  2. IDT           (interrupt table)       │  │
│  │  3. PIC           (interrupt controller)  │  │
│  │  4. PS/2 Controller (keyboard+mouse)     │  │
│  │  5. Keyboard Driver (AZERTY Set2)        │  │
│  │  6. Paging Setup  (4-level page tables)  │  │
│  │  7. Frame Allocator (memory map parsing) │  │
│  │  8. Heap Init      (100 KiB allocator)   │  │
│  │  9. File System    (Hierarchical RAMFS)  │  │
│  │  10. Auth System    (user management)    │  │
│  │  11. RTC Driver     (time+timezone)     │  │
│  │  12. Mouse Driver   (PS/2 input)        │  │
│  │  13. Task System    (Executor init)     │  │
│  │  14. Interrupts enabled                  │  │
│  │  15. UI Launch     (shell prompt)        │  │
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
│  Authentication & User Management               │
│  ┌───────────────────────────────────────────┐  │
│  │  AuthManager                               │  │
│  │  ├── users: Vec<User>                     │  │
│  │  ├── current_user: Option<User>           │  │
│  │  ├── next_uid: u32                        │  │
│  │  ├── login(username, password) -> bool   │  │
│  │  ├── logout()                             │  │
│  │  ├── add_user(username, pass) -> uid      │  │
│  │  ├── delete_user(username) -> Result      │  │
│  │  └── get_current_uid() -> u32             │  │
│  │                                            │  │
│  │  User                                      │  │
│  │  ├── username: String                     │  │
│  │  ├── password_hash: String                │  │
│  │  ├── role: Role (Admin/Standard)          │  │
│  │  └── uid: u32                             │  │
│  │                                            │  │
│  │  Role Enum                                │  │
│  │  ├── Admin     - Full system access       │  │
│  │  └── Standard  - Limited access           │  │
│  └───────────────────────────────────────────┘  │
├─────────────────────────────────────────────────┤
│  Hierarchical File System Architecture          │
│  ┌───────────────────────────────────────────┐  │
│  │  RamFileSystem                            │  │
│  │  ├── root: Directory                      │  │
│  │  ├── cwd: Vec<String>                     │  │
│  │  ├── look() -> Vec<(name, type)>         │  │
│  │  ├── open(path) -> Result                 │  │
│  │  ├── room(name, uid) -> Result            │  │
│  │  ├── write_file(name, content, uid)       │  │
│  │  └── read_file(name) -> Option<String>    │  │
│  │                                            │  │
│  │  Directory                                 │  │
│  │  ├── inode: Inode                         │  │
│  │  └── entries: BTreeMap<String, FsNode>    │  │
│  │                                            │  │
│  │  Inode                                     │  │
│  │  ├── uid: u32                             │  │
│  │  ├── permissions: u16                      │  │
│  │  └── node_type: File/Directory            │  │
│  │                                            │  │
│  │  FsNode Variants                          │  │
│  │  ├── File(File)                           │  │
│  │  └── Directory(Directory)                 │  │
│  └───────────────────────────────────────────┘  │
├─────────────────────────────────────────────────┤
│  Managed Peripherals:                           │
│  • VGA 0xB8000  - Text screen                  │
│  • COM1 0x3F8   - Serial port                  │
│  • PIC 0x20/0xA0 - Interrupt controller        │
│  • PS/2 0x60/0x64 - Keyboard + Mouse           │
│  • PIT 0x40     - Programmable Interval Timer  │
│  • RTC 0x70/0x71 - Real Time Clock (CMOS)      │
└─────────────────────────────────────────────────┘
```

## 📁 Project Structure

```
jc-os/
├── Cargo.toml                    # Rust project configuration
├── Readme.md                     # This file
├── x86_64-jc-os.json             # Custom target spec
├── tools/
│   └── ai_bridge.py              # AI Bridge server (LLM integration)
├── src/
│   ├── main.rs                   # Entry point + initialization
│   ├── gdt.rs                    # GDT + TSS (segmentation)
│   ├── interrupts.rs             # IDT + PIC handling + handlers
│   ├── vga_buffer.rs             # Color VGA text driver
│   ├── serial.rs                 # COM1 serial output (UART 16550)
│   ├── memory.rs                 # Paging + frame allocator
│   ├── allocator.rs              # Heap allocator (linked-list)
│   ├── fs.rs                     # RAM File System (RAMFS)
│   ├── auth.rs                   # User authentication system
│   ├── task.rs                   # Task structures + async support
│   ├── executor.rs               # Task executor + scheduler
│   ├── shell.rs                  # Interactive shell with login
│   └── drivers/
│       ├── mod.rs                # Drivers module (export)
│       ├── keyboard.rs           # PS/2 AZERTY keyboard driver + shell
│       ├── mouse.rs              # PS/2 mouse driver (in development)
│       └── rtc.rs                # Real Time Clock driver
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

### 5. Hierarchical RAM File System (`src/fs.rs`)
**Role**: Hierarchical in-memory file storage with directories and permissions

```
Structure:
• Inode: uid, permissions, node_type (File/Directory)
• File: inode + data (Vec<u8>)
• Directory: inode + entries (BTreeMap<String, FsNode>)
• RamFileSystem: root Directory + cwd (current working directory)
• Global instance protected by Mutex

FsNode Enum:
• File(File) - Regular file with content
• Directory(Directory) - Container for other nodes

Features:
• Hierarchical Structure:
  - Root directory "/" as entry point
  - Current Working Directory (CWD) navigation
  - open(path) - Navigate to directory
  - look() - List current directory contents
  - room(name, uid) - Create new directory

• File Operations:
  - write_file(name, content, uid) - Create/overwrite with UID tracking
  - read_file(name) - Read file as String (returns Option)
  - remove_file(name) - Delete file/directory (returns bool)
  - get_stats() - Returns (file_count, total_bytes)

• Path Navigation:
  - "/" - Return to root
  - ".." - Go up one level
  - name - Enter subdirectory

• Security:
  - UID tracking for file ownership
  - Permission flags (0o644 for files, 0o755 for directories)
  - Home directory auto-creation for new users

Storage:
• In-memory only (volatile)
• Unicode support via UTF-8 lossless conversion
• No persistence (data lost on reboot)
```

### 6. User Authentication & Management (`src/auth.rs`)
**Role**: User management, authentication, and access control

```
Structure:
• Role Enum: Admin, Standard
• User: username, password_hash, role, uid
• AuthManager: users Vec, current_user Option, next_uid u32

Features:
• login(username, password) -> bool
  - Authenticates user credentials
  - Case-insensitive username matching
  - Returns true on successful authentication
  - Sets current_user session

• logout()
  - Clears current user session
  - Sets current_user to None

• add_user(username, password) -> Result<u32, &str>
  - Creates new user with Standard role
  - Assigns unique UID (starting from 1000)
  - Prevents duplicate usernames
  - Returns new UID on success

• delete_user(username) -> Result<(), &str>
  - Removes user from system
  - Protects primary admin account
  - Prevents deleting current user
  - Returns error if user not found

• get_current_username() -> String
  - Returns current username or "Guest" if not logged in

• get_current_uid() -> u32
  - Returns current user's UID
  - Returns 1000 for Guest

• Role-based access control
  - Admin: Full system access, user management
  - Standard: Limited permissions

Default User:
• Username: "andre"
• Password: "admin123"
• Role: Admin
• UID: 0

Security Features:
• Password masking during input
• Session management
• Credential validation
• Case-insensitive username matching
• Admin-only user management operations

Lazy Static Initialization:
• AUTH: Mutex<AuthManager> for thread-safe access
• Automatically initialized at kernel startup
```

### 7. Real-Time Clock (`src/drivers/rtc.rs`)
**Role**: CMOS RTC access for time keeping with automatic timezone

```
Hardware Interface:
• Address Port: 0x70 (write register index)
• Data Port: 0x71 (read/write data)
• BCD Format: Binary Coded Decimal

RtcTime Structure:
• seconds: u8 (0-59)
• minutes: u8 (0-59)
• hours: u8 (0-23)

Functions:
• read_rtc_register(reg: u8) -> u8
  - Writes register index to port 0x70
  - Reads data from port 0x71
  - Returns raw BCD value

• get_time() -> RtcTime
  - Reads registers 0x00 (seconds), 0x02 (minutes), 0x04 (hours)
  - Reads date registers 0x07 (day), 0x08 (month), 0x09 (year)
  - Converts BCD to decimal
  - Applies timezone adjustment (France UTC+1/+2)
  - Returns RtcTime struct with corrected time

BCD Conversion:
• BCD = (value & 0x0F) + ((value / 16) * 10)
• Extracts low nibble and high nibble
• Combines for correct decimal value

Timezone Support:
• Automatic adjustment for France timezone
• Summer time (DST): UTC+2 (March-October)
• Winter time: UTC+1 (November-February)
• DST calculated using European rules (last Sunday of March/October)

Features:
• Battery-backed time keeping (independent of power)
• Standard CMOS RTC chip compatible
• 24-hour format support
• No interrupts required for reading
• Real-time clock display in shell (updated every second)
```

### 8. PS/2 Mouse Driver (`src/drivers/mouse.rs`)
**Role**: PS/2 mouse input handling for cursor tracking

```
Hardware Interface:
• Command Port: 0x64 (PS/2 controller)
• Data Port: 0x60 (keyboard/mouse data)
• Auxiliary Port: Enabled via command 0xA8

MouseState Structure:
• phase: u8 - Packet decoding phase (0-2)
• buffer: [u8; 3] - Raw packet data
• x, y: i32 - Current cursor position
• old_x, old_y: i32 - Previous position for rendering

Packet Protocol (3 bytes):
• Byte 0: Flags (bit 0=Left, 1=Right, 2=Middle, 3=Always 1, 4=X sign, 5=Y sign, 6=X overflow, 7=Y overflow)
• Byte 1: X movement delta (signed)
• Byte 2: Y movement delta (signed)

Functions:
• init() - Initialize mouse controller
  - Enables auxiliary port
  - Configures interrupt enable
  - Sets bit default mouse parameters
  - Enables data reporting

• add_mouse_data(data: u8) - Process incoming mouse data
  - Phase-based packet decoding
  - Movement delta calculation with sign extension
  - Position clamping to screen bounds (0-79 for X, 0-24 for Y)
  - Cursor rendering

• draw_cursor(x, y, old_x, old_y) - Render mouse cursor
  - Tracks cursor position changes
  - Prepares for visual cursor display

Features:
• 3-byte packet protocol standard
• Movement delta with sign extension
• Screen boundary clamping
• Button state tracking (left, right, middle)
• Real-time position updates
• Auxiliary port communication
• Data reporting enable/disable

Status:
• Driver initialized and functional
• Cursor position tracking implemented
• Visual cursor rendering prepared
```

### 9. Task Management (`src/task.rs`)
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

### 9. Task Executor (`src/executor.rs`)
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

### 10. VGA Buffer (`src/vga_buffer.rs`)
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

### 11. PS/2 Keyboard (`src/drivers/keyboard.rs`)
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
• date     - Display current time from RTC
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

### 12. Interactive Shell (`src/shell.rs`)
**Role**: Command interpreter with authentication

```
Shell Features:
• Secure login system on boot
• Password masking for sensitive input
• Command history and buffer management
• Multi-line command support
• Color-coded prompt with user info

Login System:
• Requires authentication before command access
• Username and password prompts
• Credential validation via AuthManager
• Session persistence until logout

Prompt Format:
• Shows current username and hostname
• Visual indicator of authentication status
• Example: "andre@jc-os:~$ "

Session Management:
• Automatic login requirement
• Session tracking with AuthManager
• User identification for commands
• Future: Multiple user sessions

Command Buffer:
• 256 character capacity
• Backspace with visual feedback
• Escape key to clear and reset
• Support for long commands with wrapping
```

### 13. Serial Port (`src/serial.rs`)
**Role**: Debugging via serial connection + AI Bridge communication

```
Configuration:
• Port: COM1 (0x3F8)
• UART: 16550 standard
• Output: stdout during QEMU debugging
• AI Bridge: Bidirectional communication on port 1234

Features:
• Boot log: "[JC-OS] Kernel starting..."
• System log: "[GDT] Loaded", "[IDT] Loaded"
• Memory stats: "Heap Allocator Ready"
• Panic display
• Serial print for debugging
• AI communication: read_line() for receiving LLM responses
• AI requests: serial_println!("AI_REQ:{}", query) for sending queries
```

### 14. AI Bridge (`tools/ai_bridge.py`)
**Role**: Local LLM integration for JC-OS AI assistant

```
Configuration:
• Script: tools/ai_bridge.py
• Model: SmolLM2-135M-Instruct-Q8_0.gguf
• Host: 127.0.0.1
• Port: 1234
• Protocol: TCP socket communication

Features:
• Socket server listening on port 1234
• Receives AI queries via serial from kernel
• Uses llama_cpp for local LLM inference
• Sends responses back via serial connection
• Text cleaning for VGA display (ASCII only)
• Temperature: 0.1 for stable responses

Communication Protocol:
1. Kernel sends: serial_println!("AI_REQ:{query}")
2. Bridge receives query and processes with LLM
3. Bridge sends response with trailing \n
4. Kernel reads response via serial::read_line()

Usage:
```bash
# Start the AI bridge before running JC-OS
python tools/ai_bridge.py

# In JC-OS shell:
ia what is a kernel?
[JC-AI]: A kernel is the core of an operating system...
```
```

### 15. Syscalls & User Mode (`src/syscalls.rs`, `src/user.rs`)
**Role**: User mode program execution with system call interface

```
Architecture:
• SYSCALL/SYSRET mechanism (AMD64)
• Privilege level switching (Ring 0 ↔ Ring 3)
• GS_BASE registers for kernel data access
• Separate user and kernel stacks

Kernel Setup:
• Initialization in kernel_main()
1. init_gs_base() - Set GS/IA32_KERNEL_GS_BASE MSR
2. syscalls::init() - EFER, LSTAR, STAR, SFMask configuration
3. make_gs_accessible_from_user() - Mark KERNEL_DATA readable
4. set_kernel_stack() - Initialize privilege stack

Syscall Entry Point (Assembly):
• swapgs - Swap user/kernel GS
• Stack swapping - Kernel stack activation
• Argument remapping - System V ABI compliance
  - User: rax=ID, rdi=arg1, rsi=arg2
  - Kernel: rdi=ID, rsi=arg1, rdx=arg2
• Call C function syscall_handler()
• Context restoration on sysret

System Calls:
• SYS_PRINT (ID=1): Print string to serial
  - Arguments: rdi=pointer, rsi=length
  - Output: "[RING 3] <message>"
• SYS_EXIT (ID=0): Terminate user program
  - Arguments: rdi=exit_code
  - Result: Kernel halt loop

User Mode Execution:
• jump_to_user() - iretq transition to Ring 3
• Stack frame: SS, RSP, RFLAGS, CS, RIP
• IF bit set in RFLAGS for user interrupts
• Code and stack pages marked USER_ACCESSIBLE

Memory Access:
• KERNEL_DATA (16 bytes) - Kernel stack pointers
  - Offset 0x00: kernel_stack (RSP during syscall)
  - Offset 0x08: user_stack (saved user RSP)
• Force unlocking 10 code pages for .text/.rodata
• Force unlocking 5 stack pages for user stack

Example User Program:
```rust
#[no_mangle]
pub extern "C" fn user_test_program() -> ! {
    sys_print("Hello from Ring 3!");
    sys_print("JC-OS user mode is working.");
    sys_exit(0);
}

fn sys_print(s: &str) {
    unsafe { asm!("syscall", in("rax") 1, in("rdi") s.as_ptr(), in("rsi") s.len()); }
}

fn sys_exit(code: u64) -> ! {
    unsafe { asm!("syscall", in("rax") 0, in("rdi") code, options(noreturn)); }
}
```

Testing:
```
JC-OS Shell:
andre@jc-os:/$ run
--- Preparing Ring 3 (Hierarchical Access) ---
[DEBUG] Code address: 0x2349e0
[DEBUG] Stack start:  0x284000
--- Jumping to Ring 3 ---
[SYSCALL] Handler: ID=1, ARG1=0xAAAA, ARG2=14
[RING 3] Hello from Ring 3!
[RING 3] JC-OS user mode is working.
[SYSCALL] SYS_EXIT (code=0)
[SYSTEM] User program exited with code 0
```
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

## 🔐 Login Credentials

By default, JC-OS v0.4 includes a secure login system:

```
Username: andre
Password: admin123
Role: Admin
UID: 0
```

**Note**: The first time you run JC-OS v0.4, you will be presented with a login screen. Use the default credentials above to access the shell. Administrators can create new users using the `useradd` command.

## ⌨️ Shell Commands

### Authentication

| Command | Description |
|---------|-------------|
| (Login) | Enter username and password at startup |
| whoami  | Display current authenticated user |
| logout  | End current session |
| useradd | Create new user (Admin only) |
| userdel | Delete user (Admin only) |

### File Management

| Command | Description | Usage |
|---------|-------------|-------|
| `look` | List directory contents | `look` |
| `open` | Change directory | `open <directory>` |
| `room` | Create directory | `room <name>` |
| `where` | Show current path | `where` |
| `note` | Create file with content | `note <filename> <content>` |
| `read` | Read file content | `read <filename>` |
| `drop` | Delete file/directory | `drop <filename>` |
| `edit` | Modify file | `edit <filename> <new_content>` |
| `type` | Interactive file editor | `type <filename>` |

### Navigation

| Command | Description |
|---------|-------------|
| `/` | Go to root directory |
| `..` | Go up one directory level |
| `directory_name` | Enter subdirectory |

### System Information

| Command | Description | Output Example |
|---------|-------------|----------------|
| `info` | Display system info | JC-OS v0.4 - Andre Edition |
| `whoami` | Display current user | andre |
| `date` | Display current time (timezone adjusted) | Time: 14:30:45 |
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
| `ia` | Ask AI assistant (requires AI bridge) |

### Editor Shortcuts (type command)

| Shortcut | Description |
|----------|-------------|
| Ctrl+S | Save file |
| Ctrl+Q | Quit editor |

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
[FS] RAM File System initialized (Hierarchical)
[AUTH] Authentication system initialized
[RTC] Real Time Clock initialized (Timezone: Europe/Paris)
[MOUSE] PS/2 Mouse driver initialized
[EXECUTOR] Task scheduler ready

 JC-OS - BARE METAL KERNEL v0.4 - RUST EDITION

--- LOGIN REQUIRED ---
Username: andre
Password: ********
Welcome back, andre!

Digital Sovereignty System
File System: READY (Hierarchical RAMFS) | Try: look, open, room, where
Task Scheduling: READY | Async/Await supported
Authentication: ENABLED | Session active

andre@jc-os:/$ help
Commands: help, info, stats, echo, whoami, look, open, room, where, note, read, drop, type, useradd, userdel, date, neofetch, ia

andre@jc-os:/$ date
Time: 14:30:45 (UTC+2, Summer Time)

andre@jc-os:/$ ia what is JC-OS?
[JC-AI]: JC-OS is a minimalist bare-metal operating system kernel written in Rust.

andre@jc-os:/$ room home
Directory 'home' created.

andre@jc-os:/$ open home
andre@jc-os:/home$

andre@jc-os:/home$ room andre
Directory 'andre' created.

andre@jc-os:/home$ note welcome.txt "Welcome to JC-OS!"
File 'welcome.txt' created.

andre@jc-os:/home$ look
andre/
welcome.txt

andre@jc-os:/home$ where
/home

andre@jc-os:/home$ open andre
andre@jc-os:/home/andre$

andre@jc-os:/home/andre$ type test.txt
[TYPE: test.txt] (Ctrl+S to save, Ctrl+Q to exit)
Hello from interactive editor!

andre@jc-os:/home/andre$ useradd john secret123
[AUTH] User 'john' created with UID 1000.
[FS] Home directory /home/john created.

andre@jc-os:/home/andre$ stats
--- SYSTEM STATS ---
Files/Folders : 2
Used Space    : 25 bytes

andre@jc-os:/home/andre$ neofetch
  _/_/   JC-OS v0.4 - Rust Edition
 _/      User : andre
_/_/_/    FS   : Hierarchical RAMFS
           Time : 14:30:45

andre@jc-os:/home/andre$ where
/home/andre

andre@jc-os:/home/andre$ logout

--- LOGIN REQUIRED ---
Username: andre
Password: ********
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
| `crossbeam-queue` | 0.3.12 | Lock-free queue for task scheduling |
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

### RTC time showing incorrect values
- Verify RTC is properly initialized in QEMU
- Check CMOS battery status (virtual in QEMU)
- Ensure BCD conversion is working correctly

### Authentication login fails
- Verify credentials: username "andre", password "admin123"
- Check that AUTH system initialized in boot log
- Ensure passwords are case-sensitive for username matching
- Try resetting credentials if persistent storage available

### Async tasks not running
Verify executor is initialized and run() is called in main loop

## � Bug Fixes & Critical Corrections

### Double Fault Prevention (v0.4.1)

**Issue**: CPU triggering double faults during exception handling

**Root Cause**: 
- Page table entry access without presence validation
- Missing exception handlers in IDT  
- Cascading page faults during exception handling

**Solutions Implemented**:
1. **Safe Page Table Access**: Added `PageTableFlags::PRESENT` checks before dereferencing frames
2. **Comprehensive Exception Handlers**:
   - Page Fault Handler - Logs faulting address with detailed error info
   - General Protection Fault Handler - CPU violation reporting
   - Invalid Opcode Handler - Instruction error capture
   - Breakpoint Handler - Debug support
3. **IST Stack for Double Faults** - Dedicated stack prevents recursive failures
4. **Graceful Error Handling** - Returns `false` instead of panicking on mapping failures

### Syscall Implementation & Argument Passing (v0.4.1)

**Issue**: Garbled arguments when transitioning to user mode syscalls

**Root Causes**:
1. Syscall entry point not rearranging arguments per System V AMD64 ABI
2. GS_BASE register not initialized for user-accessible kernel data
3. IA32_KERNEL_GS_BASE MSR pointing to undefined memory
4. Incorrect iretq stack frame order for Ring 3 transition

**Solutions Implemented**:
1. **Argument Remapping** (`src/syscalls.rs` lines 23-25):
   ```asm
   mov rdx, rsi          # arg2 (was in rsi)
   mov rsi, rdi          # arg1 (was in rdi)  
   mov rdi, rax          # ID (was in rax)
   ```

2. **Dual GS_BASE Initialization**:
   - GS_BASE → User mode kernel data access
   - IA32_KERNEL_GS_BASE MSR → Kernel mode mirror
   - Both point to KERNEL_DATA structure (16 bytes)

3. **User-Accessible Memory Marking**:
   - `make_gs_accessible_from_user()` calls `force_user_access()`
   - Ensures syscall handler can write to gs:[0x00] and gs:[0x08]
   - Called after memory initialization completes

4. **Corrected iretq Stack Frame**:
   - Proper ordering: SS → RSP → RFLAGS → CS → RIP
   - IF bit set to enable interrupts in user mode
   - Correct privilege level transition (Ring 0 → Ring 3)

### Memory Protection Fixes (v0.4.1)

**Issue**: User programs accessing unmapped kernel memory causing page faults

**Solutions**:
1. **Multi-Page Unlocking**: Unlock 10 pages around user code for .text and .rodata
2. **Stack Protection**: Unlock full 16KB user stack (4-5 pages)
3. **Bounds Checking**: Validate pages exist before attempting access
4. **Return Status**: `force_user_access()` returns `bool` for success/failure
5. **Graceful Degradation**: Continue on partial failures, report to shell

- [ ] **File Permissions** - Permission enforcement per UID
## 🔮 Future Improvements

- [x] **System Calls** - User-mode to kernel-mode transitions (IMPLEMENTED v0.4.1)
- [x] **Page Fault Handler** - Memory error reporting and debugging (IMPLEMENTED v0.4.1)
- [x] **Exception Handlers** - Complete IDT exception coverage (IMPLEMENTED v0.4.1)
- [ ] **Mouse Cursor Rendering** - Visual cursor with click event handling
- [ ] **Kernel Heap Expansion** - Dynamic heap growth based on demand
- [ ] **Persistent Storage** - Disk driver with FAT32 reading/writing
- [ ] **Advanced Shell** - Tab completion, command history, environment variables
- [ ] **Preemptive Scheduling** - Timer-based task switching instead of cooperative
- [ ] **Multiple Executors** - Multi-core task distribution
- [ ] **Task Priorities** - Priority-based task scheduling with priority queues
- [ ] **Inter-Task Communication** - Channels, signals, and message passing
- [ ] **Virtual File System** - VFS abstraction layer for multiple file systems
- [ ] **Process Management** - Complex process creation, termination, and IPC
- [ ] **Additional Syscalls** - Read, write, fork, exec, wait beyond basic I/O
- [ ] **Memory Protection** - Stricter user/kernel memory isolation
- [ ] **Enhanced Authentication** - Password hashing with bcrypt/argon2
- [ ] **Multi-User Sessions** - Multiple concurrent user sessions/terminals
- [ ] **Audit Logging** - Authentication logs, command history tracking
- [ ] **Network Support** - Network card driver and basic networking
- [ ] **GUI Subsystem** - Window manager and basic graphics rendering
- [ ] **File Permissions** - Permission enforcement per UID/GID
- [ ] **Signal Handling** - POSIX signal delivery to user programs


## 🔒 Security Features

### Current Implementation
- **User Authentication**: Login required before shell access
- **Role-Based Access**: Admin vs Standard user roles
- **User Management**: Admin-only user creation and deletion
- **UID Tracking**: Unique user identification system
- **Session Management**: Track current authenticated user
- **Password Masking**: Hide password input during login
- **Credential Validation**: Case-insensitive username matching
- **Home Directory Isolation**: Each user gets personal directory

### Planned Security Enhancements
- **Password Hashing**: Replace plain-text password storage with bcrypt/argon2
- **Multi-Factor Authentication**: Additional verification methods
- **Session Timeout**: Automatic logout after inactivity
- **Account Lockout**: Brute-force protection
- **Audit Trail**: Log all authentication attempts and privileged actions
- **Secure Boot**: Verify kernel integrity at startup
- **User Isolation**: Separate memory spaces per user
- **Permission System**: File and command access control enforcement

## 📄 License

This project is licensed under Apache 2.0.

## 🤝 Contributions

Issues and pull requests are welcome to improve the project!

---

**JC-OS v0.4 - Andre Edition**  
A minimalist bare-metal operating system written in Rust
