extern crate ncurses;
extern crate libc;
use std::ptr::null_mut;
use color::{Color, ColorPair};
use color::Color::*;
use terminal::Terminal;

pub struct SystemTerminal {
    default_color: ColorPair,
}

/// Initializes system terminal. **WARNING: Must NOT be called more than once in runtime.**
pub fn init_terminal() -> Result<SystemTerminal,()> {
    if ncurses::initscr() == null_mut()
    || ncurses::noecho() != ncurses::OK
    || ncurses::start_color() != ncurses::OK {
        return Err(())
    }
        
    for i in 0..8 {
        for j in 0..8 {
            if ncurses::init_pair(i * 8 + j + 1, i, j) != ncurses::OK {
                return Err(())
            }
        }
    }
    
    Ok(SystemTerminal { default_color: ColorPair { fg: Some(DarkWhite), bg: Some(DarkBlack) }})
}

impl Terminal for SystemTerminal {
    fn size(&self) -> (u16, u16) {
        (0, 0)
    }

    fn resize(&mut self, width: u16, height: u16) -> Result<(),()> {
        Err(())
    }

    fn get_cursor_pos(&self) -> (u16, u16) {
        (0, 0)
    }

    fn set_cursor_pos(&mut self, x: u16, y: u16) ->  Result<(),()> {
        Err(())
    }

    fn get_default_color(&self) -> ColorPair {
        self.default_color
    }

    fn set_default_color(&mut self, color: ColorPair) {
        self.default_color = color.compose(self.default_color);
    }

    fn write_colored<T>(&mut self, s: &str, colors: T) where T: Iterator<Item=ColorPair> {
        for (ch, co) in s.chars().zip(colors) {
            set_color(co.compose(self.default_color));
            ncurses::addch(ch as u64);
        }
        ncurses::refresh();
    }
}

impl Drop for SystemTerminal {
    fn drop(&mut self) {

    }
}

fn set_color(color: ColorPair) {
    ncurses::attrset(match color.fg.unwrap() {
        LightBlack| LightRed | LightGreen | LightYellow | LightBlue | LightMagenta | LightCyan
        | LightWhite => ncurses::A_BOLD(),
        _ => 0,
    } | ncurses::COLOR_PAIR(match color.fg.unwrap() {
        DarkBlack | LightBlack => 0,
        DarkRed | LightRed => 1,
        DarkGreen | LightGreen => 2,
        DarkYellow | LightYellow => 3,
        DarkBlue | LightBlue => 4,
        DarkMagenta | LightMagenta => 5,
        DarkCyan | LightCyan => 6,
        DarkWhite | LightWhite => 7,
    } * 8 + match color.bg.unwrap() {
        DarkBlack | LightBlack => 0,
        DarkRed | LightRed => 1,
        DarkGreen | LightGreen => 2,
        DarkYellow | LightYellow => 3,
        DarkBlue | LightBlue => 4,
        DarkMagenta | LightMagenta => 5,
        DarkCyan | LightCyan => 6,
        DarkWhite | LightWhite => 7,
    } + 1));
}