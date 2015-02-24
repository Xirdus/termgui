extern crate "kernel32-sys" as kernel32_sys;
extern crate winapi;
use std::mem::uninitialized;
use std::ptr::null_mut;
use std::cmp::{min, max};
use std::iter::repeat;
use color::*;
use color::Color::*;

trivial_error! {
    TerminalError = "Couldn't initialize terminal";
    ResizeError = "Couldn't resize terminal";
    CursorError = "Couldn't set cursor to given position";
}

pub struct Terminal {
    buffer: Vec<winapi::CHAR_INFO>,
    term_width: u16,
    cur_pos: usize,
    default_color: ColorPair,

    out_handle: winapi::HANDLE,
    old_out_handle: winapi::HANDLE,
}

impl Terminal {
    pub fn initialize() -> Result<Terminal,TerminalError> {
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
            let mut term = Terminal {
                buffer: Vec::new(),
                term_width: 0,
                cur_pos: 0,
                default_color: ColorPair {
                    fg: Some(DarkWhite),
                    bg: Some(DarkBlack),
                },
                out_handle: handle,
                old_out_handle: old_handle,
            };
            term.set_dimensions(x as u16, y as u16);
            kernel32_sys::SetConsoleActiveScreenBuffer(handle);
            Ok(term)
        }
    }

    pub fn size(&self) -> (u16, u16) {
        (self.term_width, (self.buffer.len() / self.term_width as usize) as u16)
    }

    fn get_real_size(&self) -> (u16, u16) {
        let mut info = unsafe {uninitialized()};
        unsafe {
            kernel32_sys::GetConsoleScreenBufferInfo(self.out_handle, &mut info);
        }
        ((info.srWindow.Right - info.srWindow.Left + 1) as u16,
         (info.srWindow.Bottom - info.srWindow.Top + 1) as u16)
    }

    pub fn resize(&mut self, width: u16, height: u16) -> Result<(),ResizeError> {
        let mut rt = Ok(());
        self.update_console_window(width, height);
        let (new_width, new_height) = self.get_real_size();
        if (width, height) != (new_width, new_height) {
            rt = Err(ResizeError);
            self.update_console_window(new_width, new_height);
        }
        self.set_dimensions(new_width, new_height);
        rt
    }

    fn update_console_window(&mut self, width: u16, height: u16) {
        let (term_x, term_y) = self.size();
        unsafe {
            kernel32_sys::SetConsoleScreenBufferSize(self.out_handle,
                winapi::COORD {
                    X: max(term_x, width) as i16,
                    Y: max(term_y, height) as i16,
                });
            kernel32_sys::SetConsoleWindowInfo(self.out_handle, winapi::TRUE,
                &winapi::SMALL_RECT {
                    Left: 0,
                    Top: 0,
                    Right: (width - 1) as i16,
                    Bottom: (height - 1) as i16,
                });
            kernel32_sys::SetConsoleScreenBufferSize(self.out_handle,
                winapi::COORD {
                    X: width as i16,
                    Y: height as i16,
                });
        }
    }

    fn set_dimensions(&mut self, width: u16, height: u16) {
        self.term_width = width;
        self.buffer.resize((width * height) as usize,
            winapi::CHAR_INFO {
                Char: 0,
                Attributes: convert_color_pair(self.default_color),
            });
    }

    pub fn get_cursor_pos(&self) -> (u16, u16) {
        (self.cur_pos as u16 % self.term_width, (self.cur_pos / self.term_width as usize) as u16)
    }

    pub fn set_cursor_pos(&mut self, x: u16, y: u16) ->  Result<(),CursorError> {
        unsafe {
            kernel32_sys::SetConsoleCursorPosition(self.out_handle,
                winapi::COORD {
                    X: x as i16,
                    Y: y as i16,
                });
        }
        let mut info = unsafe { uninitialized() };
        unsafe {
            kernel32_sys::GetConsoleScreenBufferInfo(self.out_handle, &mut info);
        }
        let (real_x, real_y) = (info.dwCursorPosition.X as u16, info.dwCursorPosition.Y as u16);
        self.cur_pos = (real_y * self.term_width + real_x) as usize;
        if (x, y) == (real_x, real_y) {
            Ok(())
        } else {
            Err(CursorError)
        }
    }

    pub fn get_default_color(&self) -> ColorPair {
        self.default_color
    }

    pub fn set_default_color(&mut self, color: ColorPair) {
        self.default_color = color.compose(self.default_color);
    }

    pub fn write(&mut self, s: &str) {
        let color = self.default_color;
        self.write_in_color(s, color);
    }

    pub fn write_in_color(&mut self, s: &str, color: ColorPair) {
        self.write_colored(s, repeat(color));
    }

    pub fn write_colored<T>(&mut self, s: &str, colors: T) where T: Iterator<Item=ColorPair> {
        for ((out, char), col) in self.buffer[self.cur_pos..].iter_mut()
                                  .zip(s.utf16_units())
                                  .zip(colors) {
            *out = winapi::CHAR_INFO {
                Char: char,
                Attributes: convert_color_pair(col.compose(self.default_color)),
            };
            self.cur_pos += 1;
        }
        self.cur_pos = min(self.cur_pos, self.buffer.len() - 1);
        let (x, y) = self.get_cursor_pos();
        self.set_cursor_pos(x, y).ok();
        self.present();
    }

    fn present(&mut self) {
        let (x, y) = self.size();
        let (x, y) = (x as i16, y as i16);
        unsafe {
            kernel32_sys::WriteConsoleOutputW(self.out_handle, self.buffer.as_ptr(),
                winapi::COORD {
                    X: x, Y: y
                }, winapi::COORD {
                    X: 0, Y: 0
                }, &mut winapi::SMALL_RECT {
                    Left: 0,
                    Top: 0,
                    Right: x - 1,
                    Bottom: y - 1,
                });
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe {
            kernel32_sys::SetConsoleActiveScreenBuffer(self.old_out_handle);
            kernel32_sys::CloseHandle(self.out_handle);
        }
    }
}

fn convert_color(color: Color) -> winapi::WORD {
    use self::winapi::*;
    (match color {
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
    }) as u16
}

fn convert_color_pair(color: ColorPair) -> winapi::WORD {
    convert_color(color.fg.unwrap()) | (convert_color(color.bg.unwrap()) << 4)
}
