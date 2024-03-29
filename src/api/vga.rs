use core::fmt;
use core::fmt::Write;

use volatile::Volatile;
use lazy_static::lazy_static;
use spin::Mutex;

use x86_64::instructions::interrupts;

const HEIGHT: usize = 25;
const WIDTH: usize = 80;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        row: 0,
        column: 0,
        color_entry: ColorEntry::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorEntry(u8);

impl ColorEntry {
    fn new(foreground: Color, background: Color) -> ColorEntry {
        ColorEntry((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Char {
    ascii: u8,
    color_entry: ColorEntry,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<Char>; WIDTH]; HEIGHT],
}

pub struct Writer {
    row: usize,
    column: usize,
    color_entry: ColorEntry,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                if self.column >= WIDTH {
                    self.newline();
                }
                let row = self.row;
                let column = self.column;
                let color_entry = self.color_entry;
                self.buffer.chars[row][column].write(Char {
                    ascii: byte,
                    color_entry
                });
                self.column += 1;
            }
        }
    }

    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn newline(&mut self) {
        self.row += 1;
        self.column = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}


#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::api::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}