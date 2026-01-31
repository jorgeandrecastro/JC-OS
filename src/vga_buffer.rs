use volatile::Volatile;
use core::fmt;
use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0, Blue = 1, Green = 2, Cyan = 3,
    Red = 4, Magenta = 5, Brown = 6, LightGray = 7,
    DarkGray = 8, LightBlue = 9, LightGreen = 10, LightCyan = 11,
    LightRed = 12, Pink = 13, Yellow = 14, White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    pub column_position: usize,
    pub row_position: usize,
    // On mémorise la fin de chaque ligne pour le backspace intelligent
    line_lengths: [usize; BUFFER_HEIGHT],
    pub color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });
                self.column_position += 1;
                // On met à jour la longueur de la ligne actuelle
                self.line_lengths[self.row_position] = self.column_position;
            }
        }
        self.update_cursor(); 
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn new_line(&mut self) {
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            // Scroll : on décale aussi les longueurs de lignes
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
                self.line_lengths[row - 1] = self.line_lengths[row];
            }
            self.clear_row(BUFFER_HEIGHT - 1);
            self.line_lengths[BUFFER_HEIGHT - 1] = 0;
        }
        self.column_position = 0;
        self.update_cursor();
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
        self.line_lengths[row] = 0;
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
        self.row_position = 0;
        self.update_cursor();
    }

    pub fn set_color_code(&mut self, color_code: ColorCode) {
        self.color_code = color_code;
    }

    pub fn update_cursor(&mut self) {
        let pos = self.row_position * BUFFER_WIDTH + self.column_position;
        unsafe {
            let mut addr_port = Port::new(0x3D4);
            let mut data_port = Port::new(0x3D5);
            addr_port.write(0x0F as u8);
            data_port.write((pos & 0xFF) as u8);
            addr_port.write(0x0E as u8);
            data_port.write(((pos >> 8) & 0xFF) as u8);
        }
    }

    pub fn backspace(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
        } else if self.row_position > 0 {
            // REMONTÉE INTELLIGENTE
            self.row_position -= 1;
            // On se remet à la fin du texte de la ligne du dessus
            self.column_position = self.line_lengths[self.row_position];
            
            // Si la ligne était pleine (80), on recule d'un cran pour pouvoir effacer
            if self.column_position >= BUFFER_WIDTH {
                self.column_position = BUFFER_WIDTH - 1;
            }
            
            // Si la ligne est vide (juste un \n), on ne fait rien de plus
            if self.column_position == 0 {
                self.update_cursor();
                return;
            }
            self.column_position -= 1;
        } else {
            return;
        }

        let row = self.row_position;
        let col = self.column_position;
        self.buffer.chars[row][col].write(ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        });
        // On met à jour la mémoire de longueur de ligne
        self.line_lengths[self.row_position] = self.column_position;
        self.update_cursor();
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        line_lengths: [0; BUFFER_HEIGHT],
        color_code: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

pub fn clear_screen() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().clear_screen();
    });
}

pub fn backspace() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().backspace();
    });
}

pub fn print_char(c: char) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().write_byte(c as u8);
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writer.write_fmt(args).unwrap();
    });
}