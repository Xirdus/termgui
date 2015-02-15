extern crate "kernel32-sys" as kernel32_sys;
extern crate winapi;
use self::winapi::*;
use std::mem::uninitialized;
use std::mem::zeroed;
use std::ptr::null_mut;
use std::cmp::max;

trivial_error! {
    TerminalError = "Unable to get terminal";
}

pub struct Terminal {
    handle: winapi::HANDLE,
    old_handle: winapi::HANDLE,
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
        let mut term = Terminal {
            handle: handle,
            old_handle: old_handle
        };
        term.resize(x, y);
        kernel32_sys::SetConsoleActiveScreenBuffer(handle);

        Ok(term)
    }
}

impl Terminal {
    pub fn size(&self) -> (i16, i16) {
        let mut info = unsafe { zeroed() };
        unsafe {
            kernel32_sys::GetConsoleScreenBufferInfo(self.handle, &mut info);
        }
        (info.srWindow.Right - info.srWindow.Left + 1,
         info.srWindow.Bottom - info.srWindow.Top + 1)
    }

    pub fn resize(&mut self, x: i16, y: i16) {
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
}

impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe {
            kernel32_sys::SetConsoleActiveScreenBuffer(self.old_handle);
            kernel32_sys::CloseHandle(self.handle);
        }
    }
}
