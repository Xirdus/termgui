extern crate "kernel32-sys" as kernel32_sys;
extern crate winapi;
use std::mem::uninitialized;
use std::ptr::null_mut;
use std::cmp::max;
use std::iter::repeat;
use std::num::ToPrimitive;
use color::*;
use color::Color::*;

trivial_error! {
    TerminalError = "Unable to get terminal";
}

pub enum CursorType {
    None,
    Small,
    Big
}

pub struct Terminal {
    handle: winapi::HANDLE,
    old_handle: winapi::HANDLE,
    default_color: ColorPair
}

pub fn init_terminal() -> Result<Terminal, TerminalError> {
    unsafe {
        if kernel32_sys::GetConsoleWindow().is_null() {
            if kernel32_sys::AllocConsole() == winapi::FALSE
              || kernel32_sys::GetConsoleWindow().is_null() {
                return Err(TerminalError)
            }
        }
        let old_handle = kernel32_sys::GetStdHandle(winapi::STD_OUTPUT_HANDLE);
        let handle = kernel32_sys::CreateConsoleScreenBuffer(
            winapi::GENERIC_READ | winapi::GENERIC_WRITE,
            0, null_mut(), winapi::CONSOLE_TEXTMODE_BUFFER, null_mut());
        let mut info = uninitialized();
        kernel32_sys::GetConsoleScreenBufferInfo(handle, &mut info);
        let (x, y) = (info.srWindow.Right - info.srWindow.Left + 1,
                      info.srWindow.Bottom - info.srWindow.Top + 1);
        let term = Terminal {
            handle: handle,
            old_handle: old_handle,
            default_color: ColorPair {
                fg: Some(DarkWhite),
                bg: Some(DarkBlack)
            }
        };
        term.resize(x, y);
        kernel32_sys::SetConsoleActiveScreenBuffer(handle);

        Ok(term)
    }
}

impl Terminal {
    pub fn size(&self) -> (i16, i16) {
        let mut info = unsafe {uninitialized()};
        unsafe {
            kernel32_sys::GetConsoleScreenBufferInfo(self.handle, &mut info);
        }
        (info.srWindow.Right - info.srWindow.Left + 1,
         info.srWindow.Bottom - info.srWindow.Top + 1)
    }

    pub fn resize(&self, x: i16, y: i16) {
        let (old_x, old_y) = self.size();
        unsafe {
            kernel32_sys::SetConsoleScreenBufferSize(self.handle,
                winapi::COORD {X: max(x, old_x), Y: max(y, old_y)});
            kernel32_sys::SetConsoleWindowInfo(self.handle, winapi::TRUE,
                &winapi::SMALL_RECT {
                    Left: 0,
                    Top: 0,
                    Right: x-1,
                    Bottom: y-1
                });
            kernel32_sys::SetConsoleScreenBufferSize(self.handle,
                winapi::COORD {X: x, Y: y});
        }
    }

    fn get_cursor_position(&self) -> (i16, i16) { // don't want to make it pub
        let mut info = unsafe {uninitialized()};
        unsafe {
            kernel32_sys::GetConsoleScreenBufferInfo(self.handle, &mut info);
        }
        (info.dwCursorPosition.X, info.dwCursorPosition.Y)
    }

    pub fn set_cursor_position(&self, x: i16, y: i16) {
        unsafe {
            kernel32_sys::SetConsoleCursorPosition(self.handle, winapi::COORD {X: x, Y: y});
        }
    }

    pub fn set_cursor_type(&self, cursor: CursorType) {
        let info = winapi::CONSOLE_CURSOR_INFO {
            dwSize: match cursor {
                CursorType::Big => 100,
                _ => 25
            },
            bVisible: match cursor {
                CursorType::None => winapi::FALSE,
                _ => winapi::TRUE
            }
        };
        unsafe {
            kernel32_sys::SetConsoleCursorInfo(self.handle, &info);
        }
    }

    pub fn set_default_color(&mut self, color: ColorPair) {
        self.default_color = color.compose(self.default_color);
    }

    pub fn write(&self, s: &str) {
        self.write_with_color(s, self.default_color);
    }

    pub fn write_with_color(&self, s: &str, color: ColorPair) {
        self.write_colored(s, repeat(color));
    }

    pub fn write_colored<T>(&self, s: &str, colors: T)
    where T: Iterator<Item=ColorPair> {
        let array = s.utf16_units().zip(colors).map(|(char, color)| {
            let color = color.compose(self.default_color);
            winapi::CHAR_INFO {
                Char: char,
                Attributes: convert_color_pair(color)
            }
        }).collect::<Vec<_>>();
        let (cur_x, cur_y) = self.get_cursor_position();
        let (term_x, term_y) = self.size();
        unsafe {
            kernel32_sys::WriteConsoleOutputW(self.handle, array.as_slice().as_ptr(),
                winapi::COORD {X: array.len().to_i16().unwrap(), Y: 1}, winapi::COORD {X: 0, Y: 0},
                &mut winapi::SMALL_RECT {Left: cur_x, Top: cur_y, Right: term_x, Bottom: term_y});
        }
        self.set_cursor_position(cur_x + s.len().to_i16().unwrap(), cur_y);
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe {
            kernel32_sys::SetConsoleActiveScreenBuffer(self.old_handle);
            kernel32_sys::CloseHandle(self.handle);
        }
    }
}

fn convert_color(color: Color) -> winapi::WORD {
    use self::winapi::*;
    match color {
        DarkBlack => 0,
        DarkRed => FOREGROUND_RED,
        DarkGreen => FOREGROUND_GREEN,
        DarkBlue => FOREGROUND_BLUE,
        DarkCyan => FOREGROUND_GREEN | FOREGROUND_BLUE,
        DarkMagenta => FOREGROUND_RED | FOREGROUND_BLUE,
        DarkYellow => FOREGROUND_RED | FOREGROUND_GREEN,
        DarkWhite => FOREGROUND_RED | FOREGROUND_GREEN | FOREGROUND_BLUE,
        LightBlack => FOREGROUND_RED | FOREGROUND_INTENSITY,
        LightRed => FOREGROUND_RED | FOREGROUND_INTENSITY,
        LightGreen => FOREGROUND_GREEN | FOREGROUND_INTENSITY,
        LightBlue => FOREGROUND_BLUE | FOREGROUND_INTENSITY,
        LightCyan => FOREGROUND_GREEN | FOREGROUND_BLUE | FOREGROUND_INTENSITY,
        LightMagenta => FOREGROUND_RED | FOREGROUND_BLUE | FOREGROUND_INTENSITY,
        LightYellow => FOREGROUND_RED | FOREGROUND_GREEN | FOREGROUND_INTENSITY,
        LightWhite => FOREGROUND_RED | FOREGROUND_GREEN | FOREGROUND_BLUE | FOREGROUND_INTENSITY
    }.to_u16().unwrap()
}

fn convert_color_pair(color: ColorPair) -> winapi::WORD {
    convert_color(color.fg.unwrap()) | (convert_color(color.bg.unwrap()) << 4)
}
