use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}

// --- CORRECTION POUR L'IA ---

/// Lit un seul octet sur le port série (bloquant)
pub fn read_byte() -> u8 {
    SERIAL1.lock().receive() // .receive() attend déjà que la donnée soit prête
}

/// Lit une ligne complète
/// Lit une ligne complète en ignorant les caractères de contrôle initiaux
pub fn read_line() -> alloc::string::String {
    let mut s = alloc::string::String::new();
    loop {
        let b = read_byte();
        
        // On ignore les retours à la ligne s'ils sont au tout début (nettoyage de buffer)
        if (b == b'\n' || b == b'\r') && s.is_empty() {
            continue;
        }
        
        // Fin de la réponse
        if b == b'\n' || b == b'\r' {
            break;
        }

        // On ne garde que les caractères imprimables pour éviter les glitchs VGA
        if b >= 32 && b <= 126 {
            s.push(b as char);
        }
    }
    s
}